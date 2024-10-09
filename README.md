# nbnet

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
  dev   Manage development clusters on local host
  ddev  Manage development clusters on remote hosts
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```shell
# nbnet dev -h

Manage development clusters on a local host

Usage: nbnet dev [OPTIONS] [COMMAND]

Commands:
  create       Create a new ENV
  destroy      Destroy an existing ENV
  destroy-all  Destroy all existing ENVs
  protect      Protect an existing ENV
  unprotect    Unprotect an existing ENV
  start        Start an existing ENV
  start-all    Start all existing ENVs
  stop         Stop an existing ENV
  stop-all     Stop all existing ENVs
  push-node    Push a new node to an existing ENV
  kick-node    Remove an existing node from an existing ENV
  show         Default operation, show the information of an existing ENV
  show-all     Show informations of all existing ENVs
  list         Show names of all existing ENVs
  help         Print this message or the help of the given subcommand(s)

Options:
  -e, --env-name <ENV_NAME>
  -h, --help                 Print help
```

```shell
# nbnet ddev -h

Manage development clusters on various distributed hosts

Usage: nbnet ddev [OPTIONS] [COMMAND]

Commands:
  create         Create a new ENV
  destroy        Destroy an existing ENV
  destroy-all    Destroy all existing ENVs
  protect        Protect an existing ENV
  unprotect      Unprotect an existing ENV
  start          Start an existing ENV
  start-all      Start all existing ENVs
  stop           Stop an existing ENV
  stop-all       Stop all existing ENVs
  push-node      Push a new node to an existing ENV
  migrate-node   Migrate an existing node to another host
  kick-node      Remove an existing node from an existing ENV
  push-host      Add a new host to the cluster
  kick-host      Remove a host from the cluster
  show           Default operation, show the information of an existing ENV
  show-all       Show informations of all existing ENVs
  list           Show names of all existing ENVs
  host-put-file  Put a local file to all remote hosts
  host-get-file  Get a remote file from all remote hosts
  host-exec      Execute commands on all remote hosts
  get-logs       Get the remote logs from all nodes of the ENV
  dump-vc-data   Dump the validator client data from all nodes of the ENV
  help           Print this message or the help of the given subcommand(s)

Options:
  -e, --env-name <ENV_NAME>
  -h, --help                 Print help
```
