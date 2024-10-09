use chaindev::{beacon_ddev::HostExpression, NodeID};
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
    #[clap(short_flag = 'z', about = "Generate cmdline completions and zsh")]
    GenZshCompletions,
    #[clap(short_flag = 'b', about = "Generate cmdline completions and bash")]
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
        #[clap(short = 'N', long)]
        node_id: Option<NodeID>,
    },
    #[clap(about = "Start all existing ENVs")]
    StartAll,
    #[clap(about = "Stop an existing ENV")]
    Stop {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(short = 'N', long)]
        node_id: Option<NodeID>,
    },
    #[clap(about = "Stop all existing ENVs")]
    StopAll,
    #[clap(about = "Push a new node to an existing ENV")]
    PushNode {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(long, help = "To use reth as the el client, set true")]
        is_reth: bool,
        #[clap(long, help = "To get a archive node, set true")]
        is_archive: bool,
    },
    #[clap(about = "Remove an existing node from an existing ENV")]
    KickNode {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(short = 'N', long)]
        node_id: Option<NodeID>,
    },
    #[clap(about = "Default operation, show the information of an existing ENV")]
    Show {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
    },
    #[clap(about = "Show informations of all existing ENVs")]
    ShowAll,
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
        default_value_t = 3,
        help = "How many initial nodes should be created"
    )]
    pub initial_node_num: u8,

    #[clap(short = 'a', long, help = "Set the initial nodes in archive mode?")]
    pub initial_nodes_archive_mode: bool,

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
        help = "The path of a cfg file in the form of 'https://github.com/NBnet/EGG/blob/master/defaults.env',
has no effect when the `--genesis-data-pre-created` option is specified"
    )]
    pub genesis_custom_settings_path: Option<String>,

    #[clap(
        short = 'G',
        long,
        help = "Concated paths for specifying the pre-created genesis.tar.gz and vcdata.tar.gz,
they are usally created by the `make build` of 'https://github.com/NBnet/EGG',
value format: '/PATH/TO/genesis.tar.gz+/PATH/TO/vcdata.tar.gz',
the `+` is the delimiter between them two"
    )]
    pub genesis_data_pre_created: Option<String>,

    #[clap(long, help = "The path of your custom geth binary")]
    pub el_geth_bin: Option<String>,

    #[clap(long, help = "The path of your custom reth binary")]
    pub el_reth_bin: Option<String>,

    #[clap(
        long,
        allow_hyphen_values = true,
        help = "Custom options you want to add to the geth el cmdline,
NOTE: a pair of quotes should be used when specifying extra options"
    )]
    pub el_geth_extra_options: Option<String>,

    #[clap(
        long,
        allow_hyphen_values = true,
        help = "Custom options you want to add to the reth el cmdline,
NOTE: a pair of quotes should be used when specifying extra options"
    )]
    pub el_reth_extra_options: Option<String>,

    #[clap(long, help = "The path of your custom lighthouse binary")]
    pub cl_bin: Option<String>,

    #[clap(
        long,
        allow_hyphen_values = true,
        help = "Custom options you want to add to the cl cmdline,
NOTE: a pair of quotes should be used when specifying extra options"
    )]
    pub cl_extra_options: Option<String>,

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
        #[clap(short = 'N', long)]
        node_id: Option<NodeID>,
    },
    #[clap(about = "Start all existing ENVs")]
    StartAll,
    #[clap(about = "Stop an existing ENV")]
    Stop {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(short = 'N', long)]
        node_id: Option<NodeID>,
    },
    #[clap(about = "Stop all existing ENVs")]
    StopAll,
    #[clap(about = "Push a new node to an existing ENV")]
    PushNode {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(long)]
        host_addr: Option<String>,
        #[clap(long, help = "To use reth as the el client, set true")]
        is_reth: bool,
        #[clap(long, help = "To get a archive node, set true")]
        is_archive: bool,
    },
    #[clap(about = "Migrate an existing node to another host")]
    MigrateNode {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(short = 'N', long)]
        node_id: NodeID,
        #[clap(short = 'H', long)]
        host_addr: Option<String>,
    },
    #[clap(about = "Remove an existing node from an existing ENV")]
    KickNode {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(short = 'N', long)]
        node_id: Option<NodeID>,
    },
    #[clap(about = "Add a new host to the cluster")]
    PushHost {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(
            short = 'H',
            long,
            help = include_str!("hosts.format")
        )]
        hosts: Option<HostExpression>,
    },
    #[clap(about = "Remove a host from the cluster")]
    KickHost {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(long)]
        host_addr: String,
        #[clap(long)]
        force: bool,
    },
    #[clap(about = "Default operation, show the information of an existing ENV")]
    Show {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
    },
    #[clap(about = "Show informations of all existing ENVs")]
    ShowAll,
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
        #[clap(short = 'c', long, help = "nft commands to be executed")]
        cmd: Option<String>,
        #[clap(
            short = 's',
            long,
            help = "The path of a script to be executed, will be ignored if the 'cmd' field has value"
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
            help = "optional, will use '/tmp' if missing, all remote files will be collected into this directory, <local file name> will be <remote file name> prefixed with its <host address> and <node id>"
        )]
        local_base_dir: Option<String>,
    },
    #[clap(about = "Dump the validator client data from all nodes of the ENV")]
    DumpVcData {
        #[clap(short = 'e', long)]
        env_name: Option<String>,
        #[clap(
            short = 'l',
            long,
            help = "optional, will use '/tmp' if missing, all remote files will be collected into this directory, <local file name> will be <remote file name> prefixed with its <host address> and <node id>"
        )]
        local_base_dir: Option<String>,
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
        default_value_t = 3,
        help = "How many initial nodes should be created"
    )]
    pub initial_node_num: u8,

    #[clap(short = 'a', long, help = "Set the initial nodes in archive mode?")]
    pub initial_nodes_archive_mode: bool,

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
        help = "The path of a cfg file in the form of 'https://github.com/NBnet/EGG/blob/master/defaults.env',
has no effect when the `--genesis-data-pre-created` option is specified"
    )]
    pub genesis_custom_settings_path: Option<String>,

    #[clap(
        short = 'G',
        long,
        help = "Concated paths for specifying the pre-created genesis.tar.gz and vcdata.tar.gz,
they are usally created by the `make build` of 'https://github.com/NBnet/EGG',
value format: '/PATH/TO/genesis.tar.gz+/PATH/TO/vcdata.tar.gz',
the `+` is the delimiter between them two"
    )]
    pub genesis_data_pre_created: Option<String>,

    #[clap(long, help = "The path of your custom geth binary")]
    pub el_geth_bin: Option<String>,

    #[clap(long, help = "The path of your custom reth binary")]
    pub el_reth_bin: Option<String>,

    #[clap(
        long,
        allow_hyphen_values = true,
        help = "Custom options you want to add to the geth el cmdline,
NOTE: a pair of quotes should be used when specifying extra options"
    )]
    pub el_geth_extra_options: Option<String>,

    #[clap(
        long,
        allow_hyphen_values = true,
        help = "Custom options you want to add to the reth el cmdline,
NOTE: a pair of quotes should be used when specifying extra options"
    )]
    pub el_reth_extra_options: Option<String>,

    #[clap(long, help = "The path of your custom consensus layer binary")]
    pub cl_bin: Option<String>,

    #[clap(
        long,
        allow_hyphen_values = true,
        help = "Custom options you want to add to the cl cmdline,
NOTE: a pair of quotes should be used when specifying extra options"
    )]
    pub cl_extra_options: Option<String>,

    #[clap(
        long = "force",
        help = "Try to destroy the target ENV and then recreate it"
    )]
    pub force_create: bool,
}
