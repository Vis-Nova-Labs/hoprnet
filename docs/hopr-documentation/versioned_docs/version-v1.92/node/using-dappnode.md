---
id: using-dappnode
title: Using Dappnode
---

To set up your Dappnode, follow the instructions that came with the box. Then, just install the HOPR client and you can start using your node right away!

## Installing the HOPR Client: 1.92.8 (Monte Rosa)

While connected to your Dappnode's network or via a VPN, go to the following [link](http://my.dappnode/#/installer/hopr.public.dappnode.eth). Just click the install button and wait until the install completes.

**OR Install From the DAppStore**

(**1**) Open the DAppStore using the sidebar to the left.

<img width="1604" alt="Screenshot 2023-03-17 at 1 27 33 AM" src="https://user-images.githubusercontent.com/4649787/225830875-ed154ab1-0e90-45d4-b3ca-7ace73337aeb.png">

(**2**) Select the Hopr Package in the Red Box, and click install.

P.S Sometimes, after trying to Install or Update, it will give you an error. Just re-install or update the package if this happens.

That's all! You should now be able to find the HOPR client in your 'Packages'.

![MyDapps](/img/node/Dappnode-2.png)

Simply open the client, and you should be greeted with the hopr-admin interface.

Your **security token** is set to `!5qxc9Lp1BE7IFQ-nrtttU`. You will need this to access hopr-admin.

If you are in the process of registering your node on the network registry, please complete the process [here](./network-registry-tutorial.md) before continuing.

Otherwise, the installation process is complete! You can proceed to our [hopr-admin tutorial](using-hopr-admin).

### Restoring an old node

If you have previously installed a node and have the [identity file downloaded](using-hopr-admin#backing-up-your-identity-file), you can use it to restore your old node.

**Note:** For Dappnode, you should download the latest version of HOPR before trying to restore your node.

Find HOPR in your packages and navigate to the backup section. From there, all you have to do is click 'Restore' and open your [zipped backup file](using-hopr-admin#backing-up-your-identity-file) when prompted.

![dappnode restore](/img/node/dappnode-backup.png)

## Collecting Logs

If your node crashes, you will want to collect the logs and pass them on to our ambassadors on telegram or create an issue on GitHub.

To collect the logs:

(**1**) Find HOPR in your packages and navigate to the backup section.

![Dappnode Logs](/img/node/Dappnode-logs.png)

(**2**) From there, all you have to do is click 'Download all'.

Using the downlaoded file either:

- Send it to an ambassador on our [telegram](https://t.me/hoprnet) for assistance.
- Or, create an issue using our bug template on [GitHub.](https://github.com/hoprnet/hoprnet/issues)

## Using a Custom RPC Endpoint

You can set your own RPC endpoint for HOPR to use. Ideally, you would install an Gnosis Chain Stack on your DAppNode and use it as a local provider. A local provider helps increase decentralisation and is generally good practice, but you can also use any 3rd party RPC provider of your choice as long as they are on Gnosis Chain (Formerly xDai).

**Note:** Only RPC providers on Gnosis chain will work with HOPR

### Using a your local endpoint on Dappnode

You need to install a full Gnosis Chain stack in order to run a local Gnosis/xDai node to use it as a local endpoint.

(**1**) Follow this [Link](http://my.dappnode/#/stakers/gnosis) to select and configure your Gnosis Chain clients. 

(**2**) Currently only Nethermind xDai is available as an execution client for Gnosis chain, so select it from the Execution Column (In Red Box)

(**3**) Currently only Lighthouse Gnosis and Teku Gnosis are available as consensus clients for Gnosis chain, so select one of them from their Column and enter an address you have full control over, using checkpoint sync is much faster to get up and running.  (options are in Green Boxes)

(**4**) Finally Click the Apply Button in the Blue Box and The clients will be installed and start syncing.

<img width="1667" alt="Screenshot 2023-03-17 at 1 09 48 AM" src="https://user-images.githubusercontent.com/4649787/225829118-4a608ff9-f6ea-4095-8dc6-1a63e8bdcbca.png">

Because Nethermind xDai is the only supported Gnosis Chain Execution Client currently, once you have installed and synced the Gnosis Chain stack as described above the endpoint for Nethermind xDai, your local endpoint on Dappnode will be `http://nethermind-xdai.dappnode:8545`

### Remote endpoint

Otherwise, you can use any non-local RPC provider such as [ankr.](https://www.ankr.com/)

### Changing your RPC endpoint

To change your RPC endpoint:

(**1**) Find HOPR in your packages and navigate to the 'Config' section.

![RPC Prpvider Dappnode](/img/node/HOPR-provider-Dappnode.png)

(**2**) Paste your custom RPC endpoint in the text field under `RPC Provider URL`.

(**3**) Click 'Update' and wait for your node to restart.

All done! Your Dappnode Hopr Node will now use your specified RPC endpoint.
