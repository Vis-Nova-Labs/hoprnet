import { EventEmitter } from 'events'
import type { PeerStoreType, PublicNodesEmitter } from '../types'
import Debug from 'debug'
import { red } from 'chalk'

import { CODE_P2P, CODE_IP4, CODE_IP6, CODE_TCP, CODE_UDP, MAX_RELAYS_PER_NODE } from '../constants'
import type { Connection } from 'libp2p-interfaces/connection'

import type PeerId from 'peer-id'
import { Multiaddr } from 'multiaddr'

import { nAtATime } from '@hoprnet/hopr-utils'
import type HoprConnect from '..'
import { attemptClose } from '../utils'

const log = Debug('hopr-connect:entry')
const error = Debug('hopr-connect:entry:error')

type EntryNodeData = PeerStoreType & {
  latency: number
}

type ConnectionResult = {
  entry: EntryNodeData
  conn?: Connection
}

function latencyCompare(a: ConnectionResult, b: ConnectionResult) {
  return a.entry.latency - b.entry.latency
}

function isUsableRelay(ma: Multiaddr) {
  const tuples = ma.tuples()

  return (
    tuples[0].length >= 2 && [CODE_IP4, CODE_IP6].includes(tuples[0][0]) && [CODE_UDP, CODE_TCP].includes(tuples[1][0])
  )
}

export const ENTRY_NODES_MAX_PARALLEL_DIALS = 14

export class EntryNodes extends EventEmitter {
  protected availableEntryNodes: EntryNodeData[]
  protected uncheckedEntryNodes: PeerStoreType[]

  protected usedRelays: Multiaddr[]

  private _onNewRelay: EntryNodes['onNewRelay'] | undefined
  private _onRemoveRelay: EntryNodes['onRemoveRelay'] | undefined

  constructor(
    private peerId: PeerId,
    initialNodes: PeerStoreType[],
    private publicNodesEmitter: PublicNodesEmitter | undefined,
    private dialDirectly: HoprConnect['dialDirectly']
  ) {
    super()
    this.availableEntryNodes = []
    this.uncheckedEntryNodes = initialNodes

    this.usedRelays = []
  }

  /**
   * Attaches listeners that handle addition and removal of
   * entry nodes
   */
  public start() {
    if (this.publicNodesEmitter != undefined) {
      this._onNewRelay = this.onNewRelay.bind(this)
      this._onRemoveRelay = this.onRemoveRelay.bind(this)

      this.publicNodesEmitter.on('addPublicNode', this._onNewRelay)

      this.publicNodesEmitter.on('removePublicNode', this._onRemoveRelay)
    }
  }

  /**
   * Removes event listeners
   */
  public stop() {
    if (this.publicNodesEmitter != undefined && this._onNewRelay != undefined && this._onRemoveRelay != undefined) {
      this.publicNodesEmitter.removeListener('addPublicNode', this._onNewRelay as EntryNodes['onNewRelay'])

      this.publicNodesEmitter.removeListener('removePublicNode', this._onRemoveRelay as EntryNodes['onRemoveRelay'])
    }
  }

  /**
   * @returns a list of entry nodes that are currently used
   */
  public getUsedRelays() {
    return this.usedRelays
  }

  /**
   * @returns a list of entry nodes that are considered to be online
   */
  public getAvailabeEntryNodes() {
    return this.availableEntryNodes
  }

  /**
   * @returns a list of entry nodes that will be checked once the
   * list of entry nodes is built or rebuilt next time
   */
  public getUncheckedEntryNodes() {
    return this.uncheckedEntryNodes
  }

  /**
   * Called once there is a new relay opportunity known
   * @param ma Multiaddr of node that is added as a relay opportunity
   */
  protected onNewRelay(peer: PeerStoreType) {
    if (peer.id.equals(this.peerId)) {
      return
    }

    if (peer.multiaddrs == undefined || peer.multiaddrs.length == 0) {
      log(`Received entry node ${peer.id.toB58String()} without any multiaddr`)
      return
    }

    for (const uncheckedNode of this.uncheckedEntryNodes) {
      if (uncheckedNode.id.equals(peer.id)) {
        log(`Received duplicate entry node ${peer.id.toB58String()}`)
        // TODO add difference to previous multiaddrs
        return
      }
    }

    this.uncheckedEntryNodes.push({
      id: peer.id,
      multiaddrs: peer.multiaddrs.filter(isUsableRelay)
    })
  }

  /**
   * Called once a node is considered to be offline
   * @param ma Multiaddr of node that is considered to be offline now
   */
  protected onRemoveRelay(peer: PeerId) {
    for (const [index, publicNode] of this.availableEntryNodes.entries()) {
      if (publicNode.id.equals(peer)) {
        // Remove node without changing order
        this.availableEntryNodes.splice(index, 1)
      }
    }

    let inUse = false
    const peerB58String = peer.toB58String()
    for (const [index, relayAddr] of this.usedRelays.entries()) {
      // remove second part of relay address to get relay peerId
      if (relayAddr.decapsulateCode(CODE_P2P).getPeerId() === peerB58String) {
        // Remove node without changing order
        this.usedRelays.splice(index, 1)
        inUse = true
      }
    }

    log(
      `relay ${peer.toB58String()} ${red(`removed`)}. Current addrs:\n\t${this.usedRelays
        .map((addr: Multiaddr) => addr.toString())
        .join(`\n\t`)}`
    )

    // Only rebuild list of relay nodes if we were using the
    // offline node
    if (inUse) {
      // Rebuild later
      setImmediate(this.updatePublicNodes.bind(this))
    }
  }

  /**
   * Filters list of unchecked entry nodes before contacting them
   * @returns a filtered list of entry nodes
   */
  private filterUncheckedNodes(): PeerStoreType[] {
    const knownNodes = new Set<string>(this.availableEntryNodes.map((entry: EntryNodeData) => entry.id.toB58String()))
    const nodesToCheck: PeerStoreType[] = []

    for (const uncheckedNode of this.uncheckedEntryNodes) {
      if (uncheckedNode.id.equals(this.peerId)) {
        continue
      }

      const usableAddresses: Multiaddr[] = uncheckedNode.multiaddrs.filter(isUsableRelay)

      if (knownNodes.has(uncheckedNode.id.toB58String())) {
        const index = this.availableEntryNodes.findIndex((entry) => entry.id.equals(uncheckedNode.id))

        if (index < 0) {
          continue
        }

        // Overwrite previous addresses. E.g. a node was restarted
        // and now announces with a different address
        this.availableEntryNodes[index].multiaddrs = usableAddresses

        // Nothing to do. Public nodes are added later
        continue
      }

      // Ignore if entry nodes have more than one address
      nodesToCheck.push({
        id: uncheckedNode.id,
        multiaddrs: [usableAddresses[0]]
      })
    }

    return nodesToCheck
  }

  /**
   * Updates the list of exposed entry nodes.
   * Called at startup and once an entry node is considered offline.
   */
  async updatePublicNodes(): Promise<void> {
    const nodesToCheck = this.filterUncheckedNodes()
    const TIMEOUT = 3e3

    const connectToRelay = this.connectToRelay.bind(this)
    const toCheck = nodesToCheck.concat(this.availableEntryNodes)
    const args: Parameters<typeof connectToRelay>[] = new Array(toCheck.length)

    for (const [index, nodeToCheck] of toCheck.entries()) {
      args[index] = [nodeToCheck.id, nodeToCheck.multiaddrs[0], TIMEOUT]
    }

    // const CONCURRENCY = 14 // connections

    const results = (await nAtATime(connectToRelay, args, ENTRY_NODES_MAX_PARALLEL_DIALS)).sort(latencyCompare)

    const positiveOnes = results.findIndex((result: ConnectionResult) => result.entry.latency >= 0)

    // Close all unnecessary connection
    await nAtATime(
      attemptClose,
      results
        .slice(positiveOnes + MAX_RELAYS_PER_NODE)
        .map<[Connection, (arg: any) => void]>((result) => [result.conn as Connection, error]),
      ENTRY_NODES_MAX_PARALLEL_DIALS
    )

    // Take all entry nodes that appeared to be online
    this.availableEntryNodes = results.slice(positiveOnes).map((result) => result.entry)

    // Reset list of unchecked nodes
    this.uncheckedEntryNodes = []

    const previous = new Set(this.usedRelays.map((ma) => ma.toString()))

    this.usedRelays = this.availableEntryNodes
      // select only those entry nodes with smallest latencies
      .slice(0, MAX_RELAYS_PER_NODE)
      .map(
        (entry: EntryNodeData) =>
          new Multiaddr(`/p2p/${entry.id.toB58String()}/p2p-circuit/p2p/${this.peerId.toB58String()}`)
      )

    let isDifferent = false
    for (const usedRelay of this.usedRelays) {
      if (!previous.has(usedRelay.toString())) {
        isDifferent = true
        break
      }
    }

    if (isDifferent) {
      log(`Current relay addresses:`)
      for (const ma of this.usedRelays) {
        log(`\t${ma.toString()}`)
      }

      this.emit('relay:changed')
    }
  }

  /**
   * Attempts to connect to a relay node
   * @param id peerId of the node to dial
   * @param relay multiaddr to perform the dial
   * @param timeout when to timeout on unsuccessful dial attempts
   * @returns a PeerStoreEntry containing the measured latency
   */
  private async connectToRelay(
    id: PeerId,
    relay: Multiaddr,
    timeout: number
  ): Promise<{ entry: EntryNodeData; conn?: Connection }> {
    const abort = new AbortController()
    const start = Date.now()

    const timeoutHandle = setTimeout(abort.abort.bind(abort), timeout)

    let conn: Connection | undefined
    try {
      conn = await this.dialDirectly(relay, { signal: abort.signal })
    } catch (err: any) {
      error(`error while contacting entry node.`, err.message)
    } finally {
      clearTimeout(timeoutHandle)
    }

    if (conn == undefined) {
      return {
        entry: {
          id,
          multiaddrs: [relay],
          latency: -1
        }
      }
    }

    return {
      conn,
      entry: {
        id,
        multiaddrs: [relay],
        latency: Date.now() - start
      }
    }
  }
}