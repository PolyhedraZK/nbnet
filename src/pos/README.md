# Proof-of-Stake

### Deposit

Usage:

```
# `nb deposit -h`
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

```
# nb dev/ddev deposit -h
Proof-of-Stake, deposit

Usage: nb dev/ddev {deposit|-d} [OPTIONS] --nodes <NODES>

Options:
  -e, --env-name <ENV_NAME>

  -N, --nodes <NODES>
          Comma separated NodeID[s], '3', '3,2,1', etc.
          if set to 'all', then deposit on all non-fuhrer nodes
  -n, --num-per-node <NUM_PER_NODE>
          How many validators to deposit on each node [default: 1]
  -K, --wallet-seckey-path <WALLET_SECKEY_PATH>
          The path of your private key(for gas and the deposit principal),
          the first premint account will be used if not provided
  -A, --withdraw-0x01-addr <WITHDRAW_0X01_ADDR>
          An account used to receive the funds after validators exit,
          the address coresponding to `wallet-seckey` will be used if not provided
  -x, --async-wait
          If set, return immediately after the transaction is sent,
          or wait until the deposit transaction is confirmed on chain
```

```
# nb dev/ddev validator-exit -h
Proof-of-Stake, exit all validators on the target node[s]

Usage: nb dev/ddev {validator-exit|-D} [OPTIONS] --nodes <NODES>

Options:
  -e, --env-name <ENV_NAME>
  -N, --nodes <NODES>        Comma separated NodeID[s], '3', '3,2,1', etc.
                             if set to 'all', then exit all validators of all non-fuhrer nodes
  -x, --async-wait           If set, return immediately after the request is sent,
                             or wait until the exit request is confirmed on chain
```

Workflow:
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

Example:

```shell
# Create a devnet with 1 non-fuhrer node
nb dev create -n 1

NODE_HOME=$(nb dev | jq '.meta.nodes."1".node_home' | tr -d '"')

TESTNET_DIR="${NODE_HOME}/genesis"

CFG_PATH="${TESTNET_DIR}/config.yaml"
CONTRACT=$(grep -Po '(?<=DEPOSIT_CONTRACT_ADDRESS:)\s*[\w]+' ${CFG_PATH})

VC_DATA_DIR="${NODE_HOME}/cl/vc"
VC_API_TOKEN="${VC_DATA_DIR}/validators/api-token.txt"

EL_RPC_PORT=$(nb dev | jq '.meta.nodes."1".ports.el_rpc')
EL_RPC_ENDPOINT="http://localhost:${EL_RPC_PORT}"

BN_RPC_PORT=$(nb dev | jq '.meta.nodes."1".ports.cl_bn_rpc')
BN_RPC_ENDPOINT="http://localhost:${BN_RPC_PORT}"

VC_RPC_PORT=$(nb dev | jq '.meta.nodes."1".ports.cl_vc_rpc')
VC_RPC_ENDPOINT="http://localhost:${VC_RPC_PORT}"

wallet=$(nb dev | jq '[.meta.premined_accounts][0]')
WALLET_ADDR=$(echo ${wallet} | jq 'keys[0]' | tr -d '"')
WALLET_KEY=$(echo ${wallet} | jq '[.[]][0].secretKey' | tr -d '"')
WALLET_KEY_PATH="wallet_key"
printf "${WALLET_KEY}" >${WALLET_KEY_PATH}

nb new-mnemonic | sed '/^$/d' >mnemonic

lighthouse validator-manager create \
    --testnet-dir ${TESTNET_DIR} \
    --datadir ${VC_DATA_DIR} \
    --mnemonic-path ./mnemonic \
    --first-index 0 \
    --count 2 \
    --eth1-withdrawal-address ${WALLET_ADDR} \
    --suggested-fee-recipient ${WALLET_ADDR} \
    --output-path .

lighthouse validator-manager import \
    --testnet-dir ${TESTNET_DIR} \
    --datadir ${VC_DATA_DIR} \
    --validators-file validators.json \
    --vc-url ${VC_RPC_ENDPOINT} \
    --vc-token ${VC_API_TOKEN}

nb deposit -C ${CONTRACT} -D deposits.json -W ${WALLET_KEY_PATH} -R ${EL_RPC_ENDPOINT}

# check status
curl "${BN_RPC_ENDPOINT}/lighthouse/eth1/deposit_cache" -H "accept: application/json" | jq

# check status
for pubkey in $(cat deposits.json | jq '.[].pubkey' | tr -d '"'); do
    curl "${BN_RPC_ENDPOINT}/eth/v1/beacon/states/head/validators/${pubkey}" \
        -H "accept: application/json" | jq
done
```

Embed Example:

```shell
# Create a devnet with 2 non-fuhrer nodes
nb dev create -n 2

# Deposit to all the non-fuhrer nodes
# Check `nb dev/ddev deposit -h` for detail
nb dev deposit -N all
```

### Refs

- https://lighthouse-book.sigmaprime.io/validator-management.html
- https://github.com/ethereum/staking-deposit-cli
- https://github.com/ethereum/staking-launchpad
- https://github.com/ChorusOne/eth-staking-smith
- https://ethereum.github.io/beacon-APIs
