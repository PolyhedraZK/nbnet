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
};
use chaindev::{
    beacon_dev::{self, EnvMeta, Node, NodeCmdGenerator, NodePorts, Op},
    EnvName,
};
use ruc::*;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvCfg {
    sys_cfg: beacon_dev::EnvCfg<CustomInfo, Ports, ()>,
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

                if let Some(n) = copts.env_name {
                    en = n.into();
                }

                let envopts = beacon_dev::EnvOpts {
                    host_ip: copts.host_ip,
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
            DevOp::Destroy { env_name, force } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Destroy(force)
            }
            DevOp::DestroyAll { force } => Op::DestroyAll(force),
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
            DevOp::Start { env_name, node_id } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Start(node_id)
            }
            DevOp::StartAll => Op::StartAll,
            DevOp::Stop { env_name, node_id } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Stop((node_id, false))
            }
            DevOp::StopAll => Op::StopAll(false),
            DevOp::PushNode {
                env_name,
                is_reth,
                is_archive,
            } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::PushNode((alt!(is_reth, 1, 0), is_archive))
            }
            DevOp::KickNode { env_name, node_id } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::KickNode(node_id)
            }
            DevOp::Show { env_name } => {
                if let Some(n) = env_name {
                    en = n.into();
                }
                Op::Show
            }
            DevOp::ShowAll => Op::ShowAll,
            DevOp::List => Op::List,
        };

        Self {
            sys_cfg: beacon_dev::EnvCfg { name: en, op },
        }
    }
}

impl EnvCfg {
    pub fn exec(&self) -> Result<()> {
        self.sys_cfg.exec(CmdGenerator).c(d!()).map(|_| ())
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
        todo!()
    }

    fn cmd_for_stop(
        &self,
        n: &Node<Ports>,
        e: &EnvMeta<CustomInfo, Node<Ports>>,
        force: bool,
    ) -> String {
        format!(
            "for i in \
            $(ps ax -o pid,args|grep '{}'|sed -r 's/(^ *)|( +)/ /g'|cut -d ' ' -f 2); \
            do kill {} $i; done",
            &n.home,
            alt!(force, "-9", ""),
        )
    }
}
