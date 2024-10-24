# `nb ddev`

This is a distributed version of [`nb dev`](../dev/README.md).

If you have a deep understanding of the `nb dev`, it will be very helpful for `nb ddev`, so it is highly recommended that you read [the documentation of `nb dev`](../dev/README.md) first.

#### Quick start

Through a `nb ddev -h` we can see:

```
Manage development clusters on various distributed hosts

Usage: nb ddev [OPTIONS] [COMMAND]

Commands:
  create              Create a new ENV
  destroy             Destroy an existing ENV
  destroy-all         Destroy all existing ENVs
  protect             Protect an existing ENV
  unprotect           Unprotect an existing ENV
  start               Start an existing ENV
  start-all           Start all existing ENVs
  stop                Stop an existing ENV
  stop-all            Stop all existing ENVs
  push-nodes          Push some new nodes to an existing ENV
  migrate-nodes       Migrate some existing nodes to other hosts,
                      NOTE: the 'new' node will be left stopped, a `start` operation may be needed
  kick-nodes          Remove(destroy) some node from an existing ENV
  switch-EL-to-geth   Switch the EL client to `geth`,
                      NOTE: the node will be left stopped, a `start` operation may be needed
  switch-EL-to-reth   Switch the EL client to `reth`,
                      NOTE: the node will be left stopped, a `start` operation may be needed
  push-hosts          Add some new hosts to the cluster
  kick-hosts          Remove some hosts from the cluster
  show                Default operation, show the information of an existing ENV
  show-hosts          Show the remote host configations in JSON or the `nb` native format
  show-all            Show informations of all existing ENVs
  debug-failed-nodes  Show failed nodes in a list
  list-web3-rpcs, -w  List all web3 RPC endpoints of the entire ENV
  list                Show names of all existing ENVs
  host-put-file       Put a local file to all remote hosts
  host-get-file       Get a remote file from all remote hosts
  host-exec           Execute commands on all remote hosts
  get-logs            Get the remote logs from all nodes of the ENV
  dump-vc-data        Dump the validator client data from all nodes of the ENV
  help                Print this message or the help of the given subcommand(s)
```

Set the ssh public key(eg `~/.ssh/id_rsa.pub`) of your localhost to the correct path(eg `~/.ssh/authorized_keys`) on every remote host,
and then transfer all necessary binaries(lighthouse/geth/reth/nb) to remote hosts.

Assume your have 3 remote hosts,
and you have set the ssh public key of your local machine on each of them:
- `10.0.0.2#alice`
- `10.0.0.3#bob`
- `10.0.0.4#jack`

Let's check the help information of `nb dev create`:
```
Create a new ENV

Usage: nb ddev create [OPTIONS]

Options:
  -e, --env-name <ENV_NAME>

      --hosts <HOSTS>
          Define the remote hosts for creating a new ENV,
          or perform some batch operations on these hosts directly,
          there are two alterntive ways to specify the contents.

          # The first(recommand) way is providing a json file,
          EXAMPLE:
          ```json
          {
            "fallback_ssh_local_seckeys": [
              "/home/bob/.ssh/id_ed25519"
            ],
            "fallback_ssh_port": 22,
            "fallback_ssh_user": "bob",
            "fallback_weight": 32,
            "hosts": [
              {
                "ext_ip": "8.8.8.8",
                "local_ip": "10.0.0.2",
                "ssh_local_seckeys": [
                  "/home/fh/alice/.ssh/id_rsa"
                ],
                "ssh_port": 2222,
                "ssh_user": "alice",
                "weight": 8
              },
              {
                "local_ip": "10.0.0.3",
                "weight": 4
              },
              {
                "ext_ip": "8.8.4.4",
                "local_ip": "10.0.0.4",
                "ssh_local_seckeys": [
                  "/home/jack/.ssh/id_rsa",
                  "/home/jack/.ssh/id_ed25519"
                ],
                "ssh_user": "jack"
              }
            ]
          }
          ```
          Only the `local_ip` field is mandatory, the others are optional;
          The `fallback_` prefixed fields can be used to avoid duplicate entries;
          If the `ext_ip` is missing, will fallback to the `local_ip` for external connections;
          The `weight` should be a positive number(1~255),
          if not set, the number of CPU threads on the host will be used,
          the larger the number, the more nodes the host will carry;
          The configuration here will override the `$NB_DDEV_HOSTS_JSON` settings.

          # The second way is to use the custom expressions(contents directly, not a path!),
          FORMAT:
          "
              host_ip,
              host_ip | external_ip,
              host_ip | external_ip # ssh_user,
              host_ip | external_ip # ssh_user # ssh_remote_port,
              host_ip | external_ip # ssh_user # ssh_remote_port # host_weight,
              host_ip | external_ip # ssh_user # ssh_remote_port # host_weight # ssh_seckey_path,
              ... lines one by one ...
          "

          Example A:
          "
              10.0.0.2,
              10.0.0.3 | 8.8.8.1,
              10.0.0.4 | 8.8.8.2 # bob,
              10.0.0.5 | 8.8.8.3 # bob # 22,
              10.0.0.6 | 8.8.8.4 # bob # 22 # 5,
              10.0.0.7 | 8.8.8.5 # bob # 22 # 10 # /usr/bob/.ssh/id_rsa,
          "
          Example B:
          "
              10.0.0.2, 10.0.0.3 | 8.8.8.1, 10.0.0.4 | 8.8.8.2 # bob,
          "
          Only the `host_ip` field is mandatory, the others are optional;
          If the `external_ip` is missing, will fallback to the `host_ip` for external connections;
          All whitespace characters(\n,\t, etc.) will be removed before parsing the value;
          The `host_weight` should be a positive number(1~255),
          if not set, the number of CPU threads on the host will be used,
          the larger the number, the more nodes the host will carry;
          The configuration here will override the `$NB_DDEV_HOSTS` settings.

  -n, --extra-node-num <EXTRA_NODE_NUM>
          How many extra nodes(exclude the fuhrer node) should be created,
          the actual node number will be `1 + this_value` [default: 0]
      --fullnode
          Set extra nodes in FullNode(opposite to ArchiveNode) mode?
  -t, --block-time-secs <BLOCK_TIME_SECS>
          If not set, use the default value in the genesis,
          has no effect when the `--genesis-data-pre-created` option is specified
  -g, --genesis-custom-settings-path <GENESIS_CUSTOM_SETTINGS_PATH>
          The path of a cfg file in the form of
          'https://github.com/rust-util-collections/EGG/blob/master/defaults.env',
          has no effect when the `--genesis-data-pre-created` option is specified
  -G, --genesis-data-pre-created <GENESIS_DATA_PRE_CREATED>
          Concated paths for specifying the pre-created genesis.tar.gz and vcdata.tar.gz,
          they are usally created by the `make build` of 'https://github.com/rust-util-collections/EGG',
          value format: '/PATH/TO/genesis.tar.gz+/PATH/TO/vcdata.tar.gz',
          the `+` is the delimiter between them two
      --el-geth-bin <EL_GETH_BIN>
          The path of your custom geth binary
      --el-reth-bin <EL_RETH_BIN>
          The path of your custom reth binary
      --cl-bin <CL_BIN>
          The path of your custom consensus layer binary
      --force
          Try to destroy the target ENV and then recreate it
```

Create and start a distributed cluster:
```shell
# this distributed cluster has 4 validator nodes and 1 bootstrap node
nb ddev create --hosts '10.0.0.2#alice,10.0.0.3#bob,10.0.0.4#jack'
```

If all the user names are same as the user name of your local machine, the above can be simplified to:
```shell
nb ddev create --hosts '10.0.0.2,10.0.0.3,10.0.0.4'
```

You can use `nb ddev list-web3-rpcs` to get all Web3 endpoints.

If your remote hosts has different OSs with your localhost, your local compiled binaries may not run correctly on the remote hosts. For this situation, there is a `make ddev_docker_runtime` that can solve this problem.

> The premise is that `docker` has already been installed on your remote hosts.    
> You can use the `nb ddev host-exec` to batch install docker or any other apps,    
> for example: `nb ddev host-exec -c "sudo su -c 'apt install docker.io -y'"`.    
> Of course, it is also OK to use podman instead of docker.

After a successful `make ddev_docker_runtime`, you should reset the `$NB_DDEV_HOSTS`(change all the ssh user name to 'nb', change all the ssh port to '2222'). The command itself will output suggested new values, usually without the need for you to edit them manually.

Sample outputs of the `make ddev_docker_runtime`:
```
The new contents of the ${NB_DDEV_HOSTS_JSON} file should be:
{
  "10.0.0.35|3.15.10.61": {
    "local_ip": "10.0.0.35",
    "local_network_id": "",
    "ext_ip": "3.15.10.61",
    "ssh_user": "nb",
    "ssh_port": 2222,
    "ssh_sk_path": "/home/bob/.ssh/id_ed25519",
    "weight": 16,
    "node_cnt": 0
  },
  "10.0.0.56|3.19.7.209": {
    "local_ip": "10.0.0.56",
    "local_network_id": "",
    "ext_ip": "3.19.7.209",
    "ssh_user": "nb",
    "ssh_port": 2222,
    "ssh_sk_path": "/home/bob/.ssh/id_ed25519",
    "weight": 16,
    "node_cnt": 0
  },
  "10.0.0.58|3.17.11.90": {
    "local_ip": "10.0.0.58",
    "local_network_id": "",
    "ext_ip": "3.17.11.90",
    "ssh_user": "nb",
    "ssh_port": 2222,
    "ssh_sk_path": "/home/bob/.ssh/id_ed25519",
    "weight": 16,
    "node_cnt": 0
  }
}

The new value of the ${NB_DDEV_HOSTS} should be:
"
  10.0.0.35|3.15.10.61#nb#22#16#/home/bob/.ssh/id_ed25519,
  10.0.0.56|3.19.7.209#nb#22#16#/home/bob/.ssh/id_ed25519,
  10.0.0.58|3.17.11.90#nb#22#16#/home/bob/.ssh/id_ed25519
"
```

#### Management of 'a single cluster/multiple clusters'

The usage of this section is almost the same as `nb dev`, except that you must specify an additional `--hosts` option to define the necessary information for all remote hosts.

There is also an environment variable named `$NB_DDEV_HOSTS` that has the same function as this option, but the command line option has a higher priority.

The format of acceptable values for this option is as follows:
- `host_ip#remote_user#ssh_port#host_weight#ssh_local_private_key,...`
    - example: `10.0.0.2#bob#22#1#/home/bob/.ssh/id_rsa`
- `host_ip#remote_user#ssh_port#host_weight,...`
    - example: `10.0.0.2#bob#22#9,10.0.0.3#bob#22#5`
    - this style omitted the `ssh_local_private_key` field, its value will be `$HOME/.ssh/id_rsa`
- `host_ip#remote_user#ssh_port,...`
    - example: `10.0.0.2#bob#22,10.0.0.3#bob#22`
    - this style further omitted the `host_weight` field, its value will be automatically calculated according to the number of CPUs and single computing power of each host
- `host_ip#remote_user,...`
    - example: `10.0.0.2#bob,10.0.0.3#bob`
    - this style further omitted the `ssh_port` field, its value will be '22'
- `host_ip,...`
    - example: `10.0.0.2,10.0.0.3`
    - this style further omitted the `remote_user` field, its value will be `$USER` of your local machine

NOTE:
- the delimiter between hosts is ','
- the delimiter between fields is '#'
- if there are some whitespace characters in the content, they will be trimed automatically

Also, there are additional 4 options:
- `--host-put-file`, put a local file to all remote hosts
- `--host-get-file`, get a remote file from all remote hosts
- `--host-exec`, execute commands on all remote hosts
- `--get-logs`, collect all node logs from remote hosts to local host

A json file path can also be passed to the `--hosts` option or as the value of `$NB_DDEV_HOSTS_JSON`, the json contents should be like:
```json
{
  "fallback_ssh_port": 22,
  "fallback_ssh_sk_path": "/home/bob/.ssh/id_ed25519",
  "fallback_ssh_user": "bob",
  "fallback_weight": 32,
  "hosts": [
    {
      "ext_ip": "8.8.8.8",
      "local_ip": "10.0.0.2",
      "ssh_port": 2222,
      "ssh_private_key_path": "/home/fh/alice/.ssh/id_rsa",
      "ssh_user": "alice",
      "weight": 8
    },
    {
      "local_ip": "10.0.0.3",
      "weight": 4
    },
    {
      "ext_ip": "8.8.4.4",
      "local_ip": "10.0.0.4",
      "ssh_private_key_path": "/home/jack/.ssh/id_ed25519",
      "ssh_user": "jack"
    }
  ]
}
```

#### Internal organization of data and logs

The layout is almost the same as `nb dev`, the only difference is that the node data is distributed on the remote hosts instead of your localhost, but, of course, the metadata is still stored on your localhost.

#### OS compatibility

In theory, it can run well on most linux distributions and macOS.
