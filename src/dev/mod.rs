//!
//! `nbnet dev` SubCommand
//!
//! - make sure all names and ports are unique
//!     - keep a meta file in ${GLOBAL_BASE_DIR}
//! - write ports to the running-dir of every env
//!

use crate::{
    cfg::{DevCfg, DevOp},
    common::*,
    pos::{create_mnemonic_words, deposit::do_deposit, exit::exit_by_mnemonic},
    select_nodes_by_el_kind,
};
use alloy::{
    primitives::{hex, Address},
    signers::k256::ecdsa::SigningKey,
};
use chaindev::{
    beacon_dev::{
        Env as SysEnv, EnvCfg as SysCfg, EnvMeta, EnvOpts as SysOpts, Node, NodeKind,
        Op, NODE_HOME_GENESIS_DIR_DST, NODE_HOME_GENESIS_DST, NODE_HOME_VCDATA_DST,
    },
    common::NodeCmdGenerator,
    CustomOps, EnvName, NodeID,
};
use ruc::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{
    collections::{BTreeSet, HashSet},
    fs, mem,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvCfg {
    sys_cfg: SysCfg<CustomInfo, Ports, ExtraOp>,
}

impl From<DevCfg> for EnvCfg {
    fn from(cfg: DevCfg) -> Self {
        let mut en = cfg
            .env_name
            .as_deref()
            .map(EnvName::from)
            .unwrap_or_default();
        let op = match cfg.op.unwrap_or_default() {
            DevOp::Create(copts) => {
                let (genesis_tgz_path, genesis_vkeys_tgz_path) =
                    if let Some(s) = copts.genesis_data_pre_created {
                        let paths = s.split('+').collect::<Vec<_>>();
                        if 2 != paths.len() {
                            pnk!(Err(eg!("Invalid value")));
                        }
                        for p in paths.iter() {
                            if fs::metadata(p).is_err() {
                                pnk!(Err(eg!("File not accessible")));
                            }
                        }
                        (Some(paths[0].to_owned()), Some(paths[1].to_owned()))
                    } else {
                        (None, None)
                    };

                let custom_data = CustomInfo {
                    el_geth_bin: copts.el_geth_bin.unwrap_or("geth".to_owned()),
                    el_reth_bin: copts.el_reth_bin.unwrap_or("reth".to_owned()),
                    cl_bin: copts.cl_bin.unwrap_or_else(|| "lighthouse".to_owned()),
                };

                if let Some(n) = copts.env_name {
                    en = n.into();
                }

                let envopts = SysOpts {
                    host_ip: copts.host_ip,
                    block_itv: copts.block_time_secs.unwrap_or(0),
                    genesis_pre_settings: copts
                        .genesis_custom_settings_path
                        .unwrap_or_default(),
                    genesis_tgz_path,
                    genesis_vkeys_tgz_path,
                    custom_data,
                    force_create: copts.force_create,
                };

                Op::Create { opts: envopts }
            }
            DevOp::Deposit {
                env_name,
                nodes,
                num_per_node,
                wallet_seckey_path,
                withdraw_0x01_addr,
                async_wait,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Custom(ExtraOp::Deposit {
                    nodes,
                    num_per_node,
                    wallet_seckey_path,
                    withdraw_0x01_addr,
                    async_wait,
                })
            }
            DevOp::ValidatorExit {
                env_name,
                nodes,
                async_wait,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Custom(ExtraOp::ValidatorExit { nodes, async_wait })
            }
            DevOp::Destroy { env_name, force } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Destroy { force }
            }
            DevOp::Protect { env_name } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Protect
            }
            DevOp::Unprotect { env_name } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Unprotect
            }
            DevOp::Start {
                env_name,
                nodes,
                geth,
                reth,
                ignore_failed,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Start {
                    nodes: select_nodes_by_el_kind!(nodes, geth, reth, en),
                    ignore_failed,
                }
            }
            DevOp::Stop {
                env_name,
                nodes,
                geth,
                reth,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                let nodes = if "all" == nodes.as_str() {
                    None
                } else {
                    Some(nodes)
                };
                Op::Stop {
                    nodes: select_nodes_by_el_kind!(nodes, geth, reth, en),
                    force: false,
                }
            }
            DevOp::Restart {
                env_name,
                nodes,
                geth,
                reth,
                ignore_failed,
                wait_itv_secs,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Restart {
                    nodes: select_nodes_by_el_kind!(nodes, geth, reth, en),
                    ignore_failed,
                    wait_itv_secs,
                }
            }
            DevOp::PushNodes {
                env_name,
                reth,
                fullnode,
                num,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::PushNodes {
                    custom_data: alt!(
                        reth,
                        NodeCustomData::new_with_reth().to_json_value(),
                        NodeCustomData::new_with_geth().to_json_value()
                    ),
                    fullnode,
                    num,
                }
            }
            DevOp::KickNodes {
                env_name,
                nodes,
                num,
                geth,
                reth,
                force,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                let ids =
                    select_nodes_by_el_kind!(nodes, geth, reth, en, false).map(|ids| {
                        let num = num as usize;
                        if ids.len() > num {
                            ids.into_iter().take(num).collect()
                        } else {
                            ids
                        }
                    });
                Op::KickNodes {
                    nodes: ids,
                    num,
                    force,
                }
            }
            DevOp::SwitchELToGeth { env_name, nodes } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                let nodes = nodes
                    .split(',')
                    .map(|s| s.parse::<NodeID>().c(d!()))
                    .collect::<Result<BTreeSet<_>>>();
                Op::Custom(ExtraOp::SwitchELToGeth { nodes: pnk!(nodes) })
            }
            DevOp::SwitchELToReth { env_name, nodes } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                let nodes = nodes
                    .split(',')
                    .map(|s| s.parse::<NodeID>().c(d!()))
                    .collect::<Result<BTreeSet<_>>>();
                Op::Custom(ExtraOp::SwitchELToReth { nodes: pnk!(nodes) })
            }
            DevOp::Show {
                env_name,
                clean_up,
                write_back,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Custom(ExtraOp::Show {
                    clean_up,
                    write_back,
                })
            }
            DevOp::ListRpcs {
                env_name,
                el_web3,
                el_web3_ws,
                el_metric,
                cl_bn,
                cl_bn_metric,
                cl_vc,
                cl_vc_metric,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Custom(ExtraOp::ListRpcs {
                    el_web3,
                    el_web3_ws,
                    el_metric,
                    cl_bn,
                    cl_bn_metric,
                    cl_vc,
                    cl_vc_metric,
                })
            }
            DevOp::DebugFailedNodes { env_name } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::DebugFailedNodes
            }
            DevOp::List => Op::List,
        };

        Self {
            sys_cfg: SysCfg { name: en, op },
        }
    }
}

impl EnvCfg {
    pub fn exec(&self) -> Result<()> {
        self.sys_cfg.exec(CmdGenerator).c(d!())
            .and_then(|_| match &self.sys_cfg.op {
                Op::Create { opts: _ } => {
                    let mut env = load_sysenv(&self.sys_cfg.name).c(d!())?;
                    let fuhrer = env.meta.fuhrers.values_mut().next().unwrap();
                    let map = map!{B
                        env.meta.genesis_mnemonic_words.clone() => (0..env.meta.genesis_validator_num).collect()
                    };
                    json_deposits_append(&mut fuhrer.custom_data, map)
                        .c(d!())
                        .and_then(|_|
                    env.write_cfg().c(d!())
                            )
                }
                _ => Ok(()),
            })
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
struct CmdGenerator;

impl NodeCmdGenerator<Node<Ports>, EnvMeta<CustomInfo, Node<Ports>>> for CmdGenerator {
    fn cmd_cnt_running(
        &self,
        n: &Node<Ports>,
        e: &EnvMeta<CustomInfo, Node<Ports>>,
    ) -> String {
        format!(
            "ps ax -o pid,args | grep -E '({0}.*{3}/)|({1}.*{3}/)|({2}.*{3}/)' | grep -v 'grep' | wc -l",
            e.custom_data.el_geth_bin, e.custom_data.el_reth_bin, e.custom_data.cl_bin, n.home
        )
        .replace('+', r"\+")
    }

    fn cmd_for_start(
        &self,
        n: &Node<Ports>,
        e: &EnvMeta<CustomInfo, Node<Ports>>,
    ) -> String {
        let home = &n.home;
        let genesis_dir = format!("{home}/genesis");

        let rand_jwt = ruc::algo::rand::rand_jwt();
        let auth_jwt = format!("{home}/auth.jwt");

        let geth = if e.custom_data.el_geth_bin.contains("/") {
            e.custom_data.el_geth_bin.clone()
        } else {
            format!("$(which {})", e.custom_data.el_geth_bin)
        };

        let reth = if e.custom_data.el_reth_bin.contains("/") {
            e.custom_data.el_reth_bin.clone()
        } else {
            format!("$(which {})", e.custom_data.el_reth_bin)
        };

        let lighthouse = if e.custom_data.cl_bin.contains("/") {
            e.custom_data.cl_bin.clone()
        } else {
            format!("$(which {})", e.custom_data.cl_bin)
        };

        let prepare_cmd = format!(
            r#"
echo "{rand_jwt}" > {auth_jwt} | tr -d '\n' || exit 1

cp -f {geth} {home}/geth_bin || exit 1
cp -f {reth} {home}/reth_bin || exit 1
cp -f {lighthouse} {home}/lighthouse_bin || exit 1

if [ ! -d {genesis_dir} ]; then
    tar -C {home} -xpf {home}/{NODE_HOME_GENESIS_DST} || exit 1
    if [ ! -d {genesis_dir} ]; then
        mv {home}/$(tar -tf {home}/{NODE_HOME_GENESIS_DST} | head -1) {genesis_dir} || exit 1
    fi
fi "#
        );

        let el_kind = pnk!(json_el_kind(&n.custom_data));

        let local_ip = &e.host_ip;
        let ext_ip = local_ip; // for `ddev` it should be e.external_ip?

        let ts_start = ts!();
        let (el_bootnodes, cl_bn_bootnodes, cl_bn_trusted_peers, checkpoint_sync_url) = loop {
            let online_nodes = e
                .nodes_should_be_online
                .iter()
                .map(|(k, _)| k)
                .filter(|k| *k < n.id)
                .take(8) // take some static peers
                .chain(
                    e.nodes_should_be_online
                        .iter()
                        .map(|(k, _)| k)
                        .filter(|k| *k > n.id)
                        .collect::<HashSet<_>>() // take some random peers
                        .into_iter()
                        .take(8),
                )
                .collect::<HashSet<_>>();

            if online_nodes.is_empty() {
                break (String::new(), String::new(), String::new(), String::new());
            }

            let (el_rpc_endpoints, cl_bn_rpc_endpoints): (Vec<_>, Vec<_>) = e
                .nodes
                .values()
                .chain(e.fuhrers.values())
                .filter(|n| online_nodes.contains(&n.id))
                .map(|n| {
                    (
                        format!("http://{}:{}", &e.host_ip, n.ports.el_rpc),
                        format!("http://{}:{}", &e.host_ip, n.ports.cl_bn_rpc),
                    )
                })
                .unzip();

            let el_rpc_endpoints = el_rpc_endpoints
                .iter()
                .map(|i| i.as_str())
                .collect::<Vec<_>>();

            let el_bootnodes = if let Ok(r) = el_get_boot_nodes(&el_rpc_endpoints) {
                r
            } else if 10 > (ts!() - ts_start) {
                sleep_ms!(500);
                continue;
            } else {
                break (String::new(), String::new(), String::new(), String::new());
            };

            let cl_bn_rpc_endpoints = cl_bn_rpc_endpoints
                .iter()
                .map(|i| i.as_str())
                .collect::<Vec<_>>();

            let (online_urls, cl_bn_bootnodes, cl_bn_trusted_peers) =
                if let Ok((a, b, c)) = cl_get_boot_nodes(&cl_bn_rpc_endpoints) {
                    (a, b, c)
                } else if 10 > (ts!() - ts_start) {
                    sleep_ms!(500);
                    continue;
                } else {
                    break (el_bootnodes, String::new(), String::new(), String::new());
                };

            break (
                el_bootnodes,
                cl_bn_bootnodes,
                cl_bn_trusted_peers,
                online_urls.into_iter().next().unwrap_or_default(),
            );
        };

        ////////////////////////////////////////////////
        // EL
        ////////////////////////////////////////////////

        let el_dir = format!("{home}/{EL_DIR}");
        let el_genesis = format!("{genesis_dir}/genesis.json");
        let el_discovery_port = n.ports.el_discovery;
        let el_discovery_v5_port = n.ports.el_discovery_v5;
        let el_rpc_port = n.ports.el_rpc;
        let el_rpc_ws_port = n.ports.el_rpc_ws;
        let el_engine_port = n.ports.el_engine_api;
        let el_metric_port = n.ports.el_metric;

        let el_cmd = if Eth1Kind::Geth == el_kind {
            let el_gc_mode = if matches!(n.kind, NodeKind::FullNode) {
                "full"
            } else {
                "archive" // Fuhrer nodes belong to The ArchiveNode
            };

            let cmd_init_part = format!(
                r#"
if [ ! -d {el_dir} ]; then
    mkdir -p {el_dir}/logs || exit 1
    {geth} init --datadir={el_dir} --state.scheme=hash \
        {el_genesis} >{el_dir}/logs/{EL_LOG_NAME} 2>&1 || exit 1
fi "#
            );

            let cmd_run_part_0 = format!(
                r#"
nohup {home}/geth_bin \
    --syncmode=full \
    --gcmode={el_gc_mode} \
    --networkid=$(grep -Po '(?<="chainId":)\s*\d+' {el_genesis} | tr -d ' ') \
    --datadir={el_dir} \
    --log.file={el_dir}/logs/{EL_LOG_NAME} \
    --log.compress \
    --log.rotate \
    --log.maxsize=12 \
    --log.maxbackups=20 \
    --state.scheme=hash \
    --nat=extip:{ext_ip} \
    --port={el_discovery_port} \
    --discovery.port={el_discovery_port} \
    --discovery.v5 \
    --http --http.addr={local_ip} --http.port={el_rpc_port} --http.vhosts='*' --http.corsdomain='*' \
    --http.api='admin,debug,eth,net,txpool,web3,rpc' \
    --ws --ws.addr={local_ip} --ws.port={el_rpc_ws_port} --ws.origins='*' \
    --ws.api='admin,debug,eth,net,txpool,web3,rpc' \
    --authrpc.addr={local_ip} --authrpc.port={el_engine_port} \
    --authrpc.jwtsecret={auth_jwt} \
    --metrics \
    --metrics.addr {local_ip} \
    --metrics.port={el_metric_port} "#
            );

            let cmd_run_part_1 = if el_bootnodes.is_empty() {
                String::new()
            } else {
                format!(" --bootnodes='{el_bootnodes}'")
            };

            let cmd_run_part_2 = " >/dev/null 2>&1 &";

            cmd_init_part + &cmd_run_part_0 + &cmd_run_part_1 + cmd_run_part_2
        } else if Eth1Kind::Reth == el_kind {
            let cmd_init_part = format!(
                r#"
if [ ! -d {el_dir} ]; then
    mkdir -p {el_dir}/logs || exit 1
    {reth} init --datadir={el_dir} --chain={el_genesis} \
        --log.file.directory={el_dir}/logs >/dev/null 2>&1 || exit 1
    ln -sv {el_dir}/logs/*/reth.log {el_dir}/logs/{EL_LOG_NAME} >/dev/null 2>&1 || exit 1
fi "#
            );

            let cmd_run_part_0 = format!(
                r#"
nohup {home}/reth_bin node \
    --chain={el_genesis} \
    --datadir={el_dir} \
    --log.file.directory={el_dir}/logs \
    --log.file.max-size=12 \
    --log.file.max-files=20 \
    --ipcdisable \
    --nat=extip:{ext_ip} \
    --port={el_discovery_port} \
    --discovery.port={el_discovery_port} \
    --enable-discv5-discovery \
    --discovery.v5.port={el_discovery_v5_port} \
    --http --http.addr={local_ip} --http.port={el_rpc_port} --http.corsdomain='*' \
    --http.api='admin,debug,eth,net,txpool,web3,rpc' \
    --ws --ws.addr={local_ip} --ws.port={el_rpc_ws_port} --ws.origins='*' \
    --ws.api='admin,debug,eth,net,txpool,web3,rpc' \
    --authrpc.addr={local_ip} --authrpc.port={el_engine_port} \
    --authrpc.jwtsecret={auth_jwt} \
    --metrics='0.0.0.0:{el_metric_port}' "#
            );

            let cmd_run_part_1 = if el_bootnodes.is_empty() {
                String::new()
            } else {
                format!(" --bootnodes='{el_bootnodes}' --trusted-peers='{el_bootnodes}'")
            };

            //
            // // This option is unstable in `reth`,
            // // should do NOT use it for now
            //
            // if matches!(n.kind, NodeKind::FullNode) {
            //     cmd_run_part_1.push_str(" --full");
            // }

            let cmd_run_part_2 = " >/dev/null 2>&1 &";

            cmd_init_part + &cmd_run_part_0 + &cmd_run_part_1 + cmd_run_part_2
        } else {
            pnk!(Err(eg!("The fuhrering world is over!")))
        };

        ////////////////////////////////////////////////
        // CL
        ////////////////////////////////////////////////

        let cl_bn_dir = format!("{home}/{CL_BN_DIR}");
        let cl_vc_dir = format!("{home}/{CL_VC_DIR}");
        let cl_genesis = genesis_dir;
        let cl_bn_discovery_port = n.ports.cl_discovery;
        let cl_bn_discovery_quic_port = n.ports.cl_discovery_quic;
        let cl_bn_rpc_port = n.ports.cl_bn_rpc;
        let cl_vc_rpc_port = n.ports.cl_vc_rpc;
        let cl_bn_metric_port = n.ports.cl_bn_metric;
        let cl_vc_metric_port = n.ports.cl_vc_metric;

        let (cl_slots_per_rp, epochs_per_migration, reconstruct_states) =
            if matches!(n.kind, NodeKind::FullNode) {
                (2048, 256, "")
            } else {
                (32, u64::MAX, "--reconstruct-historic-states")
            };

        let cl_bn_cmd = {
            let cmd_run_part_0 = format!(
                r#"
mkdir -p {cl_bn_dir} || exit 1
sleep 0.5

nohup {home}/lighthouse_bin beacon_node \
    --testnet-dir={cl_genesis} \
    --datadir={cl_bn_dir} \
    --logfile={cl_bn_dir}/logs/{CL_BN_LOG_NAME} \
    --logfile-compress \
    --logfile-max-size=12 \
    --logfile-max-number=20 \
    --staking \
    {reconstruct_states} \
    --epochs-per-migration={epochs_per_migration} \
    --slots-per-restore-point={cl_slots_per_rp} \
    --enr-address={ext_ip} \
    --disable-enr-auto-update \
    --disable-upnp \
    --disable-packet-filter \
    --subscribe-all-subnets \
    --listen-address={local_ip} \
    --port={cl_bn_discovery_port} \
    --discovery-port={cl_bn_discovery_port} \
    --quic-port={cl_bn_discovery_quic_port} \
    --execution-endpoints='http://{local_ip}:{el_engine_port}' \
    --jwt-secrets={auth_jwt} \
    --suggested-fee-recipient={FEE_RECIPIENT} \
    --http --http-address={local_ip} \
    --http-port={cl_bn_rpc_port} --http-allow-origin='*' \
    --metrics --metrics-address={local_ip} \
    --metrics-port={cl_bn_metric_port} --metrics-allow-origin='*' "#
            );

            let mut cmd_run_part_1 = if cl_bn_bootnodes.is_empty() {
                String::new()
            } else {
                format!(" --boot-nodes='{cl_bn_bootnodes}' --trusted-peers='{cl_bn_trusted_peers}'")
            };

            if node_sync_from_genesis() || checkpoint_sync_url.is_empty() {
                cmd_run_part_1.push_str(" --allow-insecure-genesis-sync");
            } else {
                cmd_run_part_1
                    .push_str(&format!(" --checkpoint-sync-url={checkpoint_sync_url}"));
            }

            // Disable this line in the `ddev` mod?
            cmd_run_part_1.push_str(" --enable-private-discovery");

            let cmd_run_part_2 = " >/dev/null 2>&1 &";

            cmd_run_part_0 + &cmd_run_part_1 + cmd_run_part_2
        };

        let cl_vc_cmd = {
            let beacon_nodes = format!("http://{local_ip}:{}", n.ports.cl_bn_rpc);

            let cmd_run_part_0 = if n.id == *e.fuhrers.keys().next().unwrap() {
                let id = n.id;
                let ts = ts!();
                // The first fuhrer node
                format!(
                    r#"
if [[ -f '{home}/{NODE_HOME_VCDATA_DST}' ]]; then
    vcdata_dir_name=$(tar -tf {home}/{NODE_HOME_VCDATA_DST} | head -1 | tr -d '/')
    if [[ (! -d '{cl_vc_dir}/validators') && ("" != $vcdata_dir_name) ]]; then
        rm -rf /tmp/{cl_vc_dir}_{id}_{ts} || exit 1
        mkdir -p {cl_vc_dir} /tmp/{cl_vc_dir}_{id}_{ts} || exit 1
        tar -C /tmp/{cl_vc_dir}_{id}_{ts} -xpf {home}/{NODE_HOME_VCDATA_DST} || exit 1
        mv /tmp/{cl_vc_dir}_{id}_{ts}/${{vcdata_dir_name}}/* {cl_vc_dir}/ || exit 1
    fi
fi "#
                )
            } else {
                String::new()
            };

            let cmd_run_part_1 = format!(
                r#"
mkdir -p {cl_vc_dir} || exit 1
sleep 1

nohup {home}/lighthouse_bin validator_client \
    --testnet-dir={cl_genesis} \
    --datadir={cl_vc_dir}\
    --logfile={cl_vc_dir}/logs/{CL_VC_LOG_NAME} \
    --logfile-compress \
    --logfile-max-size=12 \
    --logfile-max-number=20 \
    --beacon-nodes='{beacon_nodes}' \
    --init-slashing-protection \
    --suggested-fee-recipient={FEE_RECIPIENT} \
    --unencrypted-http-transport \
    --enable-doppelganger-protection \
    --http --http-address="127.0.0.1" \
    --http-port={cl_vc_rpc_port} --http-allow-origin='*' \
    --metrics --metrics-address={local_ip} \
    --metrics-port={cl_vc_metric_port} --metrics-allow-origin='*' \
    >/dev/null 2>&1 &
     "#
            );

            cmd_run_part_0 + &cmd_run_part_1
        };

        ////////////////////////////////////////////////
        // FINAL
        ////////////////////////////////////////////////

        format!(
            r#"

            {prepare_cmd}

            {el_cmd}

            {cl_bn_cmd}

            {cl_vc_cmd}

            "#
        )
    }

    fn cmd_for_stop(
        &self,
        n: &Node<Ports>,
        _e: &EnvMeta<CustomInfo, Node<Ports>>,
        force: bool,
    ) -> String {
        format!(
            "for i in \
            $(ps ax -o pid,args|grep '{}'|grep -v grep|sed -r 's/(^ *)|( +)/ /g'|cut -d ' ' -f 2); \
            do kill {} $i; done",
            &n.home,
            alt!(force, "-9", ""),
        )
    }
}

//////////////////////////////////////////////////
//////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
enum ExtraOp {
    Deposit {
        nodes: String, /*comma separated node IDs*/
        num_per_node: u8,
        wallet_seckey_path: Option<String>,
        withdraw_0x01_addr: Option<String>,
        async_wait: bool,
    },
    ValidatorExit {
        nodes: String, /*comma separated node IDs*/
        async_wait: bool,
    },
    Show {
        clean_up: bool,
        write_back: bool,
    },
    ListRpcs {
        el_web3: bool,
        el_web3_ws: bool,
        el_metric: bool,
        cl_bn: bool,
        cl_bn_metric: bool,
        cl_vc: bool,
        cl_vc_metric: bool,
    },
    SwitchELToGeth {
        nodes: BTreeSet<NodeID>,
    },
    SwitchELToReth {
        nodes: BTreeSet<NodeID>,
    },
}

impl CustomOps for ExtraOp {
    fn exec(&self, en: &EnvName) -> Result<()> {
        let mut env = load_sysenv(en).c(d!())?;

        match self {
            Self::Deposit {
                nodes,
                num_per_node,
                wallet_seckey_path,
                withdraw_0x01_addr,
                async_wait,
            } => {
                let nodes = nodes.trim();
                let withdraw_addr = withdraw_0x01_addr.as_ref().map(|addr| addr.trim());
                let mut env = load_sysenv(en).c(d!())?;

                // All non-fuhrer nodes
                let nodes = if "all" == nodes {
                    env.meta.nodes.values().cloned().collect::<Vec<_>>()
                } else {
                    let ids = nodes
                        .split(',')
                        .map(|id| id.parse::<NodeID>().c(d!()))
                        .collect::<Result<Vec<_>>>()?;
                    let mut nodes = vec![];
                    for id in ids.iter() {
                        if env.meta.fuhrers.contains_key(id) {
                            return Err(eg!(
                                "Fuhrer node(id: {}) does not accept deposits",
                                id
                            ));
                        }
                        if let Some(n) = env.meta.nodes.get(id).cloned() {
                            nodes.push(n);
                        } else {
                            return Err(eg!("The node(id: {}) does not exist", id));
                        }
                    }
                    nodes
                };

                if nodes.is_empty() {
                    return Err(eg!("No target nodes found!"));
                }

                let (wallet_addr, wallet_key) = if let Some(path) = wallet_seckey_path {
                    let key = fs::read_to_string(path).c(d!())?.trim().to_owned();
                    let addr = if let Some(addr) = withdraw_addr.map(|s| s.to_owned()) {
                        addr
                    } else {
                        let k = hex::decode(&key)
                            .c(d!())
                            .and_then(|k| SigningKey::from_slice(&k).c(d!()))?;
                        Address::from_private_key(&k).to_string()
                    };
                    (addr, key)
                } else {
                    let (addr, obj) = env
                        .meta
                        .premined_accounts
                        .as_object()
                        .unwrap()
                        .iter()
                        .next()
                        .c(d!())?;
                    let addr = withdraw_addr.unwrap_or(addr).to_owned();
                    let key = obj.as_object().unwrap()["secretKey"]
                        .as_str()
                        .unwrap()
                        .to_owned();
                    (addr, key)
                };

                let testnet_dir =
                    format!("{}/{NODE_HOME_GENESIS_DIR_DST}", env.meta.home);
                let config_yml = format!("{}/config.yaml", testnet_dir);
                let cfg = fs::read_to_string(config_yml)
                    .c(d!())
                    .and_then(|s| serde_yml::from_str::<serde_yml::Value>(&s).c(d!()))?;
                let deposit_contract =
                    cfg["DEPOSIT_CONTRACT_ADDRESS"].as_str().unwrap().to_owned();

                let runtime = crate::common::new_sb_runtime();

                let selected_node_idx = ts!() as usize % nodes.len();
                let el_rpc_endpoint = format!(
                    "http://{}:{}",
                    env.meta.host_ip, nodes[selected_node_idx].ports.el_rpc
                );

                for n in nodes.into_iter() {
                    let tmp_dir =
                        format!("/tmp/{}_{}", ts!(), ruc::algo::rand::rand_jwt());
                    omit!(fs::remove_dir_all(&tmp_dir));
                    fs::create_dir_all(&tmp_dir).c(d!())?;

                    let mnemonic_path = format!("{tmp_dir}/mnemonic.txt");

                    let deposits_json = format!("{tmp_dir}/deposits.json");
                    let validators_json = format!("{tmp_dir}/validators.json");

                    let mnemonic = create_mnemonic_words();
                    fs::write(&mnemonic_path, &mnemonic).c(d!())?;

                    let node_vc_data_dir = format!("{}/{CL_VC_DIR}", n.home);
                    let node_vc_api_token =
                        format!("{}/validators/api-token.txt", node_vc_data_dir);

                    // let node_el_rpc_endpoint =
                    //     format!("http://{}:{}", env.meta.host_ip, n.ports.el_rpc);

                    let node_vc_rpc_endpoint =
                        format!("http://localhost:{}", n.ports.cl_vc_rpc);

                    let num_per_node = if 0 == *num_per_node {
                        ts!() as u8 % 20 + 1
                    } else {
                        *num_per_node
                    };
                    let cmd = format!(
                        r#"
                        lighthouse validator-manager create \
                            --testnet-dir {testnet_dir} \
                            --mnemonic-path {mnemonic_path} \
                            --first-index 0 \
                            --count {num_per_node} \
                            --eth1-withdrawal-address {wallet_addr} \
                            --suggested-fee-recipient {wallet_addr} \
                            --output-path {tmp_dir}
                        "#
                    );
                    ruc::cmd::exec_output(&cmd).c(d!())?;

                    let node_cmd = format!(
                        r#"
                        lighthouse validator-manager import \
                            --testnet-dir {testnet_dir} \
                            --datadir {node_vc_data_dir} \
                            --validators-file {validators_json} \
                            --vc-url {node_vc_rpc_endpoint} \
                            --vc-token {node_vc_api_token}
                        "#
                    );
                    ruc::cmd::exec_output(&node_cmd).c(d!())?;

                    let deposits_json = fs::read_to_string(deposits_json).c(d!())?;
                    runtime
                        .block_on(do_deposit(
                            &el_rpc_endpoint,
                            &deposit_contract,
                            &deposits_json,
                            &wallet_key,
                            *async_wait,
                        ))
                        .c(d!())?;

                    json_deposits_append(
                        &mut env.meta.nodes.get_mut(&n.id).unwrap().custom_data,
                        map! {B mnemonic => (0..num_per_node as u16).collect() },
                    )
                    .c(d!())
                    .and_then(|_| env.write_cfg().c(d!()))
                    .and_then(|_| fs::remove_dir_all(tmp_dir).c(d!()))?;
                }

                Ok(())
            }
            Self::ValidatorExit { nodes, async_wait } => {
                let nodes = nodes.trim();
                let mut env = load_sysenv(en).c(d!())?;

                // All non-fuhrer nodes
                let nodes = if "all" == nodes {
                    env.meta.nodes.values().cloned().collect::<Vec<_>>()
                } else {
                    let ids = nodes
                        .split(',')
                        .map(|id| id.parse::<NodeID>().c(d!()))
                        .collect::<Result<Vec<_>>>()?;
                    let mut nodes = vec![];
                    for id in ids.iter() {
                        if env.meta.fuhrers.contains_key(id) {
                            return Err(eg!(
                                "Fuhrer node(id: {}) does not accept deposits",
                                id
                            ));
                        }
                        if let Some(n) = env.meta.nodes.get(id).cloned() {
                            nodes.push(n);
                        } else {
                            return Err(eg!("The node(id: {}) does not exist", id));
                        }
                    }
                    nodes
                };

                if nodes.is_empty() {
                    return Err(eg!("No target nodes found!"));
                }

                let testnet_dir =
                    format!("{}/{NODE_HOME_GENESIS_DIR_DST}", env.meta.home);

                let beacon_rpc_endpoint =
                    format!("http://{}:{}", env.meta.host_ip, nodes[0].ports.cl_bn_rpc);

                for n in nodes.into_iter() {
                    if let Some(c) = n.custom_data {
                        for (mnemonic, idxs) in c["deposits"].as_object().c(d!())?.iter()
                        {
                            for idx in idxs.as_array().c(d!())?.iter() {
                                let idx = idx.as_u64().c(d!())? as u16;
                                let ret = exit_by_mnemonic(
                                    &beacon_rpc_endpoint,
                                    &testnet_dir,
                                    mnemonic,
                                    idx,
                                    *async_wait,
                                )
                                .c(d!("Node: {}, {}/{}", n.id, mnemonic, idx))
                                .and_then(|_| {
                                    json_deposits_remove(
                                        &mut env
                                            .meta
                                            .nodes
                                            .get_mut(&n.id)
                                            .unwrap()
                                            .custom_data,
                                        mnemonic,
                                        idx,
                                    )
                                    .c(d!())
                                })
                                .and_then(|_| env.write_cfg().c(d!()));
                                info_omit!(ret);
                            }
                        }
                    }
                }

                Ok(())
            }
            Self::Show {
                clean_up,
                write_back,
            } => {
                if *clean_up {
                    macro_rules! cl_up {
                        ($nodes: tt) => {{
                            for n in env.meta.$nodes.values_mut() {
                                json_deposits_clean_up(&mut n.custom_data).c(d!())?;
                            }
                        }};
                    }
                    cl_up!(fuhrers);
                    cl_up!(nodes);

                    if *write_back {
                        env.write_cfg().c(d!())?;
                    }
                }

                let mut ret = pnk!(serde_json::to_value(&env));

                ret.as_object_mut()
                    .unwrap()
                    .remove("node_cmdline_generator");

                let meta = ret["meta"].as_object_mut().unwrap();

                meta.remove("genesis");
                meta.remove("genesis_vkeys");
                meta.remove("genesis_mnemonic_words");
                meta.remove("genesis_validator_num");
                meta.remove("nodes_should_be_online");
                meta.remove("next_node_id");

                let mut list_to_cnt = |field: &str| {
                    for ids in meta[field]
                        .as_object_mut()
                        .unwrap()
                        .values_mut()
                        .flat_map(|v| {
                            v.as_object_mut().unwrap()["custom_data"]
                                .as_object_mut()
                                .unwrap()["deposits"]
                                .as_object_mut()
                                .unwrap()
                                .values_mut()
                        })
                    {
                        mem::swap(
                            ids,
                            &mut JsonValue::Number(ids.as_array().unwrap().len().into()),
                        );
                    }
                };

                list_to_cnt("fuhrer_nodes");
                list_to_cnt("nodes");

                println!("{}", pnk!(serde_json::to_string_pretty(&ret)));

                Ok(())
            }
            Self::ListRpcs {
                el_web3,
                el_web3_ws,
                el_metric,
                cl_bn,
                cl_bn_metric,
                cl_vc,
                cl_vc_metric,
            } => {
                let default = !(*el_web3
                    || *el_web3_ws
                    || *el_metric
                    || *cl_bn
                    || *cl_bn_metric
                    || *cl_vc
                    || *cl_vc_metric);
                let env = load_sysenv(en).c(d!())?;

                let mut buf_el_web3 = vec![];
                let mut buf_el_web3_ws = vec![];
                let mut buf_el_metric = vec![];
                let mut buf_cl_bn = vec![];
                let mut buf_cl_bn_metric = vec![];
                let mut buf_cl_vc = vec![];
                let mut buf_cl_vc_metric = vec![];

                env.meta
                    .fuhrers
                    .values()
                    .chain(env.meta.nodes.values())
                    .for_each(|n| {
                        if *el_web3 || default {
                            buf_el_web3.push(format!(
                                "    http://{}:{}",
                                env.meta.host_ip, n.ports.el_rpc
                            ));
                        }
                        if *el_web3_ws || default {
                            buf_el_web3_ws.push(format!(
                                "    http://{}:{}",
                                env.meta.host_ip, n.ports.el_rpc_ws
                            ));
                        }
                        if *el_metric || default {
                            buf_el_metric.push(format!(
                                "    http://{}:{}",
                                env.meta.host_ip, n.ports.el_metric
                            ));
                        }
                        if *cl_bn || default {
                            buf_cl_bn.push(format!(
                                "    http://{}:{}",
                                env.meta.host_ip, n.ports.cl_bn_rpc
                            ));
                        }
                        if *cl_bn_metric || default {
                            buf_cl_bn_metric.push(format!(
                                "    http://{}:{}",
                                env.meta.host_ip, n.ports.cl_bn_metric
                            ));
                        }
                        if *cl_vc || default {
                            buf_cl_vc.push(format!(
                                "    http://{}:{}",
                                env.meta.host_ip, n.ports.cl_vc_rpc
                            ));
                        }
                        if *cl_vc_metric || default {
                            buf_cl_vc_metric.push(format!(
                                "    http://{}:{}",
                                env.meta.host_ip, n.ports.cl_vc_metric
                            ));
                        }
                    });

                if !buf_el_web3.is_empty() {
                    println!("\x1b[33;1mEL WEB3 RPCs:\x1b[0m");
                    buf_el_web3.iter().for_each(|l| {
                        println!("{l}");
                    });
                }

                if !buf_el_web3_ws.is_empty() {
                    println!("\x1b[33;1mEL WEB3 WS RPCs:\x1b[0m");
                    buf_el_web3_ws.iter().for_each(|l| {
                        println!("{l}");
                    });
                }

                if !buf_el_metric.is_empty() {
                    println!("\x1b[33;1mEL METRIC RPCs:\x1b[0m");
                    buf_el_metric.iter().for_each(|l| {
                        println!("{l}");
                    });
                }

                if !buf_cl_bn.is_empty() {
                    println!("\x1b[33;1mCL BEACON RPCs:\x1b[0m");
                    buf_cl_bn.iter().for_each(|l| {
                        println!("{l}");
                    });
                }

                if !buf_cl_bn_metric.is_empty() {
                    println!("\x1b[33;1mCL BEACON METRIC RPCs:\x1b[0m");
                    buf_cl_bn_metric.iter().for_each(|l| {
                        println!("{l}");
                    });
                }

                if !buf_cl_vc.is_empty() {
                    println!("\x1b[33;1mCL VALIDATOR RPCs:\x1b[0m");
                    buf_cl_vc.iter().for_each(|l| {
                        println!("{l}");
                    });
                }

                if !buf_cl_vc_metric.is_empty() {
                    println!("\x1b[33;1mCL VALIDATOR METRIC RPCs:\x1b[0m");
                    buf_cl_vc_metric.iter().for_each(|l| {
                        println!("{l}");
                    });
                }

                Ok(())
            }
            Self::SwitchELToGeth { nodes } => {
                let mut ns = vec![];
                for id in nodes.iter() {
                    let n = env
                        .meta
                        .nodes
                        .get(id)
                        .or_else(|| env.meta.fuhrers.get(id))
                        .cloned()
                        .c(d!("The node(id: {id}) not found"))?;
                    alt!(
                        !json_el_kind_matched(&n.custom_data, Eth1Kind::Geth).c(d!())?,
                        ns.push(n)
                    );
                }

                SysCfg {
                    name: en.clone(),
                    op: Op::<CustomInfo, Ports, ExtraOp>::Stop {
                        nodes: Some(ns.iter().map(|n| n.id).collect()),
                        force: false,
                    },
                }
                .exec(CmdGenerator)
                .c(d!())?;

                sleep_ms!(3000); // wait for the graceful exiting process

                for (i, n) in ns.iter().enumerate() {
                    // Just remove $EL_DIR.
                    // When starting up, if $EL_DIR is detected to not exist,
                    // the new client will re-create it, and sync data from the CL.
                    fs::remove_dir_all(format!("{}/{EL_DIR}", n.home)).c(d!())?;

                    println!(
                        "The {}th node has been switched, node id: {}",
                        1 + i,
                        n.id
                    );
                }

                for id in ns.iter().map(|n| n.id) {
                    json_el_kind_set(
                        &mut env
                            .meta
                            .nodes
                            .get_mut(&id)
                            .or_else(|| env.meta.fuhrers.get_mut(&id))
                            .unwrap()
                            .custom_data,
                        Eth1Kind::Geth,
                    )
                    .c(d!())?;
                }

                env.write_cfg().c(d!())
            }
            Self::SwitchELToReth { nodes } => {
                let mut ns = vec![];
                for id in nodes.iter() {
                    let n = env
                        .meta
                        .nodes
                        .get(id)
                        .or_else(|| env.meta.fuhrers.get(id))
                        .cloned()
                        .c(d!("The node(id: {id}) not found"))?;
                    alt!(
                        !json_el_kind_matched(&n.custom_data, Eth1Kind::Reth).c(d!())?,
                        ns.push(n)
                    );
                }

                SysCfg {
                    name: en.clone(),
                    op: Op::<CustomInfo, Ports, ExtraOp>::Stop {
                        nodes: Some(ns.iter().map(|n| n.id).collect()),
                        force: false,
                    },
                }
                .exec(CmdGenerator)
                .c(d!())?;

                sleep_ms!(3000); // wait for the graceful exiting process

                for (i, n) in ns.iter().enumerate() {
                    // Just remove $EL_DIR.
                    // When starting up, if $EL_DIR is detected to not exist,
                    // the new client will re-create it, and sync data from the CL.
                    fs::remove_dir_all(format!("{}/{EL_DIR}", n.home)).c(d!())?;

                    println!(
                        "The {}th node has been switched, node id: {}",
                        1 + i,
                        n.id
                    );
                }

                for id in ns.iter().map(|n| n.id) {
                    json_el_kind_set(
                        &mut env
                            .meta
                            .nodes
                            .get_mut(&id)
                            .or_else(|| env.meta.fuhrers.get_mut(&id))
                            .unwrap()
                            .custom_data,
                        Eth1Kind::Reth,
                    )
                    .c(d!())?;
                }

                env.write_cfg().c(d!())
            }
        }
    }
}

fn load_sysenv(en: &EnvName) -> Result<SysEnv<CustomInfo, Ports, CmdGenerator>> {
    SysEnv::load_env_by_name(en)
        .c(d!())?
        .c(d!("ENV does not exist!"))
}
