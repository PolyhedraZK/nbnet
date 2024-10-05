use chaindev::beacon_dev::NodePorts;
use ruc::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomInfo {
    pub el_bin: String,
    pub el_is_reth: bool,
    pub el_extra_options: String,
    pub cl_bin: String,
    pub cl_is_prysm: bool,
    pub cl_extra_options: String,
}

impl Default for CustomInfo {
    fn default() -> Self {
        Self {
            el_bin: String::from("geth"),
            el_is_reth: false,
            el_extra_options: String::from(""),
            cl_bin: String::from("lighthouse"),
            cl_is_prysm: false,
            cl_extra_options: String::from(""),
        }
    }
}

impl CustomInfo {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Active ports of a node
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct Ports {
    el_discovery: u16,
    el_discovery_v5: u16, // reth only
    el_engine_api: u16,
    el_rpc: u16,
    el_metric: u16, // geth only

    cl_discovery: u16,
    cl_discovery_quic: u16,
    cl_bn_rpc: u16,
    cl_vc_rpc: u16,
    cl_bn_metric: u16,
    cl_vc_metric: u16,
}

impl NodePorts for Ports {
    fn try_create(ports: &[u16]) -> Result<Self> {
        if ports.len() != Self::reserved().len() {
            return Err(eg!("invalid length"));
        }

        let i = Self {
            el_discovery: ports[0],
            el_discovery_v5: ports[1], // reth only
            el_engine_api: ports[2],
            el_rpc: ports[3],
            el_metric: ports[4], // geth only

            cl_discovery: ports[5],
            cl_discovery_quic: ports[6],
            cl_bn_rpc: ports[7],
            cl_vc_rpc: ports[8],
            cl_bn_metric: ports[9],
            cl_vc_metric: ports[10],
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
    /// eg. ETH el(geth/reth) web3 API rpc
    fn get_el_rpc(&self) -> u16 {
        self.el_rpc
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
