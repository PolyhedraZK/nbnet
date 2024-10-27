# Proof-of-Stake

### Deposit

Usage:

```shell
# nb deposit
Manage deposit operations

Usage: nb deposit --rpc-endpoint <RPC_ENDPOINT> \
            --deposit-contract-addr <DEPOSIT_CONTRACT_ADDR> \
            --deposit-data-json-path <DEPOSIT_DATA_JSON_PATH> \
            --wallet-signkey-path <WALLET_SIGNKEY_PATH>

Options:
  -R, --rpc-endpoint <RPC_ENDPOINT>
          EL rpc endpoint, e.g., 'http://localhost:8545'
  -C, --deposit-contract-addr <DEPOSIT_CONTRACT_ADDR>
          Deposit contract address on the EL side,
          e.g., '0x4242424242424242424242424242424242424242'
  -D, --deposit-data-json-path <DEPOSIT_DATA_JSON_PATH>
          Deposit data in the standard JSON format,
          produced by the ETH official 'staking-deposit-cli' tool
  -W, --wallet-signkey-path <WALLET_SIGNKEY_PATH>
          The deposit principal will be deducted from this wallet
```

Workflow Example:
1. Get the [staking-deposit-cli](https://github.com/ethereum/staking-deposit-cli) tool, and then `deposit new-mnemonic`
    - The 'deposit_data-xxx.json' file
        - Send it to the on-chain deposit contract
    - The 'keystore-m_xxx.json' file
        - Used by the `lighthouse validator-manager import`
    - Another way is to use the `lighthouse validator-manager create`
        - So there is no need to prepare an extra tool
        - You need to prepare a mnemonic in advance
            - E.g., create a new one with `nb new-mnemonic`
2. `CONTRACT='0x4242424242424242424242424242424242424242'`
3. `KEY='/PATH/TO/YOUR/PRIVATE/KEY'`
4. `RPC='http://localhost:8545'`
5. `nb deposit -C $CONTRACT -D deposit_data-xxx.json -W $KEY -R $RPC`

Cmdline Example:
```shell
lighthouse validator-manager create \
    --mnemonic-path ./mnemonic \
    --testnet-dir genesis_dir \
    --first-index 0 \
    --count 2 \
    --eth1-withdrawal-address 0x8943545177806ED17B9F23F0a21ee5948eCaa776 \
    --output-path .

lighthouse validator-manager import \
    --datadir cl/vc \
    --testnet-dir genesis_dir \
    --validators-file ./validators.json \
    --vc-url http://localhost:5062 \
    --vc-token cl/vc/validators/api-token.txt
```

### Refs

- https://lighthouse-book.sigmaprime.io/validator-management.html
- https://github.com/ethereum/staking-deposit-cli
- https://ethereum.github.io/beacon-APIs
