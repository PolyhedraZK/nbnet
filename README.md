![GitHub top language](https://img.shields.io/github/languages/top/nbnet/nbnet)
[![Rust](https://github.com/nbnet/netnet/actions/workflows/rust.yml/badge.svg)](https://github.com/nbnet/netnet/actions/workflows/rust.yml)
![Minimum rustc version](https://img.shields.io/badge/rustc-1.81+-lightgray.svg)

# nbnet

### Known issues

- The `fullnode` mode of `reth` can not be used, it is unstable in practice
- `reth` can not be used as the genesis node, it will hang at the next restarting
- `reth` will fail to restart without a final block(before the first final block)

### Cmdline completions

For zsh:
```shell
nbnet -z > ~/.cargo/bin/zsh.nbnet
echo -e "\n source ~/.cargo/bin/zsh.nbnet" >> ~/.zshrc
source ~/.zshrc
```

For bash:
```shell
nbnet -b > ~/.cargo/bin/bash.nbnet
echo -e "\n source ~/.cargo/bin/bash.nbnet" >> ~/.bashrc
source ~/.bashrc
```

### Cmdline usage

```shell
# nbnet -h
Usage: nbnet <COMMAND>

Commands:
  dev                       Manage development clusters on a local host
  ddev                      Manage development clusters on various distributed hosts
  gen-zsh-completions, -z   Generate the cmdline completion script for zsh
  gen-bash-completions, -b  Generate the cmdline completion script for bash
  help                      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```shell
# nbnet dev -h
Manage development clusters on a local host

Usage: nbnet dev [OPTIONS] [COMMAND]

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
  kick-nodes          Remove(destroy) some nodes from an existing ENV
  switch-EL-to-geth   Switch the EL client to `geth`,
                      NOTE: the node will be left stopped, a `start` operation may be needed
  switch-EL-to-reth   Switch the EL client to `reth`,
                      NOTE: the node will be left stopped, a `start` operation may be needed
  show                Default operation, show the information of an existing ENV
  show-all            Show informations of all existing ENVs
  list-web3-rpcs, -w  List all web3 RPC endpoints of the entire ENV
  list                Show names of all existing ENVs
  help                Print this message or the help of the given subcommand(s)

Options:
  -e, --env-name <ENV_NAME>
  -h, --help                 Print help
```

```shell
# nbnet ddev -h
Manage development clusters on various distributed hosts

Usage: nbnet ddev [OPTIONS] [COMMAND]

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
  show-all            Show informations of all existing ENVs
  list-web3-rpcs, -w  List all web3 RPC endpoints of the entire ENV
  list                Show names of all existing ENVs
  host-put-file       Put a local file to all remote hosts
  host-get-file       Get a remote file from all remote hosts
  host-exec           Execute commands on all remote hosts
  get-logs            Get the remote logs from all nodes of the ENV
  dump-vc-data        Dump the validator client data from all nodes of the ENV
  help                Print this message or the help of the given subcommand(s)

Options:
  -e, --env-name <ENV_NAME>
  -h, --help                 Print help
```
