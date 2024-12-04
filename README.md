![GitHub top language](https://img.shields.io/github/languages/top/PolyhedraZK/EXPchain)
[![Rust](https://github.com/PolyhedraZK/EXPchain/actions/workflows/rust.yml/badge.svg)](https://github.com/PolyhedraZK/EXPchain/actions/workflows/rust.yml)
![Minimum rustc version](https://img.shields.io/badge/rustc-1.81+-lightgray.svg)

> **Yes! This is the right weapon for you 'ETH Grandmaster' !!**

# EXPchain

**EXPchain is a powerful presence for creating and managing custom ETH2 networks.**

It has two modes.

Mode `dev` is designed to create and manage local clusters running on a single host. It does not require complex configuration, and is an excellent tool for deploying and managing devnets, very suitable for rapid development.

Mode `ddev` is designed to manage distributed multi-machine clusters, and can be used to deploy formal testnet or even mainnet. Its convenient and powerful functions do not even require DevOps to participate in, the develop team can manage the network well by themselves relying on the `exp` kit.

General intro:
- You can deploy as many networks as needed
    - Each network can easily extend to hundreds of nodes
- You can flexibly add or kick out nodes
- You can flexibly start and stop nodes
    - Specify explicit IDs or filter by client type
- Many other practical functions...

## == Quick Start ==

**NOTE: all demos below are based on the Linux OS.**

#### Prepare Binaries

Use `make bin_all` to build the necessary binaries:
- geth
- reth(unstable, test only!)
- lighthouse
- exp

If you do not want to spend the compiling time, feel free to use your own binaries:
- Downloading them from the offical sites of these projects
- Or use your own pre-compiled binaries

For the `exp` binary, download the statically compiled(linked) package from [**this link**](https://github.com/PolyhedraZK/EXPchain/releases/download/v0.6.0/exp.linux.amd64.tar.gz), and put it in your `$PATH`.

If you want to compile a `exp` binary from source, use `make install`. It will be located at `~/.cargo/bin/`, so you should make sure that this directory is under your `$PATH`. We assume you have already been familiar with the configuration of the rust development environment, so we won't introduce this aspect.

#### Command Line Usage

```
# exp -h
Usage: exp <COMMAND>

Commands:
  dev                       Manage development clusters on a local host
  ddev                      Manage development clusters on various distributed hosts
  deposit                   Manage deposit operations
  validator-exit            Exit an existing validator from the beacon chain
  new-mnemonic              Create a 24-words bip39 mnemonic
  gen-zsh-completions, -z   Generate the cmdline completion script for zsh
  gen-bash-completions, -b  Generate the cmdline completion script for bash
  help                      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

```

For more detailed information, you can get it through the `exp <SUBCOMMAND> -h`.

For example:
```
# exp ddev show -h
Default operation, show the information of an existing ENV

Usage: exp ddev show [OPTIONS]

Options:
  -e, --env-name <ENV_NAME>
  -c, --clean-up             Clean up expired data before showing
  -w, --write-back           Whether to write back after a `clean up`
```

#### Shell Completion

Before any real `exp` operation, let's config the shell completion to improve command line efficiency.

For zsh:
```shell
mkdir -p ~/.cargo/bin
exp -z > ~/.cargo/bin/zsh_exp.completion
echo -e "\n source ~/.cargo/bin/zsh_exp.completion" >> ~/.zshrc
source ~/.zshrc
```

For bash:
```shell
mkdir -p ~/.cargo/bin
exp -b > ~/.cargo/bin/bash_exp.completion
echo -e "\n source ~/.cargo/bin/bash_exp.completion" >> ~/.bashrc
source ~/.bashrc
```

#### A simple workflow

For `exp dev`:

1. `exp dev create`: create a new ENV
    - 4 nodes, the first one own all the initial validators
    - the el client is geth
    - the cl client is lighthouse
2. `exp dev show`: show the information of the ENV
3. `exp dev stop -N all`: stop all nodes of the ENV
4. `exp dev start`: restart all nodes of the ENV
5. `exp dev push-node`: add a new node to the ENV
6. `exp dev kick-node`: remove a node from the ENV
    - the first node can never be removed
7. `exp dev destroy`: destroy the entire ENV

For `exp ddev`:

1. Declare the remote hosts
    - `export EXP_DDEV_HOSTS="10.0.0.2#bob,10.0.0.3#bob"`
    - This means: you can log in to these two hosts through ssh protocol with username bob without password
2. `exp ddev create`: create a new ENV
    - 4 nodes, the first one own all the initial validators
    - the el client is geth
    - the cl client is lighthouse
3. `exp ddev show`: show the information of the ENV
4. `exp ddev stop -N all`: stop all nodes of the ENV
5. `exp ddev start`: restart all nodes of the ENV
6. `exp ddev push-node`: add a new node to the ENV
7. `exp ddev kick-node`: remove a node from the ENV
    - the first node can never be removed
8. `exp ddev destroy`: destroy the entire ENV

## == More Detailed Tutorials ==

- Check [**this page**](src/dev/README.md) for `exp dev`
- Check [**this page**](src/ddev/README.md) for `exp ddev`
- Check [**this page**](src/pos/README.md) for `exp deposit`

## == ENV VARs ==
- `$EXP_DDEV_HOSTS_JSON`
    - Specify hosts infomations in the json file path style
    - Check [**the help info**](src/cfg/hosts.format) for details
- `$EXP_DDEV_HOSTS`
    - Specify hosts infomations in the custom expressions style
    - Check [**the help info**](src/cfg/hosts.format) for details
    - The priority is lower than `$EXP_DDEV_HOSTS_JSON`
- `$RUNTIME_CHAIN_DEV_BASE_DIR`
    - All runtime data will be mgmt under the path declared by this VAR
    - Default: `/tmp/__CHAIN_DEV__`
- `$CHAIN_DEV_EGG_REPO`
    - Where to clone the EGG package for generating the genesis data
    - Default: [**rust-util-collections/EGG**](https://github.com/rust-util-collections/EGG)
- `$RUC_SSH_TIMEOUT`
    - `ssh` connection timeout, default to 20s
    - 300s at most, any value larger than this will be truncated
    - Probably only useful if you want to transfer large files to or from hosts

## == Clients Support ==

- CL clients:
  - `lighthouse`
  - `prysm`, not implemented yet, but in the plan
- EL clients:
  - `geth`
  - `reth`, limited support

## == Known Issues ==

Reth related:
- `reth` can not be used as the genesis node
    - It will hang at the next restarting
- `reth`'s `fullnode` mode is unstable in practice
    - This mode is currently banned in `exp`
- `reth` will fail to restart without a finalized block
    - That is, reth nodes should be added after the first finalized block

## Q&A

##### 0. What systems can it run on?

Linux and macOS, Linux is more recommended.

##### 1. How to set a custom chain id?

```shell
export CHAIN_ID="1234"
exp dev create
# exp ddev create
```

##### 2. How to set a custom block time?

Method 1:
```shell
BLOCK_TIME=2 # 2 seconds
exp dev create -t $BLOCK_TIME
# exp ddev create -t $BLOCK_TIME
```

Method 2:
```shell
export SLOT_DURATION_IN_SECONDS="2"
exp dev create
# exp ddev create
```

'Method 1' has higher priority.

##### 3. How to set multiple genesis parameters at the same time?

```shell
echo 'export SLOT_DURATION_IN_SECONDS="2"' > custom.env
echo 'export CHAIN_ID="1234"' >> custom.env

# Many other VAR declarations ...

exp dev create -g custom.env
# exp ddev create -g custom.env
```

For all VARs that can be declared, please check the [**defaults.env**](config/genesis/defaults.env) file.

The are two ready-made examples:
- [**mainnet.env**](config/genesis/mainnet.env)
    - Similar to the ETH mainnet configuration
- [**minimal.env**](config/genesis/minimal.env)
    - A minimal configuration, for quick testing

##### 4. Too slow when `exp dev/ddev create`

This is most likely a network problem.

During the `exp dev/ddev create` process, we need to clone the
[**EGG**](https://github.com/rust-util-collections/EGG)
repository from GitHub. If you live in a restricted country, such as North Korea, you can use a mirror source from your own country or a friendly country.

For example:
```shell
# Use mirror repo
export CHAIN_DEV_EGG_REPO="https://gitlab.com/YOUR_NAME/EGG"

# Or a more efficient approach
git clone https://gitlab.com/YOUR_NAME/EGG
export CHAIN_DEV_EGG_REPO="/PATH/TO/THE/LOCAL/EGG"
```

If you want to further shorten the startup time, you can build the
[**EGG**](https://github.com/rust-util-collections/EGG) manually and assign it to `exp`.
It should be noted that this method will cause all the custom settings of genesis from the command line to become invalid. All your customizations need to be configured directly in the
[**EGG**](https://github.com/rust-util-collections/EGG).

For example:
```shell
cd EGG
make build
exp dev create -G "data/genesis.tar.gz+data/vcdata.tar.gz"
# exp ddev create -G "data/genesis.tar.gz+data/vcdata.tar.gz"
```

##### 5. I don't want to store `exp` data under `/tmp`, what should I do?

There are two recommended methods.

Assume your target path is declared by `$P`.

Method 1:
```shell
mkdir -p $P

# `dev` and `ddev` are both mgmt under this path
export RUNTIME_CHAIN_DEV_BASE_DIR="$P"
```

Method 2:
> The `make ddev_docker_runtime` may be a better choice for `ddev`,
> check [**here**](src/ddev/README.md) for more infomation.
```shell
mkdir -p $P
ln -svf $P /tmp/__CHAIN_DEV__

# Needed by `ddev` only!
exp ddev host-exec -c "mkdir -p $P && ln -svf $P /tmp/__CHAIN_DEV__"
```

##### 6. How to check the host information separately?

```shell
exp ddev show-hosts --json

# OR

exp ddev | jq '.meta.remote_hosts'
```

##### 7. How to troubleshoot failed nodes?

Try the `exp ddev debug-failed-nodes` command, it will show all failed nodes.

Sample outputs in `dev`:
```json
[
  352,
  364,
  392,
  400
]
```

Sample outputs in `ddev`:
```json
{
  "10.0.0.2": [
    383,
    387
  ],
  "10.0.0.3": [
    352,
    364,
    392,
    400
  ]
}
```

##### 8. Issues like "Address/Port already in use...", etc.

When a large number of nodes are deployed on one or a small number of physical machines, there may be conflicts between `exp` allocated ports and ports dynamically binded by other processes.

The following commands can migigate this problem:
```shell
exp dev start -I
# exp ddev start -I
```

If some nodes always fail, try this:
```shell
# run it a few more times and it will finally clean up all failed nodes
# NOTE: the `-R` option will cause the failed node ports to be reallocated!
exp dev start -I -R
# exp ddev start -I -R
```

##### 9. How to sync from the genesis instead of a checkpoint?

```shell
export EXPCHAIN_NODE_SYNC_FROM_GENESIS=1
exp dev push-nodes
# exp ddev push-nodes
```
