# Ethereum ERC20 Snapshot Tool

A tool to capture token balances of an ERC20 contract at a specific block height.

## Setup

Edit `config.json` to update the block heights and add token addresses.

```json
{
  "contractCreation": 16201062, // block where contract was created
  "contractAddress": "0xA95C5ebB86E0dE73B4fB8c47A45B792CFeA28C23",
  "blockHeight": 16970099, // when to take snapshot
  "tokenAddresses": [
    // addresses of token and any other relative tokens to look at ie. staked assets/lp tokens
    "0xA95C5ebB86E0dE73B4fB8c47A45B792CFeA28C23",
    "0xAEF186611EC96427d161107fFE14bba8aA1C2284",
    "0xd27b7d42d24d8f7c1cf5c46ccd3b986c396fde17"
  ],
  "tokenNames": ["sdl", "stsdl", "sdl-slp"] // names to associate tokens to which are also used for json creation
}
```
