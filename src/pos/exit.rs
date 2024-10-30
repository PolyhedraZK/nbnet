use regex::Regex;
use ruc::{algo::rand::rand_jwt, cmd::exec_output, *};
use std::{fs, sync::LazyLock};

static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r".*(0x\w+)").unwrap());

/// 1. Recover voting keystore
/// 2. Send exit request to a beacon RPC endpoint
pub fn exit(
    beacon_endpoint: &str,
    genesis_dir: &str,
    keystore_path: &str,
    password_path: &str,
    async_wait: bool,
) -> Result<()> {
    let mut cmd = format!(
        r#"
        lighthouse account validator exit \
            --beacon-node {beacon_endpoint} \
            --testnet-dir {genesis_dir} \
            --keystore {keystore_path} \
            --password-file {password_path} \
            --no-confirmation \
        "#
    );

    if async_wait {
        cmd.push_str(" --no-wait");
    }

    exec_output(&cmd).c(d!()).map(|s| {
        println!("{s}");
    })
}

pub fn exit_by_mnemonic(
    beacon_endpoint: &str,
    genesis_dir: &str,
    mnemonic: &str,
    key_index: u16,
    async_wait: bool,
) -> Result<()> {
    let tmp_dir = format!("/tmp/{}", rand_jwt());
    fs::create_dir_all(&tmp_dir).c(d!())?;
    let password_file_name =
        recover_keystore(genesis_dir, &tmp_dir, mnemonic, key_index, 1).c(d!())?;

    let password_path = format!("{tmp_dir}/secrets/{password_file_name}");
    let keystore_path =
        format!("{tmp_dir}/validators/{password_file_name}/voting-keystore.json");

    exit(
        beacon_endpoint,
        genesis_dir,
        &keystore_path,
        &password_path,
        async_wait,
    )
    .c(d!())
    .and_then(|_| fs::remove_dir_all(&tmp_dir).c(d!()))
}

fn recover_keystore(
    genesis_dir: &str,
    data_dir: &str,
    mnemonic: &str,
    first_index: u16,
    count: u16,
) -> Result<String> {
    let cmd = format!(
        r#"
        echo '{mnemonic}' | lighthouse account validator recover \
            --stdin-inputs \
            --testnet-dir {genesis_dir} \
            --datadir {data_dir} \
            --first-index {first_index} \
            --count {count}
        "#
    );
    exec_output(&cmd).c(d!()).and_then(|s| {
        RE.captures(s.trim())
            .c(d!())?
            .get(1)
            .c(d!())
            .map(|c| c.as_str().to_owned())
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore]
    fn t_recover_keystore() {
        let len = "0x953a135356ae5e3b6f70c6611b9cdd64b18018e5e005b9cb6cd12c50960617b62823c76d5abd608683ad413e19dcf93a".len();

        for _ in 0..10 {
            let mnemonic = crate::pos::mnemonic::create_mnemonic_words();
            let v_pubkey = pnk!(recover_keystore(
                "static/genesis/example",
                "/tmp",
                &mnemonic,
                0,
                1
            ));
            assert_eq!(v_pubkey.len(), len);
            println!("{v_pubkey}");
        }
    }
}
