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
        self, EnvMeta, HostAddr, Hosts, Node, NodeCmdGenerator, NodePorts, Op,
    },
    EnvName,
};
use ruc::*;
use serde::{Deserialize, Serialize};
use std::{env, fs, str::FromStr};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvCfg {
    sys_cfg: beacon_ddev::EnvCfg<CustomInfo, Ports, ()>,
}

impl EnvCfg {
    pub fn exec(&self) -> Result<()> {
        self.sys_cfg.exec(CmdGenerator).c(d!()).map(|_| ())
    }
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
                    el_geth_extra_options: copts
                        .el_geth_extra_options
                        .unwrap_or_default(),
                    el_reth_bin: copts.el_reth_bin.unwrap_or("reth".to_owned()),
                    el_reth_extra_options: copts
                        .el_reth_extra_options
                        .unwrap_or_default(),
                    cl_bin: copts.cl_bin.unwrap_or_else(|| "lighthouse".to_owned()),
                    cl_extra_options: copts.cl_extra_options.unwrap_or_default(),
                };

                let envopts = beacon_ddev::EnvOpts {
                    hosts,
                    block_itv: copts.block_time_secs.unwrap_or(0),
                    genesis_pre_settings: copts
                        .genesis_custom_settings_path
                        .unwrap_or_default(),
                    genesis_tgz_path,
                    genesis_vkeys_tgz_path,
                    initial_node_num: copts.initial_node_num,
                    initial_nodes_archive_mode: copts.initial_nodes_archive_mode,
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
            DDevOp::Start { env_name, node_id } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Start(node_id)
            }
            DDevOp::StartAll => Op::StartAll,
            DDevOp::Stop { env_name, node_id } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Stop((node_id, false))
            }
            DDevOp::StopAll => Op::StopAll(false),
            DDevOp::PushNode {
                env_name,
                host_addr,
                is_reth,
                is_archive,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::PushNode((
                    host_addr.map(|a| pnk!(HostAddr::from_str(&a))),
                    alt!(is_reth, 1, 0),
                    is_archive,
                ))
            }
            DDevOp::MigrateNode {
                env_name,
                node_id,
                host_addr,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::MigrateNode((
                    node_id,
                    host_addr.map(|a| pnk!(HostAddr::from_str(&a))),
                ))
            }
            DDevOp::KickNode { env_name, node_id } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::KickNode(node_id)
            }
            DDevOp::PushHost { env_name, hosts } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                let hosts = pnk!(hosts.map(|h| h.into()).or_else(env_hosts));
                Op::PushHost(hosts)
            }
            DDevOp::KickHost {
                env_name,
                host_addr,
                force,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::KickHost((pnk!(HostAddr::from_str(&host_addr)), force))
            }
            DDevOp::Show { env_name } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Show
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
                    hosts: hosts.map(|h| h.into()).or_else(env_hosts),
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
                    hosts: hosts.map(|h| h.into()).or_else(env_hosts),
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
                    hosts: hosts.map(|h| h.into()).or_else(env_hosts),
                }
            }
            DDevOp::GetLogs {
                env_name,
                local_base_dir,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                todo!()
            }
            DDevOp::GetCfgs {
                env_name,
                local_base_dir,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                todo!()
            }
        };

        Self {
            sys_cfg: beacon_ddev::EnvCfg { name: en, op },
        }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
struct CmdGenerator;

impl NodeCmdGenerator<Node<Ports>, EnvMeta<CustomInfo, Node<Ports>>> for CmdGenerator {
    fn cmd_is_running(
        &self,
        n: &Node<Ports>,
        e: &EnvMeta<CustomInfo, Node<Ports>>,
    ) -> Result<bool> {
        todo!()
    }

    fn cmd_for_start(
        &self,
        n: &Node<Ports>,
        e: &EnvMeta<CustomInfo, Node<Ports>>,
    ) -> String {
        todo!()
    }

    fn cmd_for_stop(
        &self,
        n: &Node<Ports>,
        e: &EnvMeta<CustomInfo, Node<Ports>>,
        force: bool,
    ) -> String {
        todo!()
    }

    fn cmd_for_migrate_in(
        &self,
        src: &Node<Ports>,
        dst: &Node<Ports>,
        e: &EnvMeta<CustomInfo, Node<Ports>>,
    ) -> String {
        todo!()
    }

    fn cmd_for_migrate_out(
        &self,
        src: &Node<Ports>,
        dst: &Node<Ports>,
        e: &EnvMeta<CustomInfo, Node<Ports>>,
    ) -> String {
        todo!()
    }
}

//////////////////////////////////////////////////
//////////////////////////////////////////////////

fn env_hosts() -> Option<Hosts> {
    env::var("NBNET_DDEV_HOSTS")
        .c(d!())
        .map(|s| Hosts::from(&s))
        .ok()
}

//////////////////////////////////////////////////
//////////////////////////////////////////////////
