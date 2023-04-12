use ethers::{
    core::types::{Address, Filter},
    providers::{Http, Middleware, Provider},
};
use eyre::Result;
use std::collections::HashSet;
use std::env;
use std::sync::Arc;

mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    let config = utils::load_config().expect("Error loading config file");
    let rpc_url = &env::var("ETHEREUM_RPC_URL").expect("ETHEREUM_RPC_URL must be set");
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let client = Arc::new(provider);

    let mut token_holders: HashSet<Address> = HashSet::new();
    let mut from_block = config.contract_creation_block;
    while from_block <= config.block_height {
        let to_block = (from_block + config.batch_size).min(config.block_height);
        println!("Fetching logs from block {} to {}", from_block, to_block);

        let filter = Filter::new()
            .address(config.contract_address)
            .event("Transfer(address,address,uint256)")
            .from_block(from_block)
            .to_block(to_block);

        let logs = client.get_logs(&filter).await?;

        for log in logs.iter() {
            let from = Address::from(log.topics[1]);
            let to = Address::from(log.topics[2]);
            token_holders.insert(from);
            token_holders.insert(to);
        }

        from_block = to_block + 1;
    }

    println!("Done capturing token holders");
    let token_holders: Vec<Address> = token_holders.into_iter().collect();
    for (i, token) in config.token_addresses.iter().enumerate() {
        match utils::write_balances(
            *token,
            &token_holders,
            format!("{}-balances.json", config.token_names[i]),
            config.block_height,
            &rpc_url,
        )
        .await
        {
            Ok(()) => println!("Done {}", config.token_names[i]),
            Err(err) => eprintln!("Error: {:?}", err),
        }
    }

    Ok(())
}
