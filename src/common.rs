use chaindev::beacon_based::common::{NodeMark, NodePorts};
use rayon::prelude::*;
use ruc::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;

pub const GETH_MARK: NodeMark = 0;
pub const RETH_MARK: NodeMark = 1;

pub const EL_DIR: &str = "el";
pub const CL_BN_DIR: &str = "cl/bn";
pub const CL_VC_DIR: &str = "cl/vc";

pub const EL_LOG_NAME: &str = "el.log";
pub const CL_BN_LOG_NAME: &str = "cl.bn.log";
pub const CL_VC_LOG_NAME: &str = "cl.vc.log";

// TODO: fix me
pub const FEE_RECIPIENT: &str = "0x47102e476Bb96e616756ea7701C227547080Ea48";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomInfo {
    pub el_geth_bin: String,
    pub el_reth_bin: String,
    pub cl_bin: String,
}

impl Default for CustomInfo {
    fn default() -> Self {
        Self {
            el_geth_bin: String::from("geth"),
            el_reth_bin: String::from("reth"),
            cl_bin: String::from("lighthouse"),
        }
    }
}

// impl CustomInfo {
//     pub fn new() -> Self {
//         Self::default()
//     }
// }

/// Active ports of a node
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct Ports {
    pub el_discovery: u16,
    pub el_discovery_v5: u16, // reth only
    pub el_engine_api: u16,
    pub el_rpc: u16,
    pub el_rpc_ws: u16,
    pub el_metric: u16,

    pub cl_discovery: u16,
    pub cl_discovery_quic: u16,
    pub cl_bn_rpc: u16,
    pub cl_vc_rpc: u16,
    pub cl_bn_metric: u16,
    pub cl_vc_metric: u16,
}

impl NodePorts for Ports {
    // Reserve wide-used ports for the default node
    //
    // - lighthouse bn(discovery port): 9000
    // - lighthouse bn(quic port): 9001
    // - lighthouse bn(http rpc): 5052
    // - lighthouse vc(http rpc): 5062
    // - lighthouse bn(prometheus metrics): 5054
    // - lighthouse vc(prometheus metrics): 5064
    fn cl_reserved() -> Vec<u16> {
        vec![9000, 9001, 5052, 5062, 5054, 5064]
    }

    // Reserved ports defined by the Execution Client
    //
    // - geth/reth(discovery port): 30303
    // - reth(discovery v5 port): 9200
    // - geth/reth(engine api): 8551
    // - geth/reth(web3 rpc): 8545, 8546
    // - geth(prometheus metrics): 6060
    fn el_reserved() -> Vec<u16> {
        vec![30303, 9200, 8551, 8545, 8546, 6060]
    }

    fn try_create(ports: &[u16]) -> Result<Self> {
        if ports.len() != Self::reserved().len() {
            return Err(eg!("invalid length"));
        }

        let i = Self {
            el_discovery: ports[0],
            el_discovery_v5: ports[1], // reth only
            el_engine_api: ports[2],
            el_rpc: ports[3],
            el_rpc_ws: ports[4],
            el_metric: ports[5],

            cl_discovery: ports[6],
            cl_discovery_quic: ports[7],
            cl_bn_rpc: ports[8],
            cl_vc_rpc: ports[9],
            cl_bn_metric: ports[10],
            cl_vc_metric: ports[11],
        };

        Ok(i)
    }

    /// Get all actual ports from the instance,
    /// all: <sys ports> + <app ports>
    fn get_port_list(&self) -> Vec<u16> {
        vec![
            self.el_discovery,
            self.el_discovery_v5,
            self.el_engine_api,
            self.el_rpc,
            self.el_rpc_ws,
            self.el_metric,
            self.cl_discovery,
            self.cl_discovery_quic,
            self.cl_bn_rpc,
            self.cl_vc_rpc,
            self.cl_bn_metric,
            self.cl_vc_metric,
        ]
    }

    /// The p2p listening port in the execution side,
    /// may be used in generating the enode address for an execution node
    fn get_el_p2p(&self) -> u16 {
        self.el_discovery
    }

    /// The engine API listening port in the execution side
    /// usage(beacon): `--execution-endpoints="http://localhost:8551"`
    fn get_el_engine_api(&self) -> u16 {
        self.el_engine_api
    }

    /// The rpc listening port in the app side,
    /// eg. ETH el(geth/reth) web3 http API rpc
    fn get_el_rpc(&self) -> u16 {
        self.el_rpc
    }

    /// The rpc listening port in the app side,
    /// eg. ETH el(geth/reth) web3 websocket API rpc
    fn get_el_rpc_ws(&self) -> u16 {
        self.el_rpc_ws
    }

    /// The p2p(tcp/udp protocol) listening port in the beacon side
    /// may be used in generating the ENR address for a beacon node
    fn get_cl_p2p_bn(&self) -> u16 {
        self.cl_discovery
    }

    /// The p2p(quic protocol) listening port in the beacon side
    /// may be used in generating the ENR address for a beacon node
    fn get_cl_p2p_bn_quic(&self) -> u16 {
        self.cl_discovery_quic
    }

    /// The rpc listening port in the beacon side,
    /// usage(beacon): `--checkpoint-sync-url="http://${peer_ip}:5052"`
    fn get_cl_rpc_bn(&self) -> u16 {
        self.cl_bn_rpc
    }

    /// The rpc listening port in the vc side
    fn get_cl_rpc_vc(&self) -> u16 {
        self.cl_vc_rpc
    }
}

/// Return: "enode,enode,enode"
pub fn el_get_boot_nodes(rpc_endpoints: &[&str]) -> Result<String> {
    let ret = rpc_endpoints
        .par_iter()
        .map(|addr| {
            let body =
                r#"{"jsonrpc":"2.0","method":"admin_nodeInfo","params":[],"id":1}"#;

            ruc::http::post(
                addr,
                body.as_bytes(),
                Some(&[("Content-Type", "application/json")]),
            )
            .c(d!())
            .and_then(|(_code, resp)| serde_json::from_slice::<Value>(&resp).c(d!()))
            .map(|v| pnk!(v["result"]["enode"].as_str()).to_owned())
        })
        .filter(|i| i.is_ok())
        .collect::<Result<Vec<_>>>()
        .unwrap();

    if ret.is_empty() {
        return Err(eg!("No valid data return"));
    }

    Ok(ret.join(","))
}

/// Return: "(<enr,enr,enr>,<peer_id,peer_id,peer_id>)"
pub fn cl_get_boot_nodes(
    rpc_endpoints: &[&str],
) -> Result<(Vec<String>, String, String)> {
    let ret: (Vec<_>, (Vec<_>, Vec<_>)) = rpc_endpoints
        .par_iter()
        .map(|url| {
            ruc::http::get(
                &format!("{url}/eth/v1/node/identity"),
                Some(&[("Content-Type", "application/json")]),
            )
            .c(d!())
            .and_then(|(_code, resp)| serde_json::from_slice::<Value>(&resp).c(d!()))
            .map(|v| {
                (
                    url.to_string(),
                    (
                        pnk!(v["data"]["enr"].as_str()).to_owned(),
                        pnk!(v["data"]["peer_id"].as_str()).to_owned(),
                    ),
                )
            })
        })
        .filter(|i| i.is_ok())
        .map(|i| i.unwrap())
        .unzip();

    if ret.0.is_empty() {
        return Err(eg!("No valid data return"));
    }

    Ok((ret.0, ret.1 .0.join(","), ret.1 .1.join(",")))
}

pub fn node_sync_from_genesis() -> bool {
    env::var("NBNET_NODE_SYNC_FROM_GENESIS").is_ok()
}
