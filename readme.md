# Ethereum ERC20 Snapshot Tool

A tool to capture token balances of an ERC20 contract at a specific block height.

## Setup

Add rpc url as env variable `ETHEREUM_RPC_URL`

Edit `config.json` to update the block heights and add token addresses.

```json
{
  "contractCreation": 16201062, // block where contract was created
  "contractAddress": "0x514910771AF9Ca656af840dff83E8264EcF986CA",
  "blockHeight": 16970099, // when to take snapshot
  "tokenAddresses": [
    // addresses of token and any other relative tokens to look at ie. staked assets/lp tokens
    "0x514910771AF9Ca656af840dff83E8264EcF986CA",
    "0xb8b295df2cd735b15BE5Eb419517Aa626fc43cD5"
  ],
  "tokenNames": ["link", "stLink"] // names to associate tokens which are also used for json creation
}
```

## Run

```bash
cargo run
```
