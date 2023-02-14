import type { PeerId } from '@libp2p/interface-peer-id'

import { durations, pickVersion, pubKeyToPeerId, type HoprDB, create_counter } from '@hoprnet/hopr-utils'
import { debug } from '@hoprnet/hopr-utils'

import { AsyncIterableQueue } from 'async-iterable-queue'

import { Packet } from '../../messages/index.js'
import { AsyncIterableHelperMixer, core_mixer_set_panic_hook } from '../../../lib/core_mixer_bg.js'
core_mixer_set_panic_hook()

import type { AcknowledgementInteraction } from './acknowledgement.js'
import type { HoprOptions, SendMessage } from '../../index.js'
import type { ResolvedEnvironment } from '../../environment.js'

import type { Components } from '@libp2p/interfaces/components'

const log = debug('hopr-core:packet:forward')
const error = debug('hopr-core:packet:forward:error')

const FORWARD_TIMEOUT = durations.seconds(6)

// Metrics
const metric_fwdMessageCount = create_counter('core_counter_forwarded_messages', 'Number of forwarded messages')
const metric_recvMessageCount = create_counter('core_counter_received_messages', 'Number of received messages')

// Do not type-check JSON files
// @ts-ignore
import pkg from '../../../package.json' assert { type: 'json' }

const NORMALIZED_VERSION = pickVersion(pkg.version)

export class PacketForwardInteraction {
  protected mixer: AsyncIterableHelperMixer
  protected packetQueue: AsyncIterableQueue<Packet>

  public readonly protocols: string | string[]

  constructor(
    private libp2pComponents: Components,
    private sendMessage: SendMessage,
    private privKey: PeerId,
    private emitMessage: (msg: Uint8Array) => void,
    private db: HoprDB,
    private environment: ResolvedEnvironment,
    private acknowledgements: AcknowledgementInteraction,
    private options: HoprOptions
  ) {
    this.packetQueue = new AsyncIterableQueue<Packet>()
    this.mixer = new AsyncIterableHelperMixer(this.packetQueue)

    this.protocols = [
      // current
      `/hopr/${this.environment.id}/msg/${NORMALIZED_VERSION}`,
      // deprecated
      `/hopr/${this.environment.id}/msg`
    ]
  }

  private errHandler(err: any) {
    error(`Error while receiving packet`, err)
  }

  async start() {
    this.libp2pComponents.getRegistrar().handle(this.protocols, async ({ connection, stream }) => {
      try {
        for await (const chunk of stream.source) {
          const packet = Packet.deserialize(chunk, this.privKey, connection.remotePeer)

          this.packetQueue.push(packet)
        }
      } catch (err) {
        this.errHandler(err)
      }
    })

    this.handleMixedPackets()
  }

  stop() {
    this.packetQueue.end().then(function (_) {})
  }

  async handleMixedPackets() {
    console.log(this.mixer.next)
    let self = this
    for await (const packet of {
      [Symbol.asyncIterator]() {
        return {
          next: self.mixer.next()
        }
      }
    }) {
      await this.handleMixedPacket(packet)
    }
  }

  async interact(counterparty: PeerId, packet: Packet): Promise<void> {
    await this.sendMessage(counterparty, this.protocols, packet.serialize(), false, {
      timeout: FORWARD_TIMEOUT
    })
  }

  async handleMixedPacket(packet: Packet) {
    await packet.checkPacketTag(this.db)

    if (packet.isReceiver) {
      this.emitMessage(packet.plaintext)
      // Send acknowledgements independently
      this.acknowledgements.sendAcknowledgement(packet, packet.previousHop.toPeerId())
      metric_recvMessageCount.increment()
      // Nothing else to do
      return
    }

    // Packet should be forwarded
    try {
      await packet.validateUnacknowledgedTicket(this.db, this.options.checkUnrealizedBalance)
    } catch (err) {
      log(`Ticket validation failed. Dropping packet`, err)
      return
    }

    await packet.storeUnacknowledgedTicket(this.db)

    try {
      await packet.forwardTransform(this.privKey, this.db)
    } catch (err) {
      log(`Packet transformation failed. Dropping packet`, err)
      return
    }

    try {
      await this.interact(pubKeyToPeerId(packet.nextHop), packet)
    } catch (err) {
      log(`Forwarding transformed packet failed.`, err)
      return
    }

    // Send acknowledgements independently
    this.acknowledgements.sendAcknowledgement(packet, packet.previousHop.toPeerId())
    metric_fwdMessageCount.increment()
  }
}
