//!
//! `nbnet ddev` SubCommand
//!
//! The distributed version of `nbnet dev`.
//!

use crate::{
    cfg::{DDevCfg, DDevOp},
    common::*,
    pos::{create_mnemonic_words, deposit::do_deposit, exit::exit_by_mnemonic},
};
use alloy::{
    primitives::{hex, Address},
    signers::k256::ecdsa::SigningKey,
};
use chaindev::{
    beacon_ddev::{
        remote::{
            collect_files_from_nodes as env_collect_files,
            collect_tgz_from_nodes as env_collect_tgz,
        },
        Env as SysEnv, EnvCfg as SysCfg, EnvMeta, EnvOpts as SysOpts, Node, NodeKind,
        Op, NODE_HOME_GENESIS_DIR_DST, NODE_HOME_GENESIS_DST, NODE_HOME_VCDATA_DST,
    },
    common::{
        hosts::{HostAddr, HostExpression, Hosts},
        remote::Remote,
        NodeCmdGenerator,
    },
    CustomOps, EnvName, NodeID,
};
use ruc::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{
    collections::{BTreeSet, HashSet},
    env, fs, mem,
    str::FromStr,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvCfg {
    sys_cfg: SysCfg<CustomInfo, Ports, ExtraOp>,
}

impl EnvCfg {
    pub fn exec(&self) -> Result<()> {
        self.sys_cfg
            .exec(CmdGenerator)
            .c(d!())
            .and_then(|_| match &self.sys_cfg.op {
                Op::Create { opts: _ } => {
                    let mut env = load_sysenv(&self.sys_cfg.name).c(d!())?;
                    let fuhrer = env.meta.fuhrers.values_mut().next().unwrap();
                    let map = map!{B
                        env.meta.genesis_mnemonic_words.clone() => (0..env.meta.genesis_validator_num).collect()
                    };
                    json_deposits_append(&mut fuhrer.custom_data, map);
                    env.write_cfg().c(d!())
                }
                _ => Ok(()),
            })
    }
}

#[macro_export]
macro_rules! select_nodes_by_el_kind {
    ($nodes: expr, $geth: expr, $reth: expr, $en: expr, $include_fuhrer_nodes: expr) => {{
        if $nodes.is_none() && !$geth && !$reth {
            None
        } else if $nodes.is_some() && !$geth && !$reth {
            let parsed = $nodes
                .unwrap()
                .split(',')
                .map(|id| id.parse::<NodeID>().c(d!()))
                .collect::<Result<BTreeSet<_>>>();

            Some(pnk!(parsed, "Invalid ID[s], parse failed"))
        } else {
            let env = pnk!(load_sysenv(&$en));
            let get_ids = |nodes: std::collections::BTreeMap<NodeID, Node<Ports>>| {
                nodes
                    .values()
                    .filter(|n| {
                        if $geth && $reth {
                            true
                        } else if $geth {
                            json_el_kind_matched(&n.custom_data, Eth1Kind::Geth)
                        } else if $reth {
                            json_el_kind_matched(&n.custom_data, Eth1Kind::Reth)
                        } else {
                            true
                        }
                    })
                    .map(|n| n.id)
                    .collect::<BTreeSet<_>>()
            };

            let mut ids = get_ids(env.meta.nodes);

            if $include_fuhrer_nodes {
                ids.append(&mut get_ids(env.meta.fuhrers));
            }

            if let Some(s) = $nodes {
                let parsed = s
                    .split(',')
                    .map(|id| id.parse::<NodeID>().c(d!()))
                    .collect::<Result<BTreeSet<_>>>();
                let parsed_ids = pnk!(parsed, "Invalid ID[s], parse failed");
                ids = ids.intersection(&parsed_ids).copied().collect();
            }

            Some(ids)
        }
    }};
    ($nodes: expr, $geth: expr, $reth: expr, $en: expr) => {{
        select_nodes_by_el_kind!($nodes, $geth, $reth, $en, true)
    }};
}

impl From<DDevCfg> for EnvCfg {
    fn from(cfg: DDevCfg) -> Self {
        let mut en = cfg
            .env_name
            .as_deref()
            .map(EnvName::from)
            .unwrap_or_default();

        let op = match cfg.op.unwrap_or_default() {
            DDevOp::Create(copts) => {
                if let Some(n) = copts.env_name {
                    en = n.into();
                }

                let hosts = copts
                    .hosts
                    .as_deref()
                    .map(|hs| hs.into())
                    .or_else(env_hosts);
                let hosts = pnk!(
                    hosts,
                    "No hosts registered! Use `--hosts` or $NB_DDEV_HOSTS to set."
                );

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

                let envopts = SysOpts {
                    hosts,
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
            DDevOp::Deposit {
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
            DDevOp::ValidatorExit {
                env_name,
                nodes,
                async_wait,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Custom(ExtraOp::ValidatorExit { nodes, async_wait })
            }
            DDevOp::Destroy { env_name, force } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Destroy { force }
            }
            DDevOp::DestroyAll { force } => Op::DestroyAll { force },
            DDevOp::Protect { env_name } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Protect
            }
            DDevOp::Unprotect { env_name } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Unprotect
            }
            DDevOp::Start {
                env_name,
                nodes,
                geth,
                reth,
                ignore_failed,
                realloc_ports,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Start {
                    nodes: select_nodes_by_el_kind!(nodes, geth, reth, en),
                    ignore_failed,
                    realloc_ports,
                }
            }
            DDevOp::StartAll => Op::StartAll,
            DDevOp::Stop {
                env_name,
                nodes,
                geth,
                reth,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Stop {
                    nodes: select_nodes_by_el_kind!(nodes, geth, reth, en),
                    force: false,
                }
            }
            DDevOp::StopAll => Op::StopAll { force: false },
            DDevOp::PushNodes {
                env_name,
                host_addr,
                reth,
                fullnode,
                num,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::PushNodes {
                    host: host_addr.map(|a| pnk!(HostAddr::from_str(&a))),
                    custom_data: alt!(
                        reth,
                        NodeCustomData::new_with_reth().to_json_value(),
                        NodeCustomData::new_with_geth().to_json_value()
                    ),
                    fullnode,
                    num,
                }
            }
            DDevOp::MigrateNodes {
                env_name,
                nodes,
                host_addr,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                let parsed = nodes
                    .split(',')
                    .map(|id| id.parse::<NodeID>().c(d!()))
                    .collect::<Result<BTreeSet<_>>>();
                let nodes = pnk!(parsed, "Invalid ID[s], parse failed");
                Op::MigrateNodes {
                    nodes,
                    host: host_addr.map(|a| pnk!(HostAddr::from_str(&a))),
                }
            }
            DDevOp::KickNodes {
                env_name,
                nodes,
                num,
                geth,
                reth,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                let ids =
                    select_nodes_by_el_kind!(nodes, geth, reth, en, false).map(|ids| {
                        let num = num as usize;
                        if ids.len() > num {
                            ids.into_iter().rev().take(num).collect()
                        } else {
                            ids
                        }
                    });
                Op::KickNodes { nodes: ids, num }
            }
            DDevOp::PushHosts { env_name, hosts } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                let hosts = pnk!(hosts.map(|h| pnk!(parse_cfg(&h))).or_else(env_hosts));
                Op::PushHosts { hosts }
            }
            DDevOp::KickHosts {
                env_name,
                host_ids,
                force,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::KickHosts {
                    hosts: host_ids.split(",").map(|h| h.to_owned()).collect(),
                    force,
                }
            }
            DDevOp::Show { env_name } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Custom(ExtraOp::Show)
            }
            DDevOp::ShowHosts { hosts, json } => {
                Op::Custom(ExtraOp::ShowHosts { hosts, json })
            }
            DDevOp::ListRpcs {
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
            DDevOp::DebugFailedNodes { env_name } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::DebugFailedNodes
            }
            DDevOp::List => Op::List,
            DDevOp::HostPutFile {
                env_name,
                local_path,
                remote_path,
                hosts,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::HostPutFile {
                    local_path,
                    remote_path,
                    hosts: hosts.map(|h| pnk!(parse_cfg(&h))).or_else(env_hosts),
                }
            }
            DDevOp::HostGetFile {
                env_name,
                remote_path,
                local_base_dir,
                hosts,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::HostGetFile {
                    remote_path,
                    local_base_dir,
                    hosts: hosts.map(|h| pnk!(parse_cfg(&h))).or_else(env_hosts),
                }
            }
            DDevOp::HostExec {
                env_name,
                cmd,
                script_path,
                hosts,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::HostExec {
                    cmd,
                    script_path,
                    hosts: hosts.map(|h| pnk!(parse_cfg(&h))).or_else(env_hosts),
                }
            }
            DDevOp::GetLogs {
                env_name,
                local_base_dir,
                nodes,
                failed,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Custom(ExtraOp::GetLogs {
                    local_dir: local_base_dir,
                    nodes,
                    failed,
                })
            }
            DDevOp::DumpVcData {
                env_name,
                local_base_dir,
                nodes,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Custom(ExtraOp::DumpVcData {
                    local_dir: local_base_dir,
                    nodes,
                })
            }
            DDevOp::SwitchELToGeth { env_name, nodes } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                let nodes = nodes
                    .split(',')
                    .map(|s| s.parse::<NodeID>().c(d!()))
                    .collect::<Result<BTreeSet<_>>>();
                Op::Custom(ExtraOp::SwitchELToGeth { nodes: pnk!(nodes) })
            }
            DDevOp::SwitchELToReth { env_name, nodes } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                let nodes = nodes
                    .split(',')
                    .map(|s| s.parse::<NodeID>().c(d!()))
                    .collect::<Result<BTreeSet<_>>>();
                Op::Custom(ExtraOp::SwitchELToReth { nodes: pnk!(nodes) })
            }
        };

        Self {
            sys_cfg: SysCfg { name: en, op },
        }
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

        let prepare_cmd = format!(
            r#"
echo "{rand_jwt}" > {auth_jwt} | tr -d '\n' || exit 1

if [ ! -d {genesis_dir} ]; then
    tar -C {home} -xpf {home}/{NODE_HOME_GENESIS_DST} || exit 1
    if [ ! -d {genesis_dir} ]; then
        mv {home}/$(tar -tf {home}/{NODE_HOME_GENESIS_DST} | head -1) {genesis_dir} || exit 1
    fi
fi "#
        );

        let el_kind = json_el_kind(&n.custom_data);

        let local_ip = &n.host.addr.local_ip;
        let ext_ip = &n.host.addr.connection_addr(); // for `ddev` it should be e.external_ip

        let ts_start = ts!();
        let (el_bootnodes, cl_bn_bootnodes, cl_bn_trusted_peers, checkpoint_sync_url) = loop {
            let online_nodes = e
                .nodes_should_be_online
                .iter()
                .map(|(k, _)| k)
                .filter(|k| *k < n.id) // early nodes only
                .collect::<HashSet<_>>() // for random purpose
                .into_iter()
                .take(5)
                .collect::<Vec<_>>();

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
                        format!(
                            "http://{}:{}",
                            &n.host.addr.connection_addr(),
                            n.ports.el_rpc
                        ),
                        format!(
                            "http://{}:{}",
                            &n.host.addr.connection_addr(),
                            n.ports.cl_bn_rpc
                        ),
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
        // let el_discovery_v5_port = n.ports.el_discovery_v5;
        let el_rpc_port = n.ports.el_rpc;
        let el_rpc_ws_port = n.ports.el_rpc_ws;
        let el_engine_port = n.ports.el_engine_api;
        let el_metric_port = n.ports.el_metric;

        let el_cmd = if Eth1Kind::Geth == el_kind {
            let geth = &e.custom_data.el_geth_bin;

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
        {el_genesis} >>{el_dir}/logs/{EL_LOG_NAME} 2>&1 || exit 1
fi "#
            );

            let cmd_run_part_0 = format!(
                r#"
nohup {geth} \
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
    --http --http.addr={local_ip} --http.port={el_rpc_port} --http.vhosts='*' --http.corsdomain='*' \
    --http.api='admin,debug,eth,net,txpool,web3,rpc' \
    --ws --ws.addr={local_ip} --ws.port={el_rpc_ws_port} --ws.origins='*' \
    --ws.api='admin,debug,eth,net,txpool,web3,rpc' \
    --authrpc.addr={local_ip} --authrpc.port={el_engine_port} \
    --authrpc.jwtsecret={auth_jwt} \
    --metrics \
    --metrics.port={el_metric_port} "#
            );

            let cmd_run_part_1 = if el_bootnodes.is_empty() {
                String::new()
            } else {
                format!(" --bootnodes='{el_bootnodes}'")
            };

            let cmd_run_part_2 = " >>/dev/null 2>&1 &";

            cmd_init_part + &cmd_run_part_0 + &cmd_run_part_1 + cmd_run_part_2
        } else if Eth1Kind::Reth == el_kind {
            let reth = &e.custom_data.el_reth_bin;

            let cmd_init_part = format!(
                r#"
if [ ! -d {el_dir} ]; then
    mkdir -p {el_dir}/logs || exit 1
    {reth} init --datadir={el_dir} --chain={el_genesis} \
        --log.file.directory={el_dir}/logs >>/dev/null 2>&1 || exit 1
    ln -sv {el_dir}/logs/*/reth.log {el_dir}/logs/{EL_LOG_NAME} >/dev/null || exit 1
fi "#
            );

            let cmd_run_part_0 = format!(
                r#"
nohup {reth} node \
    --chain={el_genesis} \
    --datadir={el_dir} \
    --log.file.directory={el_dir}/logs \
    --log.file.max-size=12 \
    --log.file.max-files=20 \
    --ipcdisable \
    --nat=extip:{ext_ip} \
    --port={el_discovery_port} \
    --discovery.port={el_discovery_port} \
    --http --http.addr={local_ip} --http.port={el_rpc_port} --http.corsdomain='*' \
    --http.api='admin,debug,eth,net,txpool,web3,rpc' \
    --ws --ws.addr={local_ip} --ws.port={el_rpc_ws_port} --ws.origins='*' \
    --ws.api='admin,debug,eth,net,txpool,web3,rpc' \
    --authrpc.addr={local_ip} --authrpc.port={el_engine_port} \
    --authrpc.jwtsecret={auth_jwt} \
    --metrics='{local_ip}:{el_metric_port}' "#
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

            let cmd_run_part_2 = " >>/dev/null 2>&1 &";

            cmd_init_part + &cmd_run_part_0 + &cmd_run_part_1 + cmd_run_part_2
        } else {
            pnk!(Err(eg!("The fuhrering world is over!")))
        };

        ////////////////////////////////////////////////
        // CL
        ////////////////////////////////////////////////

        let lighthouse = &e.custom_data.cl_bin;

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

nohup {lighthouse} beacon_node \
    --testnet-dir={cl_genesis} \
    --datadir={cl_bn_dir} \
    --logfile={cl_bn_dir}/logs/{CL_BN_LOG_NAME} \
    --logfile-compress \
    --logfile-max-size=12 \
    --logfile-max-number=20 \
    --staking \
    {reconstruct_states} \
    --subscribe-all-subnets \
    --epochs-per-migration={epochs_per_migration} \
    --slots-per-restore-point={cl_slots_per_rp} \
    --enr-address={ext_ip} \
    --disable-enr-auto-update \
    --disable-upnp \
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

            let cmd_run_part_2 = " >>/dev/null 2>&1 &";

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
    if [[ (! -d '{cl_vc_dir}/validators') && ("" != ${{vcdata_dir_name}}) ]]; then
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

nohup {lighthouse} validator_client \
    --testnet-dir={cl_genesis} \
    --datadir={cl_vc_dir} \
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
    >>/dev/null 2>&1 &
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

    fn cmd_for_migrate(
        &self,
        src: &Node<Ports>,
        dst: &Node<Ports>,
        _e: &EnvMeta<CustomInfo, Node<Ports>>,
    ) -> impl FnOnce() -> Result<()> {
        || {
            let tgz = format!("vcdata_{}.tar.gz", ts!());

            let src_cmd = format!("tar -C /tmp -zcf {tgz} {}/{CL_VC_DIR}", &src.home);
            let src_remote = Remote::from(&src.host);
            src_remote
                .exec_cmd(&src_cmd)
                .c(d!())
                .and_then(|_| src_remote.get_file("/tmp/{tgz}", "/tmp/{tgz}").c(d!()))?;

            let dst_cmd = format!(
                "rm -rf {0}/{CL_VC_DIR} && tar -C {0} -zcf /tmp/{tgz}",
                &src.home
            );
            let dst_remote = Remote::from(&dst.host);
            dst_remote
                .put_file("/tmp/{tgz}", "/tmp/{tgz}")
                .c(d!())
                .and_then(|_| dst_remote.exec_cmd(&dst_cmd).c(d!()))
                .map(|_| ())
        }
    }
}

//////////////////////////////////////////////////
//////////////////////////////////////////////////

fn env_hosts() -> Option<Hosts> {
    if let Ok(json) = env::var("NB_DDEV_HOSTS_JSON") {
        let r = fs::read(json)
            .c(d!())
            .and_then(|b| Hosts::from_json_cfg(&b).c(d!()));
        Some(pnk!(r))
    } else if let Ok(expr) = env::var("NB_DDEV_HOSTS") {
        Some(Hosts::from(&expr))
    } else {
        None
    }
}

fn parse_cfg(json_path_or_expr: &str) -> Result<Hosts> {
    if let Ok(j) = fs::read(json_path_or_expr) {
        Hosts::from_json_cfg(&j).c(d!())
    } else {
        Hosts::from_str(json_path_or_expr).c(d!())
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
    Show,
    ShowHosts {
        hosts: Option<HostExpression>,
        json: bool, /*in JSON format or not*/
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
    GetLogs {
        local_dir: Option<String>,
        nodes: Option<String>, /*specified nodes only, comma separated*/
        failed: bool,          /*failed node only*/
    },
    DumpVcData {
        local_dir: Option<String>,
        nodes: Option<String>, /*specified nodes only, comma separated*/
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

                let el_rpc_endpoint = format!(
                    "http://{}:{}",
                    nodes[0].host.addr.connection_addr(),
                    nodes[0].ports.el_rpc
                );

                for n in nodes.into_iter() {
                    let tmp_dir =
                        format!("/tmp/{}_{}", ts!(), ruc::algo::rand::rand_jwt());
                    omit!(fs::remove_dir_all(&tmp_dir));
                    fs::create_dir_all(&tmp_dir).c(d!())?;

                    let mnemonic_path = format!("{tmp_dir}/mnemonic.txt");

                    let deposits_json = format!("{tmp_dir}/deposits.json");
                    let validators_json = format!("{tmp_dir}/validators.json");
                    let node_validators_json = format!("{}/validators.json", n.home);

                    let mnemonic = create_mnemonic_words();
                    fs::write(&mnemonic_path, &mnemonic).c(d!())?;

                    let node_testnet_dir =
                        format!("{}/{NODE_HOME_GENESIS_DIR_DST}", n.home);
                    let node_vc_data_dir = format!("{}/{CL_VC_DIR}", n.home);
                    let node_vc_api_token =
                        format!("{}/validators/api-token.txt", node_vc_data_dir);

                    // let node_el_rpc_endpoint = format!(
                    //     "http://{}:{}",
                    //     n.host.addr.connection_addr(),
                    //     n.ports.el_rpc
                    // );

                    let node_vc_rpc_endpoint =
                        format!("http://localhost:{}", n.ports.cl_vc_rpc);

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

                    let remote = Remote::from(&n.host);
                    remote
                        .put_file(&validators_json, &node_validators_json)
                        .c(d!())?;
                    let node_cmd = format!(
                        r#"
                        lighthouse validator-manager import \
                            --testnet-dir {node_testnet_dir} \
                            --datadir {node_vc_data_dir} \
                            --validators-file {node_validators_json} \
                            --vc-url {node_vc_rpc_endpoint} \
                            --vc-token {node_vc_api_token} || exit 1
                        rm -f {node_validators_json}
                        "#
                    );
                    remote.exec_cmd(&node_cmd).c(d!())?;

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
                        map! {B mnemonic => (0..*num_per_node as u16).collect() },
                    );

                    env.write_cfg()
                        .c(d!())
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

                let beacon_rpc_endpoint = format!(
                    "http://{}:{}",
                    nodes[0].host.addr.connection_addr(),
                    nodes[0].ports.cl_bn_rpc
                );

                for n in nodes.into_iter() {
                    if let Some(c) = n.custom_data {
                        for (mnemonic, idxs) in c["deposits"].as_object().c(d!())?.iter()
                        {
                            for idx in idxs.as_array().c(d!())?.iter() {
                                let idx = idx.as_u64().c(d!())? as u16;
                                let ret = exit_by_mnemonic(
                                    &beacon_rpc_endpoint,
                                    &testnet_dir,
                                    mnemonic.as_str(),
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
            Self::Show => {
                let env = load_sysenv(en).c(d!())?;
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

                let mut hosts = meta.remove("remote_hosts").unwrap();
                meta.insert(
                    "remote_hosts".to_string(),
                    hosts
                        .as_object_mut()
                        .unwrap()
                        .values_mut()
                        .map(|v| v.take())
                        .collect::<Vec<_>>()
                        .into(),
                );

                println!("{}", pnk!(serde_json::to_string_pretty(&ret)));

                Ok(())
            }
            Self::ShowHosts { hosts, json } => {
                let hosts = pnk!(hosts
                    .as_ref()
                    .map(|h| pnk!(parse_cfg(h)))
                    .or_else(env_hosts));
                if *json {
                    let s = serde_json::to_string_pretty(&hosts).unwrap();
                    println!("{s}");
                } else {
                    let s = hosts
                        .as_ref()
                        .values()
                        .map(|h| {
                            format!(
                                "  {}{}{}{}{}#nb#2222#{}#{}",
                                h.meta.addr.local_network_id,
                                alt!(h.meta.addr.local_network_id.is_empty(), "", "%"),
                                h.meta.addr.local_ip,
                                alt!(h.meta.addr.ext_ip.is_none(), "", "|"),
                                h.meta.addr.ext_ip.as_deref().unwrap_or_default(),
                                h.weight,
                                h.meta.ssh_sk_path.to_str().unwrap_or_default()
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(",\n");
                    println!("\"\n{s}\n\"");
                };
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
                                n.host.addr.connection_addr(),
                                n.ports.el_rpc
                            ));
                        }
                        if *el_web3_ws || default {
                            buf_el_web3_ws.push(format!(
                                "    http://{}:{}",
                                n.host.addr.connection_addr(),
                                n.ports.el_rpc_ws
                            ));
                        }
                        if *el_metric || default {
                            buf_el_metric.push(format!(
                                "    http://{}:{}",
                                n.host.addr.connection_addr(),
                                n.ports.el_metric
                            ));
                        }
                        if *cl_bn || default {
                            buf_cl_bn.push(format!(
                                "    http://{}:{}",
                                n.host.addr.connection_addr(),
                                n.ports.cl_bn_rpc
                            ));
                        }
                        if *cl_bn_metric || default {
                            buf_cl_bn_metric.push(format!(
                                "    http://{}:{}",
                                n.host.addr.connection_addr(),
                                n.ports.cl_bn_metric
                            ));
                        }
                        if *cl_vc || default {
                            buf_cl_vc.push(format!(
                                "    http://{}:{}",
                                n.host.addr.connection_addr(),
                                n.ports.cl_vc_rpc
                            ));
                        }
                        if *cl_vc_metric || default {
                            buf_cl_vc_metric.push(format!(
                                "    http://{}:{}",
                                n.host.addr.connection_addr(),
                                n.ports.cl_vc_metric
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
            Self::GetLogs {
                local_dir,
                nodes,
                failed,
            } => {
                let env = load_sysenv(en).c(d!())?;

                let mut ids = if let Some(s) = nodes {
                    s.split(',')
                        .map(|id| id.parse::<NodeID>().c(d!()))
                        .collect::<Result<Vec<_>>>()
                        .map(Some)?
                } else {
                    None
                };

                let errlist = if ids.is_none() && *failed {
                    let (failed_cases, errlist) = env.collect_failed_nodes();
                    if failed_cases.is_empty() {
                        ids = Some(vec![]);
                    } else {
                        ids = Some(
                            failed_cases.values().flatten().copied().collect::<Vec<_>>(),
                        );
                    }
                    errlist
                } else {
                    vec![]
                };

                env_collect_files(
                    &env,
                    ids.as_deref(),
                    &[
                        &format!("{EL_DIR}/logs/{EL_LOG_NAME}"),
                        &format!("{CL_BN_DIR}/logs/{CL_BN_LOG_NAME}"),
                        &format!("{CL_VC_DIR}/logs/{CL_VC_LOG_NAME}"),
                        "mgmt.log",
                    ],
                    local_dir.as_deref(),
                )
                .c(d!())?;

                if errlist.is_empty() {
                    Ok(())
                } else {
                    Err(eg!(errlist
                        .iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join("\n")))
                }
            }
            Self::DumpVcData { local_dir, nodes } => {
                let ids = if let Some(s) = nodes {
                    s.split(',')
                        .map(|id| id.parse::<NodeID>().c(d!()))
                        .collect::<Result<Vec<_>>>()
                        .map(Some)?
                } else {
                    None
                };
                load_sysenv(en).c(d!()).and_then(|env| {
                    env_collect_tgz(
                        &env,
                        ids.as_deref(),
                        &[CL_VC_DIR],
                        local_dir.as_deref(),
                    )
                    .c(d!())
                })
            }
            Self::SwitchELToGeth { nodes } => {
                let mut env = load_sysenv(en).c(d!())?;

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
                        !json_el_kind_matched(&n.custom_data, Eth1Kind::Geth),
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
                    let remote = Remote::from(&n.host);

                    // Just remove $EL_DIR.
                    // When starting up, if $EL_DIR is detected to not exist,
                    // the new client will re-create it, and sync data from the CL.
                    remote
                        .exec_cmd(&format!("rm -rf {}/{EL_DIR}", n.home))
                        .c(d!())?;

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
                    );
                }

                env.write_cfg().c(d!())
            }
            Self::SwitchELToReth { nodes } => {
                let mut env = load_sysenv(en).c(d!())?;

                let mut ns = vec![];
                for id in nodes.iter() {
                    let n = env
                        .meta
                        .nodes
                        .get(id)
                        .or_else(|| env.meta.fuhrers.get(id))
                        .cloned()
                        .c(d!("The node(id: {}) not found", id))?;
                    alt!(
                        !json_el_kind_matched(&n.custom_data, Eth1Kind::Reth),
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
                    let remote = Remote::from(&n.host);

                    // Just remove $EL_DIR.
                    // When starting up, if $EL_DIR is detected to not exist,
                    // the new client will re-create it, and sync data from the CL.
                    remote
                        .exec_cmd(&format!("rm -rf {}/{EL_DIR}", n.home))
                        .c(d!())?;

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
                    );
                }

                env.write_cfg().c(d!())
            }
        }
    }
}

#[inline(always)]
fn load_sysenv(en: &EnvName) -> Result<SysEnv<CustomInfo, Ports, CmdGenerator>> {
    SysEnv::load_env_by_name(en)
        .c(d!())?
        .c(d!("ENV does not exist!"))
}
