use ethereum_snapshot::utils::{load_config, process_blocks, write_balances};
use ethers::{
    core::types::Address,
    providers::{Http, Provider},
};
use eyre::Result;
use futures::future::join_all;
use std::collections::HashSet;
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let config = load_config().expect("Error loading config file");
    let rpc_url = &env::var("ETHEREUM_RPC_URL").expect("ETHEREUM_RPC_URL must be set");
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let client = Arc::new(provider);

    let mut from_block = config.contract_creation_block;
    let mut tasks = Vec::new();

    while from_block <= config.block_height {
        let to_block = (from_block + config.batch_size).min(config.block_height);

        let client_clone = client.clone();
        let task = tokio::spawn(async move {
            process_blocks(client_clone, from_block, to_block, config.contract_address).await
        });

        tasks.push(task);

        from_block = to_block + 1;
    }

    let results = join_all(tasks).await;
    let mut token_holders: HashSet<Address> = HashSet::new();

    for result in results {
        match result {
            Ok(Ok(token_set)) => {
                for holder in token_set {
                    token_holders.insert(holder);
                }
            }
            Ok(Err(err)) => {
                eprintln!("Error: {:?}", err);
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
            }
        }
    }

    println!("Token holders count: {}", token_holders.len());

    // capture second token
    // from_block = config.contract_creation_block;
    // while from_block <= config.block_height {
    //     let to_block = (from_block + config.batch_size).min(config.block_height);
    //     println!("Fetching logs from block {} to {}", from_block, to_block);
    //     let address = "0xd27b7d42d24d8f7c1cf5c46ccd3b986c396fde17".parse::<Address>()?;
    //     let filter = Filter::new()
    //         .address(address)
    //         .event("Transfer(address,address,uint256)")
    //         .from_block(from_block)
    //         .to_block(to_block);

    //     let logs = client.get_logs(&filter).await?;

    //     for log in logs.iter() {
    //         let from = Address::from(log.topics[1]);
    //         let to = Address::from(log.topics[2]);
    //         token_holders.insert(from);
    //         token_holders.insert(to);
    //     }

    //     from_block = to_block + 1;
    // }

    println!("Done capturing token holders");
    let token_holders: Vec<Address> = token_holders.into_iter().collect();

    for (i, token) in config.token_addresses.iter().enumerate() {
        match write_balances(
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
