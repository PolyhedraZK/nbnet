use chaindev::common::hosts::HostExpression;
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Cfg {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(about = "Manage development clusters on a local host")]
    Dev(DevCfg),
    #[clap(
        name = "ddev",
        about = "Manage development clusters on various distributed hosts"
    )]
    DDev(DDevCfg),
    #[clap(
        short_flag = 'z',
        about = "Generate the cmdline completion script for zsh"
    )]
    GenZshCompletions,
    #[clap(
        short_flag = 'b',
        about = "Generate the cmdline completion script for bash"
    )]
    GenBashCompletions,
}

#[derive(Debug, Args)]
pub struct DevCfg {
    #[clap(short = 'e', long)]
    pub env_name: Option<String>,

    #[clap(subcommand)]
    pub op: Option<DevOp>,
}

#[derive(Debug, Subcommand)]
pub enum DevOp {
    #[clap(about = "Create a new ENV")]
    Create(Box<DevCreationOptions>),
    #[clap(about = "Destroy an existing ENV")]
    Destroy {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(long, help = "Destroy the target ENV even if it is protected")]
        force: bool,
    },
    #[clap(about = "Destroy all existing ENVs")]
    DestroyAll {
        #[clap(long, help = "Destroy the target ENVs even if they are protected")]
        force: bool,
    },
    #[clap(about = "Protect an existing ENV")]
    Protect {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
    },
    #[clap(about = "Unprotect an existing ENV")]
    Unprotect {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
    },
    #[clap(about = "Start an existing ENV")]
    Start {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(
            short = 'N',
            long,
            help = "Comma separated NodeID[s], '3', '3,2,1', etc."
        )]
        node_ids: Option<String>,
        #[clap(long, help = "Filter nodes with the geth el")]
        geth: bool,
        #[clap(long, help = "Filter nodes with the reth el")]
        reth: bool,
        #[clap(short = 'I', long, help = "Ignore failed cases and continue")]
        ignore_failed: bool,
    },
    #[clap(about = "Start all existing ENVs")]
    StartAll,
    #[clap(about = "Stop an existing ENV")]
    Stop {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(
            short = 'N',
            long,
            help = "Comma separated NodeID[s], '3', '3,2,1', etc."
        )]
        node_ids: Option<String>,
        #[clap(long, help = "Filter nodes with the geth el")]
        geth: bool,
        #[clap(long, help = "Filter nodes with the reth el")]
        reth: bool,
    },
    #[clap(about = "Stop all existing ENVs")]
    StopAll,
    #[clap(about = "Push some new nodes to an existing ENV")]
    PushNodes {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(
            conflicts_with = "fullnode",
            long,
            help = "To use reth as the el client, set true;
NOTE: the fullnode mode of `reth` is unstable, do NOT use it"
        )]
        reth: bool,
        #[clap(conflicts_with = "reth", long, help = "To get FullNode[s], set true")]
        fullnode: bool,
        #[clap(
            short = 'n',
            long,
            default_value_t = 1,
            help = "How many new node[s] to add"
        )]
        num: u8,
    },
    #[clap(about = "Remove(destroy) some nodes from an existing ENV")]
    KickNodes {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(
            short = 'N',
            long,
            help = "Comma separated NodeID[s], '3', '3,2,1', etc."
        )]
        node_ids: Option<String>,
        #[clap(
            short = 'n',
            long,
            default_value_t = 1,
            help = "How many node[s] to kick"
        )]
        num: u8,
        #[clap(long, help = "Filter nodes with the geth el")]
        geth: bool,
        #[clap(long, help = "Filter nodes with the reth el")]
        reth: bool,
    },
    #[clap(
        name = "switch-EL-to-geth",
        about = "Switch the EL client to `geth`,
NOTE: the node will be left stopped, a `start` operation may be needed"
    )]
    SwitchELToGeth {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(
            short = 'N',
            long,
            help = "Comma separated NodeID[s], 'HostID', 'HostID,HostID', etc."
        )]
        node_ids: String,
    },
    #[clap(
        name = "switch-EL-to-reth",
        about = "Switch the EL client to `reth`,
NOTE: the node will be left stopped, a `start` operation may be needed"
    )]
    SwitchELToReth {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(
            short = 'N',
            long,
            help = "Comma separated NodeID[s], 'HostID', 'HostID,HostID', etc."
        )]
        node_ids: String,
    },
    #[clap(about = "Default operation, show the information of an existing ENV")]
    Show {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
    },
    #[clap(about = "Show informations of all existing ENVs")]
    ShowAll,
    #[clap(about = "Show failed nodes in a list")]
    DebugFailedNodes {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
    },
    #[clap(
        short_flag = 'w',
        about = "List all web3 RPC endpoints of the entire ENV"
    )]
    ListWeb3Rpcs {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
    },
    #[clap(about = "Show names of all existing ENVs")]
    List,
}

impl Default for DevOp {
    fn default() -> Self {
        Self::Show { env_name: None }
    }
}

#[derive(Debug, Args)]
pub struct DevCreationOptions {
    #[clap(short = 'e', long)]
    pub env_name: Option<String>,

    #[clap(
        short = 'H',
        long,
        default_value_t = String::from("127.0.0.1"),
        help = "Usually need not to specify")
    ]
    pub host_ip: String,

    #[clap(
        short = 'n',
        long,
        default_value_t = 0,
        help = "How many extra nodes(exclude the fuhrer node) should be created,
the actual node number will be `1 + this_value`"
    )]
    pub extra_node_num: u8,

    #[clap(
        long,
        help = "Set extra nodes in FullNode(opposite to ArchiveNode) mode?"
    )]
    pub fullnode: bool,

    #[clap(
        short = 't',
        long,
        help = "If not set, use the default value in the genesis,
has no effect when the `--genesis-data-pre-created` option is specified"
    )]
    pub block_time_secs: Option<u16>,

    #[clap(
        short = 'g',
        long,
        help = "The path of a cfg file in the form of
'https://github.com/rust-util-collections/EGG/blob/master/defaults.env',
has no effect when the `--genesis-data-pre-created` option is specified"
    )]
    pub genesis_custom_settings_path: Option<String>,

    #[clap(
        short = 'G',
        long,
        help = "Concated paths for specifying the pre-created genesis.tar.gz and vcdata.tar.gz,
they are usally created by the `make build` of 'https://github.com/rust-util-collections/EGG',
value format: '/PATH/TO/genesis.tar.gz+/PATH/TO/vcdata.tar.gz',
the `+` is the delimiter between them two"
    )]
    pub genesis_data_pre_created: Option<String>,

    #[clap(long, help = "The path of your custom geth binary")]
    pub el_geth_bin: Option<String>,

    #[clap(long, help = "The path of your custom reth binary")]
    pub el_reth_bin: Option<String>,

    #[clap(long, help = "The path of your custom lighthouse binary")]
    pub cl_bin: Option<String>,

    #[clap(
        long = "force",
        help = "Try to destroy the target ENV and then recreate it"
    )]
    pub force_create: bool,
}

#[derive(Debug, Args)]
pub struct DDevCfg {
    #[clap(short = 'e', long)]
    pub env_name: Option<String>,

    #[clap(subcommand)]
    pub op: Option<DDevOp>,
}

#[derive(Debug, Subcommand)]
pub enum DDevOp {
    #[clap(about = "Create a new ENV")]
    Create(Box<DDevCreationOptions>),
    #[clap(about = "Destroy an existing ENV")]
    Destroy {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(long, help = "Destroy the target ENV even if it is protected")]
        force: bool,
    },
    #[clap(about = "Destroy all existing ENVs")]
    DestroyAll {
        #[clap(long, help = "Destroy the target ENVs even if they are protected")]
        force: bool,
    },
    #[clap(about = "Protect an existing ENV")]
    Protect {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
    },
    #[clap(about = "Unprotect an existing ENV")]
    Unprotect {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
    },
    #[clap(about = "Start an existing ENV")]
    Start {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(
            short = 'N',
            long,
            help = "Comma separated NodeID[s], '3', '3,2,1', etc."
        )]
        node_ids: Option<String>,
        #[clap(long, help = "Filter nodes with the geth el")]
        geth: bool,
        #[clap(long, help = "Filter nodes with the reth el")]
        reth: bool,
        #[clap(short = 'I', long, help = "Ignore failed cases and continue")]
        ignore_failed: bool,
        #[clap(short = 'R', long, help = "Try to realloc ports when necessary")]
        realloc_ports: bool,
    },
    #[clap(about = "Start all existing ENVs")]
    StartAll,
    #[clap(about = "Stop an existing ENV")]
    Stop {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(
            short = 'N',
            long,
            help = "Comma separated NodeID[s], '3', '3,2,1', etc."
        )]
        node_ids: Option<String>,
        #[clap(long, help = "Filter nodes with the geth el")]
        geth: bool,
        #[clap(long, help = "Filter nodes with the reth el")]
        reth: bool,
    },
    #[clap(about = "Stop all existing ENVs")]
    StopAll,
    #[clap(about = "Push some new nodes to an existing ENV")]
    PushNodes {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(long)]
        host_addr: Option<String>,
        #[clap(
            conflicts_with = "fullnode",
            long,
            help = "To use reth as the el client, set true;
NOTE: the fullnode mode of `reth` is unstable, do NOT use it"
        )]
        reth: bool,
        #[clap(conflicts_with = "reth", long, help = "To get a FullNode, set true")]
        fullnode: bool,
        #[clap(
            short = 'n',
            long,
            default_value_t = 1,
            help = "How many new node[s] to add"
        )]
        num: u8,
    },
    #[clap(about = "Migrate some existing nodes to other hosts,
NOTE: the 'new' node will be left stopped, a `start` operation may be needed")]
    MigrateNodes {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(
            short = 'N',
            long,
            help = "Comma separated NodeID[s], '3', '3,2,1', etc."
        )]
        node_ids: String,
        #[clap(short = 'H', long)]
        host_addr: Option<String>,
    },
    #[clap(about = "Remove(destroy) some node from an existing ENV")]
    KickNodes {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(
            short = 'N',
            long,
            help = "Comma separated NodeID[s], '3', '3,2,1', etc."
        )]
        node_ids: Option<String>,
        #[clap(
            short = 'n',
            long,
            default_value_t = 1,
            help = "How many node[s] to kick"
        )]
        num: u8,
        #[clap(long, help = "Filter nodes with the geth el")]
        geth: bool,
        #[clap(long, help = "Filter nodes with the reth el")]
        reth: bool,
    },
    #[clap(
        name = "switch-EL-to-geth",
        about = "Switch the EL client to `geth`,
NOTE: the node will be left stopped, a `start` operation may be needed"
    )]
    SwitchELToGeth {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(
            short = 'N',
            long,
            help = "Comma separated NodeID[s], 'HostID', 'HostID,HostID', etc."
        )]
        node_ids: String,
    },
    #[clap(
        name = "switch-EL-to-reth",
        about = "Switch the EL client to `reth`,
NOTE: the node will be left stopped, a `start` operation may be needed"
    )]
    SwitchELToReth {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(
            short = 'N',
            long,
            help = "Comma separated NodeID[s], 'HostID', 'HostID,HostID', etc."
        )]
        node_ids: String,
    },
    #[clap(about = "Add some new hosts to the cluster")]
    PushHosts {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(
            short = 'H',
            long,
            help = include_str!("hosts.format")
        )]
        hosts: Option<HostExpression>,
    },
    #[clap(about = "Remove some hosts from the cluster")]
    KickHosts {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(
            short = 'N',
            long,
            help = "Comma separated NodeID[s], 'HostID', 'HostID,HostID', etc."
        )]
        host_ids: String,
        #[clap(long)]
        force: bool,
    },
    #[clap(about = "Default operation, show the information of an existing ENV")]
    Show {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
    },
    #[clap(
        about = "Show the remote host configations in JSON or the `nb` native format"
    )]
    ShowHosts {
        #[clap(
            short = 'H',
            long,
            help = include_str!("hosts.format")
        )]
        hosts: Option<HostExpression>,
        #[clap(long, help = "Show results in the JSON format")]
        json: bool,
    },
    #[clap(about = "Show informations of all existing ENVs")]
    ShowAll,
    #[clap(about = "Show failed nodes in a list")]
    DebugFailedNodes {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
    },
    #[clap(
        short_flag = 'w',
        about = "List all web3 RPC endpoints of the entire ENV"
    )]
    ListWeb3Rpcs {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
    },
    #[clap(about = "Show names of all existing ENVs")]
    List,
    #[clap(about = "Put a local file to all remote hosts")]
    HostPutFile {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(short = 'l', long)]
        local_path: String,
        #[clap(
            short = 'r',
            long,
            help = "optional, will use the value of 'local_path' if missing"
        )]
        remote_path: Option<String>,
        #[clap(
            short = 'H',
            long,
            help = include_str!("hosts.format"),
        )]
        hosts: Option<HostExpression>,
    },
    #[clap(about = "Get a remote file from all remote hosts")]
    HostGetFile {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(short = 'r', long)]
        remote_path: String,
        #[clap(
            short = 'l',
            long,
            help = "optional, will use '/tmp' if missing, all remote files will be collected into this directory, <local file name> will be <remote file name> prefixed with its <host address>"
        )]
        local_base_dir: Option<String>,
        #[clap(
            short = 'H',
            long,
            help = include_str!("hosts.format")
        )]
        hosts: Option<HostExpression>,
    },
    #[clap(about = "Execute commands on all remote hosts")]
    HostExec {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(
            short = 'c',
            long,
            help = "Commands to be executed on the remote hosts"
        )]
        cmd: Option<String>,
        #[clap(
            short = 's',
            long,
            help = "The path of a script to be executed,
will be ignored if the 'cmd' field has value"
        )]
        script_path: Option<String>,
        #[clap(
            short = 'H',
            long,
            help = include_str!("hosts.format")
        )]
        hosts: Option<HostExpression>,
    },
    #[clap(about = "Get the remote logs from all nodes of the ENV")]
    GetLogs {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(
            short = 'l',
            long,
            help = "optional, will use '/tmp' if missing,
all remote files will be collected into this directory,
<local file name> will be <remote file name> prefixed with its <host address> and <node id>"
        )]
        local_base_dir: Option<String>,
        #[clap(
            conflicts_with = "failed",
            short = 'N',
            long,
            help = "Comma separated NodeID[s], '3', '3,2,1', etc."
        )]
        node_ids: Option<String>,
        #[clap(
            conflicts_with = "node_ids",
            long,
            help = "Get logs of the failed nodes only"
        )]
        failed: bool,
    },
    #[clap(about = "Dump the validator client data from all nodes of the ENV")]
    DumpVcData {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(
            short = 'l',
            long,
            help = "optional, will use '/tmp' if missing,
all remote files will be collected into this directory,
<local file name> will be <remote file name> prefixed with its <host address> and <node id>"
        )]
        local_base_dir: Option<String>,
        #[clap(
            short = 'N',
            long,
            help = "Comma separated NodeID[s], '3', '3,2,1', etc."
        )]
        node_ids: Option<String>,
    },
}

impl Default for DDevOp {
    fn default() -> Self {
        Self::Show { env_name: None }
    }
}

#[derive(Debug, Args)]
pub struct DDevCreationOptions {
    #[clap(short = 'e', long)]
    pub env_name: Option<String>,

    #[clap(
        long,
        help = include_str!("hosts.format")
    )]
    pub hosts: Option<HostExpression>,

    #[clap(
        short = 'n',
        long,
        default_value_t = 0,
        help = "How many extra nodes(exclude the fuhrer node) should be created,
the actual node number will be `1 + this_value`"
    )]
    pub extra_node_num: u8,

    #[clap(
        long,
        help = "Set extra nodes in FullNode(opposite to ArchiveNode) mode?"
    )]
    pub fullnode: bool,

    #[clap(
        short = 't',
        long,
        help = "If not set, use the default value in the genesis,
has no effect when the `--genesis-data-pre-created` option is specified"
    )]
    pub block_time_secs: Option<u16>,

    #[clap(
        short = 'g',
        long,
        help = "The path of a cfg file in the form of
'https://github.com/rust-util-collections/EGG/blob/master/defaults.env',
has no effect when the `--genesis-data-pre-created` option is specified"
    )]
    pub genesis_custom_settings_path: Option<String>,

    #[clap(
        short = 'G',
        long,
        help = "Concated paths for specifying the pre-created genesis.tar.gz and vcdata.tar.gz,
they are usally created by the `make build` of 'https://github.com/rust-util-collections/EGG',
value format: '/PATH/TO/genesis.tar.gz+/PATH/TO/vcdata.tar.gz',
the `+` is the delimiter between them two"
    )]
    pub genesis_data_pre_created: Option<String>,

    #[clap(long, help = "The path of your custom geth binary")]
    pub el_geth_bin: Option<String>,

    #[clap(long, help = "The path of your custom reth binary")]
    pub el_reth_bin: Option<String>,

    #[clap(long, help = "The path of your custom consensus layer binary")]
    pub cl_bin: Option<String>,

    #[clap(
        long = "force",
        help = "Try to destroy the target ENV and then recreate it"
    )]
    pub force_create: bool,
}
