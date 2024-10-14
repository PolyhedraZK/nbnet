//!
//! `nbnet ddev` SubCommand
//!
//! The distributed version of `nbnet dev`.
//!

use crate::{
    cfg::{DDevCfg, DDevOp},
    common::*,
};
use chaindev::{
    beacon_ddev::{
        remote::{
            collect_files_from_nodes as env_collect_files,
            collect_tgz_from_nodes as env_collect_tgz, Remote,
        },
        Env as SysEnv, EnvCfg as SysCfg, EnvMeta, EnvOpts as SysOpts, HostAddr, Hosts,
        Node, NodeCmdGenerator, NodeKind, Op, NODE_HOME_GENESIS_DST,
        NODE_HOME_VCDATA_DST,
    },
    CustomOps, EnvName, NodeID,
};
use ruc::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashSet},
    env, fs,
    str::FromStr,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvCfg {
    sys_cfg: SysCfg<CustomInfo, Ports, ExtraOp>,
}

impl EnvCfg {
    pub fn exec(&self) -> Result<()> {
        self.sys_cfg.exec(CmdGenerator).c(d!()).map(|_| ())
    }
}

#[macro_export]
macro_rules! select_nodes_by_el_kind {
    ($node_ids: expr, $geth: expr, $reth: expr, $en: expr, $include_fuhrer_nodes: expr) => {{
        if $node_ids.is_none() && !$geth && !$reth {
            None
        } else if $node_ids.is_some() && !$geth && !$reth {
            let parsed = $node_ids
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
                            n.mark.unwrap_or(GETH_MARK) == GETH_MARK
                        } else if $reth {
                            n.mark.unwrap_or(GETH_MARK) == RETH_MARK
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

            if let Some(s) = $node_ids {
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
    ($node_ids: expr, $geth: expr, $reth: expr, $en: expr) => {{
        select_nodes_by_el_kind!($node_ids, $geth, $reth, $en, true)
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
                    "No hosts registered! Use `--hosts` or $NBNET_DDEV_HOSTS to set."
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
                    initial_node_num: copts.initial_node_num,
                    initial_nodes_fullnode: copts.initial_nodes_fullnode,
                    custom_data,
                    force_create: copts.force_create,
                };

                Op::Create(envopts)
            }
            DDevOp::Destroy { env_name, force } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Destroy(force)
            }
            DDevOp::DestroyAll { force } => Op::DestroyAll(force),
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
                node_ids,
                geth,
                reth,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Start(select_nodes_by_el_kind!(node_ids, geth, reth, en))
            }
            DDevOp::StartAll => Op::StartAll,
            DDevOp::Stop {
                env_name,
                node_ids,
                geth,
                reth,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Stop((select_nodes_by_el_kind!(node_ids, geth, reth, en), false))
            }
            DDevOp::StopAll => Op::StopAll(false),
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
                Op::PushNodes((
                    host_addr.map(|a| pnk!(HostAddr::from_str(&a))),
                    alt!(reth, RETH_MARK, GETH_MARK),
                    fullnode,
                    num,
                ))
            }
            DDevOp::MigrateNodes {
                env_name,
                node_ids,
                host_addr,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                let parsed = node_ids
                    .split(',')
                    .map(|id| id.parse::<NodeID>().c(d!()))
                    .collect::<Result<BTreeSet<_>>>();
                let node_ids = pnk!(parsed, "Invalid ID[s], parse failed");
                Op::MigrateNodes((
                    node_ids,
                    host_addr.map(|a| pnk!(HostAddr::from_str(&a))),
                ))
            }
            DDevOp::KickNodes {
                env_name,
                node_ids,
                num,
                geth,
                reth,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                let ids = select_nodes_by_el_kind!(node_ids, geth, reth, en, false).map(
                    |ids| {
                        let num = num as usize;
                        if ids.len() > num {
                            ids.into_iter().take(num).collect()
                        } else {
                            ids
                        }
                    },
                );
                Op::KickNodes((ids, num))
            }
            DDevOp::PushHosts { env_name, hosts } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                let hosts = pnk!(hosts.map(|h| pnk!(parse_cfg(&h))).or_else(env_hosts));
                Op::PushHosts(hosts)
            }
            DDevOp::KickHosts {
                env_name,
                host_ids,
                force,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::KickHosts((
                    host_ids.split(",").map(|h| h.to_owned()).collect(),
                    force,
                ))
            }
            DDevOp::Show { env_name } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Show
            }
            DDevOp::ListWeb3Rpcs { env_name } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Custom(ExtraOp::ListWeb3Rpcs)
            }
            DDevOp::ShowAll => Op::ShowAll,
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
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Custom(ExtraOp::GetLogs(local_base_dir))
            }
            DDevOp::DumpVcData {
                env_name,
                local_base_dir,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Custom(ExtraOp::DumpVcData(local_base_dir))
            }
            DDevOp::SwitchELToGeth { env_name, node_ids } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                let node_ids = node_ids
                    .split(',')
                    .map(|s| s.parse::<NodeID>().c(d!()))
                    .collect::<Result<BTreeSet<_>>>();
                Op::Custom(ExtraOp::SwitchELToGeth(pnk!(node_ids)))
            }
            DDevOp::SwitchELToReth { env_name, node_ids } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                let node_ids = node_ids
                    .split(',')
                    .map(|s| s.parse::<NodeID>().c(d!()))
                    .collect::<Result<BTreeSet<_>>>();
                Op::Custom(ExtraOp::SwitchELToReth(pnk!(node_ids)))
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
            "ps ax -o pid,args | grep -E '({0}.*{3})|({1}.*{3})|({2}.*{3})' | grep -v 'grep' | wc -l",
            e.custom_data.el_geth_bin, e.custom_data.el_reth_bin, e.custom_data.cl_bin, n.home
        )
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

        let mark = n.mark.unwrap_or(GETH_MARK);

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
        let el_discovery_v5_port = n.ports.el_discovery_v5;
        let el_rpc_port = n.ports.el_rpc;
        let el_rpc_ws_port = n.ports.el_rpc_ws;
        let el_engine_port = n.ports.el_engine_api;
        let el_metric_port = n.ports.el_metric;

        let el_cmd = if GETH_MARK == mark {
            let geth = &e.custom_data.el_geth_bin;

            let el_gc_mode = if matches!(n.kind, NodeKind::FullNode) {
                "full"
            } else {
                "archive" // Fuhrer nodes belong to The ArchiveNode
            };

            let cmd_init_part = format!(
                r#"
if [ ! -d {el_dir} ]; then
    mkdir -p {el_dir} || exit 1
    (which geth; {geth} version; echo) >>{el_dir}/{EL_LOG_NAME} 2>&1
    {geth} init --datadir={el_dir} --state.scheme=hash \
        {el_genesis} >>{el_dir}/{EL_LOG_NAME} 2>&1 || exit 1
fi "#
            );

            let cmd_run_part_0 = format!(
                r#"
(which geth; {geth} version; echo) >>{el_dir}/{EL_LOG_NAME} 2>&1

nohup {geth} \
    --syncmode=full \
    --gcmode={el_gc_mode} \
    --networkid=$(grep -Po '(?<="chainId":)\s*\d+' {el_genesis} | tr -d ' ') \
    --datadir={el_dir} \
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
    --metrics.port={el_metric_port} "#
            );

            let cmd_run_part_1 = if el_bootnodes.is_empty() {
                String::new()
            } else {
                format!(" --bootnodes='{el_bootnodes}'")
            };

            let cmd_run_part_2 = format!(" >>{el_dir}/{EL_LOG_NAME} 2>&1 &");

            cmd_init_part + &cmd_run_part_0 + &cmd_run_part_1 + &cmd_run_part_2
        } else if RETH_MARK == mark {
            let reth = &e.custom_data.el_reth_bin;

            let cmd_init_part = format!(
                r#"
if [ ! -d {el_dir} ]; then
    mkdir -p {el_dir} || exit 1
    (which reth; {reth} --version; echo) >>{el_dir}/{EL_LOG_NAME} 2>&1
    {reth} init --datadir={el_dir} --chain={el_genesis} \
        --log.file.directory={el_dir}/logs >>{el_dir}/{EL_LOG_NAME} 2>&1 || exit 1
fi "#
            );

            let cmd_run_part_0 = format!(
                r#"
(which reth; {reth} --version; echo) >>{el_dir}/{EL_LOG_NAME} 2>&1

nohup {reth} node \
    --chain={el_genesis} \
    --datadir={el_dir} \
    --log.file.directory={el_dir}/logs \
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

            let cmd_run_part_2 = format!(" >>{el_dir}/{EL_LOG_NAME} 2>&1 &");

            cmd_init_part + &cmd_run_part_0 + &cmd_run_part_1 + &cmd_run_part_2
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

        let cl_slots_per_rp = if matches!(n.kind, NodeKind::FullNode) {
            2048
        } else {
            32
        };

        let cl_bn_cmd = {
            let cmd_run_part_0 = format!(
                r#"
mkdir -p {cl_bn_dir} || exit 1
sleep 0.5

(which lighthouse; {lighthouse} --version; echo) >>{cl_bn_dir}/{CL_BN_LOG_NAME} 2>&1

nohup {lighthouse} beacon_node \
    --testnet-dir={cl_genesis} \
    --datadir={cl_bn_dir} \
    --staking \
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

            if checkpoint_sync_url.is_empty() {
                cmd_run_part_1.push_str(" --allow-insecure-genesis-sync");
            } else {
                cmd_run_part_1
                    .push_str(&format!(" --checkpoint-sync-url={checkpoint_sync_url}"));
            }

            let cmd_run_part_2 = format!(" >>{cl_bn_dir}/{CL_BN_LOG_NAME} 2>&1 &");

            cmd_run_part_0 + &cmd_run_part_1 + &cmd_run_part_2
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

nohup {lighthouse} validator_client \
    --testnet-dir={cl_genesis} \
    --datadir={cl_vc_dir}\
    --beacon-nodes='{beacon_nodes}' \
    --init-slashing-protection \
    --suggested-fee-recipient={FEE_RECIPIENT} \
    --unencrypted-http-transport \
    --http --http-address={local_ip} \
    --http-port={cl_vc_rpc_port} --http-allow-origin='*' \
    --metrics --metrics-address={local_ip} \
    --metrics-port={cl_vc_metric_port} --metrics-allow-origin='*' \
     >>{cl_vc_dir}/{CL_VC_LOG_NAME} 2>&1 &
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
    if let Ok(json) = env::var("NBNET_DDEV_HOSTS_JSON") {
        let r = fs::read(json)
            .c(d!())
            .and_then(|b| Hosts::from_json_cfg(&b).c(d!()));
        Some(pnk!(r))
    } else if let Ok(expr) = env::var("NBNET_DDEV_HOSTS") {
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
    ListWeb3Rpcs,
    GetLogs(Option<String>),
    DumpVcData(Option<String>),
    SwitchELToGeth(BTreeSet<NodeID>),
    SwitchELToReth(BTreeSet<NodeID>),
}

impl CustomOps for ExtraOp {
    fn exec(&self, en: &EnvName) -> Result<()> {
        let mut env = load_sysenv(en).c(d!())?;

        match self {
            Self::ListWeb3Rpcs => {
                env.meta
                    .fuhrers
                    .values()
                    .chain(env.meta.nodes.values())
                    .for_each(|n| {
                        println!(
                            " http://{}:{}",
                            n.host.addr.connection_addr(),
                            n.ports.el_rpc
                        );
                    });
                Ok(())
            }
            Self::GetLogs(ldir) => env_collect_files(
                &env,
                &[
                    &format!("{EL_DIR}/{EL_LOG_NAME}"),
                    &format!("{CL_BN_DIR}/{CL_BN_LOG_NAME}"),
                    &format!("{CL_VC_DIR}/{CL_VC_LOG_NAME}"),
                    "mgmt.log",
                ],
                ldir.as_deref(),
            )
            .c(d!()),
            Self::DumpVcData(ldir) => {
                env_collect_tgz(&env, &[CL_VC_DIR], ldir.as_deref()).c(d!())
            }
            Self::SwitchELToGeth(ids) => {
                let mut nodes = vec![];
                for id in ids.iter() {
                    let n = env
                        .meta
                        .nodes
                        .get(id)
                        .or_else(|| env.meta.fuhrers.get(id))
                        .cloned()
                        .c(d!("The node(id: {id}) not found"))?;
                    alt!(n.mark.unwrap_or(GETH_MARK) != GETH_MARK, nodes.push(n));
                }

                SysCfg {
                    name: en.clone(),
                    op: Op::<CustomInfo, Ports, ExtraOp>::Stop((
                        Some(nodes.iter().map(|n| n.id).collect()),
                        false,
                    )),
                }
                .exec(CmdGenerator)
                .c(d!())?;

                sleep_ms!(3000); // wait for the graceful exiting process

                for (i, n) in nodes.iter().enumerate() {
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

                for id in nodes.iter().map(|n| n.id) {
                    env.meta
                        .nodes
                        .get_mut(&id)
                        .or_else(|| env.meta.fuhrers.get_mut(&id))
                        .unwrap()
                        .mark = Some(GETH_MARK);
                }

                env.write_cfg().c(d!())
            }
            Self::SwitchELToReth(ids) => {
                let mut nodes = vec![];
                for id in ids.iter() {
                    let n = env
                        .meta
                        .nodes
                        .get(id)
                        .or_else(|| env.meta.fuhrers.get(id))
                        .cloned()
                        .c(d!("The node(id: {id}) not found"))?;
                    alt!(n.mark.unwrap_or(GETH_MARK) != RETH_MARK, nodes.push(n));
                }

                SysCfg {
                    name: en.clone(),
                    op: Op::<CustomInfo, Ports, ExtraOp>::Stop((
                        Some(nodes.iter().map(|n| n.id).collect()),
                        false,
                    )),
                }
                .exec(CmdGenerator)
                .c(d!())?;

                sleep_ms!(3000); // wait for the graceful exiting process

                for (i, n) in nodes.iter().enumerate() {
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

                for id in nodes.iter().map(|n| n.id) {
                    env.meta
                        .nodes
                        .get_mut(&id)
                        .or_else(|| env.meta.fuhrers.get_mut(&id))
                        .unwrap()
                        .mark = Some(RETH_MARK);
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
