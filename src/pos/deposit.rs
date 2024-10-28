use alloy::{
    contract::Interface,
    dyn_abi::DynSolValue,
    network::{EthereumWallet, TransactionBuilder},
    primitives::{
        hex::{self, FromHex},
        utils::Unit,
        Address, U256,
    },
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::{k256::ecdsa::SigningKey, local::PrivateKeySigner},
    transports::http::reqwest::Url,
};
use ruc::*;
use serde::{Deserialize, Serialize};
use std::fs;

type DepositData = Vec<DepositEntry>;

const HASH_LEN: usize = 32;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct DepositEntry {
    #[serde(with = "hex")]
    pubkey: Vec<u8>,

    #[serde(with = "hex")]
    withdrawal_credentials: Vec<u8>,

    #[serde(with = "hex")]
    signature: Vec<u8>,

    #[serde(with = "hex")]
    deposit_data_root: [u8; HASH_LEN],

    #[serde(with = "hex")]
    deposit_message_root: Vec<u8>,

    // NOTE: this value is in `Gwei`, not `wei`
    amount: U256,

    network_name: String,

    fork_version: String,

    deposit_cli_version: String,
}

pub async fn deposit(
    rpc_endpoint: &str,
    deposit_contract_addr: &str,
    deposit_data_json_path: &str,
    wallet_signkey_path: &str,
) -> Result<()> {
    let signkey = fs::read_to_string(wallet_signkey_path).c(d!())?;
    let deposit_data = fs::read_to_string(deposit_data_json_path).c(d!())?;
    do_deposit(rpc_endpoint, deposit_contract_addr, &signkey, &deposit_data)
        .await
        .c(d!())
}

// For inner usage
pub async fn do_deposit(
    rpc_endpoint: &str,
    deposit_contract_addr: &str,
    deposit_data_json: &str,
    wallet_signkey: &str,
) -> Result<()> {
    let signkey = hex::decode(wallet_signkey.trim()).c(d!())?;
    let signkey = SigningKey::from_slice(&signkey).c(d!())?;

    let wallet_addr = Address::from_private_key(&signkey);
    let contract_addr = Address::from_hex(deposit_contract_addr).c(d!())?;

    let signer = PrivateKeySigner::from_signing_key(signkey);
    let wallet = EthereumWallet::from(signer);

    let url = rpc_endpoint.parse::<Url>().c(d!())?;
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(url);

    let mut deposit_data =
        serde_json::from_str::<DepositData>(deposit_data_json).c(d!())?;

    for dd in deposit_data.iter_mut() {
        // convert 'Gwei' to 'wei'
        dd.amount = dd.amount.checked_mul(Unit::GWEI.wei()).c(d!())?;
    }

    let balance = provider.get_balance(wallet_addr).await.c(d!())?;
    let balance_guard = deposit_data.iter().map(|d| d.amount).sum::<U256>();
    if balance <= balance_guard {
        return Err(eg!(
            "Insufficient balance, should bigger than: {} wei, owned: {} wei",
            balance_guard.to_string(),
            balance.to_string()
        ));
    }

    let chain_id = provider.get_chain_id().await.c(d!())?;
    let gas_price = provider.get_gas_price().await.c(d!())?;
    let nonce = provider.get_transaction_count(wallet_addr).await.c(d!())?;

    let abi = include_bytes!("../../config/deposit/abi.json");
    let interface = serde_json::from_slice(abi).map(Interface::new).c(d!())?;

    for (idx, dd) in deposit_data.into_iter().enumerate() {
        let tx_input = interface
            .encode_input(
                "deposit",
                &[
                    dd.pubkey.into(),
                    dd.withdrawal_credentials.into(),
                    dd.signature.into(),
                    DynSolValue::FixedBytes(dd.deposit_data_root.into(), HASH_LEN),
                ],
            )
            .c(d!())?;

        let tx_req = TransactionRequest::default()
            .with_chain_id(chain_id)
            .with_gas_price(gas_price)
            .with_nonce(nonce + idx as u64)
            .with_from(wallet_addr)
            .with_to(contract_addr)
            .with_value(dd.amount)
            .with_input(tx_input);

        let gas_limit = provider.estimate_gas(&tx_req).await.c(d!())?;

        let tx_envelope = tx_req
            .with_gas_limit(gas_limit)
            .build(&wallet)
            .await
            .c(d!())?;

        let receipt = provider
            .send_tx_envelope(tx_envelope)
            .await
            .c(d!())?
            .get_receipt()
            .await
            .c(d!())?;

        if receipt.status() {
            println!(
                "Transaction: {}, In Block: {}({})",
                receipt.transaction_hash,
                receipt
                    .block_number
                    .map(|i| i.to_string())
                    .unwrap_or("null".to_owned()),
                receipt
                    .block_hash
                    .map(|i| i.to_string())
                    .unwrap_or("null".to_owned()),
            );
        } else {
            dbg!(receipt);
            pnk!(Err(eg!()));
        }
    }

    Ok(())
}
