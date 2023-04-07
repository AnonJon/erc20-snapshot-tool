use ethers::core::types::{Address, H160, U256};
use serde::Deserialize;
use serde_json;
use serde_json::json;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;

#[derive(Debug)]
pub enum BalanceError {
    Reqwest(reqwest::Error),
    ParsingError,
}
impl From<reqwest::Error> for BalanceError {
    fn from(err: reqwest::Error) -> Self {
        BalanceError::Reqwest(err)
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "contractCreation")]
    pub contract_creation_block: u64,
    #[serde(rename = "contractAddress")]
    pub contract_address: Address,
    #[serde(rename = "blockHeight")]
    pub block_height: u64,
    #[serde(rename = "tokenAddresses")]
    pub token_addresses: Vec<Address>,
    #[serde(rename = "tokenNames")]
    pub token_names: Vec<String>,
}

pub async fn get_erc20_balance_at_block(
    token_address: String,
    holder_addresses: &Vec<H160>,
    block: u64,
) -> Result<Vec<U256>, BalanceError> {
    let rpc_url = env::var("ETHEREUM_RPC_URL").expect("ETHEREUM_RPC_URL must be set");
    let string_holders: Vec<String> = holder_addresses
        .iter()
        .map(|h| format!("{:#x}", h))
        .collect();

    let requests: Vec<serde_json::Value> = string_holders
        .iter()
        .enumerate()
        .map(|(i, holder_address)| {
            json!({
                "jsonrpc": "2.0",
                "id": i + 1,
                "method": "eth_call",
                "params": [
                    {
                        "to": token_address,
                        "data": format!(
                            "0x70a08231000000000000000000000000{}",
                            &holder_address[2..]
                        )
                    },
                    format!("0x{:x}", block)
                ]
            })
        })
        .collect();

    let client = reqwest::Client::new();
    let response: Vec<serde_json::Value> = client
        .post(rpc_url)
        .json(&requests)
        .send()
        .await?
        .json()
        .await?;

    let balances: Result<Vec<U256>, BalanceError> = response
        .into_iter()
        .map(|r| {
            let balance_hex = r["result"].as_str().ok_or(BalanceError::ParsingError)?;
            U256::from_str(balance_hex).map_err(|_| BalanceError::ParsingError)
        })
        .collect();

    balances
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let file = File::open("./config.json")?;
    let reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(reader)?;
    if config.token_addresses.len() != config.token_names.len() {
        return Err("Token addresses and names must be the same length".into());
    }
    Ok(config)
}
