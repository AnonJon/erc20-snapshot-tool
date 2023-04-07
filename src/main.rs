use anyhow::Error;
use ethers::{
    core::types::{Address, Filter, H160, U256},
    providers::{Http, Middleware, Provider},
};
use eyre::Result;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;

mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let rpc_url = env::var("ETHEREUM_RPC_URL").expect("ETHEREUM_RPC_URL must be set");
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let client = Arc::new(provider);

    let config = utils::load_config().expect("Error loading config file");

    let mut token_holders: Vec<Address> = vec![];
    let filter = Filter::new()
        .address(config.contract_address)
        .event("Transfer(address,address,uint256)")
        .from_block(config.contract_creation_block)
        .to_block(config.block_height);
    let logs = client.get_logs(&filter).await?;

    for log in logs.iter() {
        let from = Address::from(log.topics[1]);
        let to = Address::from(log.topics[2]);
        if !token_holders.contains(&from) {
            token_holders.push(from);
        }
        if !token_holders.contains(&to) {
            token_holders.push(to);
        }
    }

    println!("Done capturing token holders");

    for (i, token) in config.token_addresses.iter().enumerate() {
        match write_balances(
            *token,
            &token_holders,
            format!("{}-balances.json", config.token_names[i]),
            config.block_height,
        )
        .await
        {
            Ok(()) => println!("Done {}", config.token_names[i]),
            Err(err) => eprintln!("Error: {:?}", err),
        }
    }

    Ok(())
}

async fn write_balances(
    address: Address,
    holders: &Vec<H160>,
    file_name: String,
    block: u64,
) -> Result<(), Error> {
    println!("Writing balances for {}", file_name);

    let mut token_holders: HashMap<H160, String> = HashMap::new();
    let mut new_balances: Vec<U256> = vec![];
    match utils::get_erc20_balance_at_block(format!("{:#x}", address), holders, block).await {
        Ok(balance) => {
            new_balances = balance;
        }
        Err(err) => {
            println!("Error: {:?}", err);
        }
    }
    for (i, h) in holders.iter().enumerate() {
        if new_balances[i].as_u128() == 0 {
            continue;
        }
        token_holders.insert(*h, new_balances[i].to_string());
    }

    let json_data = serde_json::to_string(&token_holders)?;
    let mut file = File::create(file_name)?;
    file.write_all(json_data.as_bytes())?;

    Ok(())
}
