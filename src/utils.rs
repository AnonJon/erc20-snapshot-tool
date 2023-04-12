use anyhow::Error;
use ethers::core::types::{Address, H160, U256};
use serde::Deserialize;
use serde_json;
use serde_json::json;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
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
    #[serde(rename = "batchSize")]
    pub batch_size: u64,
}

pub async fn write_balances(
    address: Address,
    holders: &Vec<H160>,
    file_name: String,
    block: u64,
    rpc_url: &str,
) -> Result<(), Error> {
    println!("Writing balances for {}", file_name);

    let mut token_holders: HashMap<H160, String> = HashMap::new();
    let mut new_balances: Vec<U256> = vec![];
    match get_erc20_balance_at_block(format!("{:#x}", address), holders, block, rpc_url.clone())
        .await
    {
        Ok(balance) => {
            new_balances = balance;
        }
        Err(err) => {
            eprintln!("ERROR: {:?}", err);
        }
    }
    for (holder, balance) in holders.iter().zip(new_balances.iter()) {
        if balance.as_u128() == 0 {
            continue;
        }
        token_holders.insert(*holder, balance.to_string());
    }

    let json_data = serde_json::to_string(&token_holders)?;
    let mut file = File::create(file_name)?;
    file.write_all(json_data.as_bytes())?;

    Ok(())
}

async fn get_erc20_balance_at_block(
    token_address: String,
    holder_addresses: &Vec<H160>,
    block: u64,
    rpc_url: &str,
) -> Result<Vec<U256>, BalanceError> {
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
