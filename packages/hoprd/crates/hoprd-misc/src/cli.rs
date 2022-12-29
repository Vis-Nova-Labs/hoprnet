use clap::builder::{
    IntoResettable, PossibleValue, PossibleValuesParser, Resettable, Str, StringValueParser,
    TypedValueParser,
};
use clap::{Arg, ArgAction, ArgMatches, Command};
use real_base::real;
use serde::Serialize;
use serde_json;
use serde_json::{Map, Value};
use wasm_bindgen::JsValue;

const DEFAULT_ID_PATH: &str = ".hopr-identity";

#[derive(serde::Deserialize)]
pub struct ProtocolConfigFile {
    environments: Map<String, Value>,
}

#[derive(Serialize, Parser)]
struct Args {
    enviromment: String,
    api_port: u16,
    api_host: String,
}

impl From<ArgMatches> for Args {
    fn from(m: ArgMatches) -> Self {
        Args {
            enviromment: m.get_one::<String>("name").cloned().unwrap(),
            api_port: m.get_one::<u16>("apiPort").cloned().unwrap(),
            api_host: m.get_one("apiHost").cloned().unwrap(),
        }
    }
}

#[derive(serde::Deserialize)]
struct PackageJsonFile {
    version: String,
}

fn get_package_version(path: String) -> Result<String, JsValue> {
    let data = real::read_file(&path)?;

    match serde_json::from_slice::<PackageJsonFile>(&data) {
        Ok(json) => Ok(json.version),
        Err(e) => Err(JsValue::from(e.to_string())),
    }
}

fn get_environments(path: String) -> Result<Vec<String>, JsValue> {
    let data = real::read_file(&path)?;

    let protocolConfig = serde_json::from_slice::<ProtocolConfigFile>(&data)
        .map_err(|e| JsValue::from(e.to_string()))?;

    Ok(protocolConfig
        .environments
        .iter()
        .map(|env| env.0.to_owned())
        .collect::<Vec<String>>())
}

pub fn parse_cli_arguments(cli_args: Vec<&str>) -> Result<JsValue, JsValue> {
    let envs: Vec<String> = get_environments(String::from("./packages/core/protocol-config.json"))?;

    let version = get_package_version(String::from("./package.json"))?;

    let cmd = Command::new("hoprd")
    .after_help("All CLI options can be configured through environment variables as well. CLI parameters have precedence over environment variables.")
        .version(&version)
        .arg(
            Arg::new("apiHost")
                .long("apiHost")
                .default_value("localhost")
                .env("HOPRD_API_HOST")
                .value_name("HOST")
                .help("Set host IP to which the API server will bind"))
        .arg(
            Arg::new("apiPort")
                .long("apiPort")
                .default_value("3001")
                .value_parser(clap::value_parser!(u16))
                .env("HOPRD_API_PORT")
                .help("Set host port to which the API server will bind."))
        .arg(
            Arg::new("environment")
                .long("environment")
                .required(true)
                .env("HOPRD_ENVIRONMENT")
                .value_name("ENVIRONMENT")
                .help("Environment id which the node shall run on")
                .value_parser(PossibleValuesParser::new(envs)))
        .arg(
            Arg::new("api")
                .long("api")
                .env("HOPRD_API")
                .action(ArgAction::SetTrue)
                .help("Expose the API on localhost:3001")
                .default_value("false"))
        .arg(Arg::new("apiToken")
                .long("apiToken")
                .env("HOPRD_API_TOKEN")
                .help("A REST API token and for user authentication"))
        .arg(Arg::new("healthCheck")
                .long("healthCheck")
                .env("HOPRD_HEALTH_CHECK")
                .help("Run a health check end point on localhost:8080")
                .action(ArgAction::SetTrue)
            .default_value("false"))
        .arg(Arg::new("healthCheckHost")
                .long("healthCheckHost")
                .env("HOPRD_HEALTH_CHECK_HOST")
                .help("Updates the host for the healthcheck server")
                .default_value("localhost"))
        .arg(Arg::new("healthCheckPort")
                .long("healthCheckPort")
                .env("HOPRD_HEALTH_CHECK_PORT")
                .help("Updates the port for the healthcheck server")
                .default_value("8080"))
        .arg(Arg::new("password")
                .long("password")
                .help("A password to encrypt your keys")
                .env("HOPRD_PASSWORD")
                .default_value(""))
        .arg(Arg::new("provider")
                .long("provider")
                .help("A custom RPC provider to be used for the node to connect to blockchain")
                .env("HOPRD_PROVIDER"))
        .arg(Arg::new("identity")
                .long("identity")
                .help("The path to the identity file")
                .env("HOPRD_IDENTITY")
                .default_value(DEFAULT_ID_PATH))
        .arg(Arg::new("dryRun")
                .long("dryRun")
                .help("List all the options used to run the HOPR node, but quit instead of starting")
                .env("HOPRD_DRY_RUN")
                .default_value("false")
                .action(ArgAction::SetTrue));

    // .option('data', {
    //   string: true,
    //   describe: 'manually specify the data directory to use [env: HOPRD_DATA]',
    //   default: defaultDataPath
    // })
    // .option('init', {
    //   boolean: true,
    //   describe: "initialize a database if it doesn't already exist [env: HOPRD_INIT]",
    //   default: false
    // })
    // .option('privateKey', {
    //   hidden: true,
    //   string: true,
    //   describe: 'A private key to be used for the node [env: HOPRD_PRIVATE_KEY]',
    //   default: undefined
    // })
    // .option('allowLocalNodeConnections', {
    //   boolean: true,
    //   describe: 'Allow connections to other nodes running on localhost [env: HOPRD_ALLOW_LOCAL_NODE_CONNECTIONS]',
    //   default: false
    // })
    // .option('allowPrivateNodeConnections', {
    //   boolean: true,
    //   describe:
    //     'Allow connections to other nodes running on private addresses [env: HOPRD_ALLOW_PRIVATE_NODE_CONNECTIONS]',
    //   default: false
    // })
    let args = match cmd.try_get_matches_from(cli_args) {
        Ok(matches) => Args::from(matches),
        Err(e) => return Err(JsValue::from(e.to_string())),
    };

    match serde_wasm_bindgen::to_value(&args) {
        Ok(s) => Ok(s),
        Err(e) => Err(JsValue::from(e.to_string())),
    }
    // Args::try_update_from(
    //     ,
    //     cli_args,
    // );

    // real::read_file("../package.json")
    // .map(|data| {
    //     serde_json::from_slice::<PackageJsonFile>(data)
    //         .map(|json| json.version)
    //         .map_err(|e| JsValue::from(e))
    // })
    // .map_err(|e| JsValue::from(e))

    // serde_json::from_slice(&data)
}

pub mod wasm {
    use js_sys::JsString;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsValue;

    /// Macro used to convert Vec<JsString> to Vec<&str>
    macro_rules! convert_from_jstrvec {
        ($v:expr,$r:ident) => {
            let _aux: Vec<String> = $v.iter().map(String::from).collect();
            let $r = _aux.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
        };
    }

    #[wasm_bindgen]
    pub fn parse_cli_arguments(
        cli_args: Vec<JsString>,
        envs: &JsValue,
    ) -> Result<JsValue, JsValue> {
        convert_from_jstrvec!(cli_args, cli);

        super::parse_cli_arguments(cli)
    }
}

// .env('HOPRD') // enable options to be set as environment variables with the HOPRD prefix
// .option('environment', {
//   string: true,
//   describe: 'Environment id which the node shall run on (HOPRD_ENVIRONMENT)',
//   choices: supportedEnvironments().map((env) => env.id),
//   default: defaultEnvironment()
// })

// .option('testAnnounceLocalAddresses', {
//   hidden: true,
//   boolean: true,
//   describe: 'For testing local testnets. Announce local addresses [env: HOPRD_TEST_ANNOUNCE_LOCAL_ADDRESSES]',
//   default: false
// })
// .option('testPreferLocalAddresses', {
//   hidden: true,
//   boolean: true,
//   describe: 'For testing local testnets. Prefer local peers to remote [env: HOPRD_TEST_PREFER_LOCAL_ADDRESSES]',
//   default: false
// })
// .option('testUseWeakCrypto', {
//   hidden: true,
//   boolean: true,
//   describe: 'weaker crypto for faster node startup [env: HOPRD_TEST_USE_WEAK_CRYPTO]',
//   default: false
// })
// .option('testNoAuthentication', {
//   hidden: true,
//   boolean: true,
//   describe: 'no remote authentication for easier testing [env: HOPRD_TEST_NO_AUTHENTICATION]',
//   default: undefined
// })
// .option('testNoDirectConnections', {
//   hidden: true,
//   boolean: true,
//   describe:
//     'NAT traversal testing: prevent nodes from establishing direct TCP connections [env: HOPRD_TEST_NO_DIRECT_CONNECTIONS]',
//   default: false
// })
// .option('testNoWebRTCUpgrade', {
//   hidden: true,
//   boolean: true,
//   describe:
//     'NAT traversal testing: prevent nodes from establishing direct TCP connections [env: HOPRD_TEST_NO_WEB_RTC_UPGRADE]',
//   default: false
// })
// .option('testNoUPNP', {
//   hidden: true,
//   boolean: true,
//   describe:
//     'NAT traversal testing: disable automatic detection of external IP address using UPNP [env: HOPRD_TEST_NO_UPNP]',
//   default: false
// })
// .option('heartbeatInterval', {
//   number: true,
//   describe:
//     'Interval in milliseconds in which the availability of other nodes get measured [env: HOPRD_HEARTBEAT_INTERVAL]',
//   default: HEARTBEAT_INTERVAL
// })
// .option('heartbeatThreshold', {
//   number: true,
//   describe:
//     "Timeframe in milliseconds after which a heartbeat to another peer is performed, if it hasn't been seen since [env: HOPRD_HEARTBEAT_THRESHOLD]",
//   default: HEARTBEAT_THRESHOLD
// })
// .option('heartbeatVariance', {
//   number: true,
//   describe: 'Upper bound for variance applied to heartbeat interval in milliseconds [env: HOPRD_HEARTBEAT_VARIANCE]',
//   default: HEARTBEAT_INTERVAL_VARIANCE
// })
// .option('networkQualityThreshold', {
//   number: true,
//   describe: 'Miniumum quality of a peer connection to be considered usable [env: HOPRD_NETWORK_QUALITY_THRESHOLD]',
//   default: NETWORK_QUALITY_THRESHOLD
// })
// .option('onChainConfirmations', {
//   number: true,
//   describe: 'Number of confirmations required for on-chain transactions [env: HOPRD_ON_CHAIN_CONFIRMATIONS]',
//   default: CONFIRMATIONS
// })

// {
//     "environments": {
//       "hardhat-localhost": {
//         "network_id": "hardhat",
//         "environment_type": "development",
//         "version_range": "*",
//         "channel_contract_deploy_block": 0,
//         "token_contract_address": "${HARDHAT_TOKEN_CONTACT_ADDRESS}",
//         "channels_contract_address": "${HARDHAT_CHANNELS_CONTACT_ADDRESS}",
//         "minted_token_receiver_address": "0x2402da10A6172ED018AEEa22CA60EDe1F766655C",
//         "xhopr_contract_address": "${HARDHAT_XHOPR_CONTACT_ADDRESS}",
//         "boost_contract_address": "${HARDHAT_BOOST_CONTACT_ADDRESS}",
//         "stake_contract_address": "${HARDHAT_STAKE_CONTACT_ADDRESS}",
//         "network_registry_proxy_contract_address": "${HARDHAT_REGISTRY_PROXY_CONTACT_ADDRESS}",
//         "network_registry_contract_address": "${HARDHAT_REGISTRY_CONTACT_ADDRESS}",
//         "tags": []
//       },
//       "hardhat-localhost2": {
//         "network_id": "hardhat",
//         "environment_type": "development",
//         "version_range": "*",
//         "channel_contract_deploy_block": 0,
//         "token_contract_address": "${HARDHAT_TOKEN_CONTACT_ADDRESS}",
//         "channels_contract_address": "${HARDHAT_CHANNELS_CONTACT_ADDRESS}",
//         "minted_token_receiver_address": "0x2402da10A6172ED018AEEa22CA60EDe1F766655C",
//         "xhopr_contract_address": "${HARDHAT_XHOPR_CONTACT_ADDRESS}",
//         "boost_contract_address": "${HARDHAT_BOOST_CONTACT_ADDRESS}",
//         "stake_contract_address": "${HARDHAT_STAKE_CONTACT_ADDRESS}",
//         "network_registry_proxy_contract_address": "${HARDHAT_REGISTRY_PROXY_CONTACT_ADDRESS}",
//         "network_registry_contract_address": "${HARDHAT_REGISTRY_CONTACT_ADDRESS}",
//         "tags": []
//       },
//       "master-goerli": {
//         "network_id": "goerli",
//         "environment_type": "staging",
//         "version_range": "*",
//         "channel_contract_deploy_block": 0,
//         "token_contract_address": "0xa3C8f4044b30Fb3071F5b3b02913DE524F1041dc",
//         "channels_contract_address": "0xc81E2Bf738f8018202c8Bd9dA85a12D5D7291d08",
//         "minted_token_receiver_address": "0x8C9877a1279192448cAbeC9e8C4697b159cF645e",
//         "xhopr_contract_address": "0xe8aD2ac347dA7549Aaca8F5B1c5Bf979d85bC78F",
//         "boost_contract_address": "0xDA335D985710b80e5BfC697C6Fba0A906Dd4a1CE",
//         "stake_contract_address": "0xab231873246daaff05c99b3682ea649e6C80AB68",
//         "network_registry_proxy_contract_address": "0xb311239b46feCde9D68d70Ae4bA8733C3dBC7023",
//         "network_registry_contract_address": "0xf3374666487A58aa424BF9dB9bCf74250393a757",
//         "tags": ["etherscan"]
//       },
//       "debug-goerli": {
//         "network_id": "goerli",
//         "environment_type": "staging",
//         "version_range": "*",
//         "channel_contract_deploy_block": 0,
//         "token_contract_address": "0xa3C8f4044b30Fb3071F5b3b02913DE524F1041dc",
//         "channels_contract_address": "0x43FBe6a5F571B7288Ce58F3b03a48Fcaa11fE58c",
//         "minted_token_receiver_address": "0x8C9877a1279192448cAbeC9e8C4697b159cF645e",
//         "stake_contract_address": "0xA681e2Bd553648282322e3d11a7dC96344FCBcdA",
//         "network_registry_contract_address": "0x57e74767075471B1B87C7d34968a5c91a6B6FEB4",
//         "xhopr_contract_address": "0x552aBf0EBCd6B6162519132A831C181f87e46874",
//         "boost_contract_address": "0x1c5Fe2Ac0D6Ec7a4213004CCB4aC35A71aF5aCd9",
//         "network_registry_proxy_contract_address": "0xe08280669faDA942d550c0bcAA9Bd6484a19abd8",
//         "tags": ["development"]
//       },
//       "tuttlingen": {
//         "environment_type": "production",
//         "network_id": "xdai",
//         "version_range": "1.83",
//         "channel_contract_deploy_block": 0,
//         "token_contract_address": "0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1",
//         "channels_contract_address": "0xd2229d5d54bE8ABC9A2b2d5cFdEd22B48FB8ce2c",
//         "stake_contract_address": "0x2cDD13ddB0346E0F620C8E5826Da5d7230341c6E",
//         "network_registry_contract_address": "${HARDHAT_REGISTRY_CONTACT_ADDRESS}",
//         "tags": []
//       },
//       "prague": {
//         "network_id": "xdai",
//         "environment_type": "production",
//         "version_range": "1.84",
//         "channel_contract_deploy_block": 0,
//         "token_contract_address": "0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1",
//         "channels_contract_address": "0x4f98F01cb02083eb5457CA0DDC6B37073ec5388a",
//         "stake_contract_address": "0x2cDD13ddB0346E0F620C8E5826Da5d7230341c6E",
//         "network_registry_contract_address": "${HARDHAT_REGISTRY_CONTACT_ADDRESS}",
//         "tags": []
//       },
//       "budapest": {
//         "network_id": "xdai",
//         "environment_type": "production",
//         "version_range": "1.85",
//         "channel_contract_deploy_block": 0,
//         "token_contract_address": "0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1",
//         "channels_contract_address": "0xEee8AB66b7169b3f9024676165fB1D2a25E472c5",
//         "stake_contract_address": "0x2cDD13ddB0346E0F620C8E5826Da5d7230341c6E",
//         "network_registry_contract_address": "${HARDHAT_REGISTRY_CONTACT_ADDRESS}",
//         "tags": []
//       },
//       "athens": {
//         "network_id": "xdai",
//         "environment_type": "production",
//         "version_range": "1.86",
//         "channel_contract_deploy_block": 0,
//         "token_contract_address": "0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1",
//         "channels_contract_address": "0xD2F008718EEdD7aF7E9a466F5D68bb77D03B8F7A",
//         "stake_contract_address": "0x2cDD13ddB0346E0F620C8E5826Da5d7230341c6E",
//         "network_registry_contract_address": "${HARDHAT_REGISTRY_CONTACT_ADDRESS}",
//         "tags": []
//       },
//       "lisbon": {
//         "network_id": "xdai",
//         "environment_type": "production",
//         "version_range": "1.87",
//         "channel_contract_deploy_block": 0,
//         "token_contract_address": "0x978D91ddFdb0c6eaCC22A258fE498957a79c5F4C",
//         "channels_contract_address": "0x1753C9eE656c54f443ce2Fe7248076f9bA4eC100",
//         "stake_contract_address": "0x2cDD13ddB0346E0F620C8E5826Da5d7230341c6E",
//         "network_registry_contract_address": "${HARDHAT_REGISTRY_CONTACT_ADDRESS}",
//         "tags": []
//       },
//       "ouagadougou": {
//         "network_id": "xdai",
//         "environment_type": "production",
//         "version_range": "1.88",
//         "channel_contract_deploy_block": 0,
//         "token_contract_address": "0x978D91ddFdb0c6eaCC22A258fE498957a79c5F4C",
//         "channels_contract_address": "0x0BB1Ff7A9b3f2c87267626630aa0195cAE3ed5E3",
//         "xhopr_contract_address": "0xD057604A14982FE8D88c5fC25Aac3267eA142a08",
//         "boost_contract_address": "0x43d13D7B83607F14335cF2cB75E87dA369D056c7",
//         "stake_contract_address": "0xae933331ef0bE122f9499512d3ed4Fa3896DCf20",
//         "network_registry_contract_address": "${HARDHAT_REGISTRY_CONTACT_ADDRESS}",
//         "tags": []
//       },
//       "paleochora": {
//         "network_id": "xdai",
//         "environment_type": "production",
//         "version_range": "1.89",
//         "channel_contract_deploy_block": 0,
//         "token_contract_address": "0x978D91ddFdb0c6eaCC22A258fE498957a79c5F4C",
//         "channels_contract_address": "0xCe7209FD81ED129C2ea8369715c168419e4148Ef",
//         "xhopr_contract_address": "0xD057604A14982FE8D88c5fC25Aac3267eA142a08",
//         "boost_contract_address": "0x43d13D7B83607F14335cF2cB75E87dA369D056c7",
//         "stake_contract_address": "0x5Bb7e435aDa333A6714e27962e4Bb6aFDE1cECd4",
//         "network_registry_contract_address": "0xf0D4D48866C4F9665212B29A64c14A1f0FEDFD4c",
//         "network_registry_proxy_contract_address": "0x15e068Ef1f76319b1848b1fcB3e49D68724AEE07",
//         "tags": ["etherscan"]
//       },
//       "monte_rosa": {
//         "network_id": "xdai",
//         "environment_type": "production",
//         "version_range": "1.89||1.90||1.91",
//         "channel_contract_deploy_block": 0,
//         "token_contract_address": "0x66225dE86Cac02b32f34992eb3410F59DE416698",
//         "channels_contract_address": "0xFaBeE463f31E39eC8952bBfB4490C41103bf573e",
//         "xhopr_contract_address": "0xD057604A14982FE8D88c5fC25Aac3267eA142a08",
//         "boost_contract_address": "0x43d13D7B83607F14335cF2cB75E87dA369D056c7",
//         "stake_contract_address": "0xd80fbbfe9d057254d80eebb49f17aca66a238e2d",
//         "network_registry_contract_address": "0x819E6a81e1e3f96CF1ac9200477C2d09c676959D",
//         "network_registry_proxy_contract_address": "0x1C0C4EFb9a2ccE18d66eaFFc585876d8CA768013",
//         "tags": ["etherscan"]
//       }
//     },
//     "networks": {
//       "hardhat": {
//         "description": "Hardhat is an Ethereum development environment",
//         "chain_id": 1,
//         "live": false,
//         "max_fee_per_gas": "1 gwei",
//         "max_priority_fee_per_gas": "0.2 gwei",
//         "default_provider": "http://127.0.0.1:8545/",
//         "native_token_name": "ETH",
//         "hopr_token_name": "wxHOPR"
//       },
//       "xdai": {
//         "description": "The xDai chain is a stable payments EVM (Ethereum Virtual Machine) blockchain designed for fast and inexpensive transactions",
//         "chain_id": 100,
//         "live": true,
//         "max_fee_per_gas": "10 gwei",
//         "max_priority_fee_per_gas": "2 gwei",
//         "default_provider": "https://provider-proxy.hoprnet.workers.dev/xdai_mainnet",
//         "native_token_name": "xDAI",
//         "hopr_token_name": "wxHOPR"
//       },
//       "goerli": {
//         "description": "Görli Testnet is the first proof-of-authority cross-client testnet, synching Parity Ethereum, Geth, Nethermind, Hyperledger Besu (formerly Pantheon), and EthereumJS",
//         "chain_id": 5,
//         "live": true,
//         "max_fee_per_gas": "200 gwei",
//         "max_priority_fee_per_gas": "2 gwei",
//         "default_provider": "https://provider-proxy.hoprnet.workers.dev/eth_goerli",
//         "etherscan_api_url": "http://api-goerli.etherscan.io/api",
//         "native_token_name": "gETH",
//         "hopr_token_name": "wxHOPR"
//       }
//     }
//   }
