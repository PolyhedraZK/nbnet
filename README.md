![GitHub top language](https://img.shields.io/github/languages/top/nbnet/nbnet)
[![Rust](https://github.com/nbnet/nbnet/actions/workflows/rust.yml/badge.svg)](https://github.com/nbnet/nbnet/actions/workflows/rust.yml)
![Minimum rustc version](https://img.shields.io/badge/rustc-1.81+-lightgray.svg)

> **Yes! This is the right weapon for you 'ETH Grandmaster' !!**

# NBnet

**The `nb` kit is a powerful presence for creating and managing custom ETH2 networks.**

It has two modes.

Mode `dev` is designed to create and manage local clusters running on a single host. It does not require complex configuration, and is an excellent tool for deploying and managing devnets, very suitable for rapid development.

Mode `ddev` is designed to manage distributed multi-machine clusters, and can be used to deploy formal testnet or even mainnet. Its convenient and powerful functions do not even require DevOps to participate in, the develop team can manage the network well by themselves relying on the `nb` kit.

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
- reth
- lighthouse
- nb

If you do not want to spend the compiling time, feel free to use your own binaries:
- Downloading them from the offical sites of these projects
- Or use your own pre-compiled binaries

For the `nb` binary, download the statically compiled(linked) package from [**this link**](https://github.com/NBnet/nbnet/releases/download/v0.2.1/nb.linux.amd64.tar.gz), and put it in your `$PATH`.

If you want to compile a `nb` binary from source, use `make install`. It will be located at `~/.cargo/bin/`, so you should make sure that this directory is under your `$PATH`. We assume you have already been familiar with the configuration of the rust development environment, so we won't introduce this aspect.

#### Command Line Usage

```shell
# nb -h
Usage: nb <COMMAND>

Commands:
  dev                       Manage development clusters on a local host
  ddev                      Manage development clusters on various distributed hosts
  gen-zsh-completions, -z   Generate the cmdline completion script for zsh
  gen-bash-completions, -b  Generate the cmdline completion script for bash
```

For more detailed information, you can get it through the `nb <SUBCOMMAND> -h`.

For example:
```shell
# nb dev destroy -h
Destroy an existing ENV

Usage: nb dev destroy [OPTIONS]

Options:
  -e, --env-name <ENV_NAME>
      --force                Destroy the target ENV even if it is protected
```

#### Shell Completion

Before any real `nb` operation, let's config the shell completion to improve command line efficiency.

For zsh:
```shell
mkdir -p ~/.cargo/bin
nb -z > ~/.cargo/bin/zsh_nb.completion
echo -e "\n source ~/.cargo/bin/zsh_nb.completion" >> ~/.zshrc
source ~/.zshrc
```

For bash:
```shell
mkdir -p ~/.cargo/bin
nb -b > ~/.cargo/bin/bash_nb.completion
echo -e "\n source ~/.cargo/bin/bash_nb.completion" >> ~/.bashrc
source ~/.bashrc
```

#### A simple workflow

For `nb dev`:

1. `nb dev create`: create a new ENV
    - 4 nodes, the first one own all the initial validators
    - the el client is geth
    - the cl client is lighthouse
2. `nb dev show`: show the information of the ENV
3. `nb dev stop`: stop all nodes of the ENV
4. `nb dev start`: restart all nodes of the ENV
5. `nb dev push-node`: add a new node to the ENV
6. `nb dev kick-node`: remove a node from the ENV
    - the first node can never be removed
7. `nb dev destroy`: destroy the entire ENV

For `nb ddev`:

1. Declare the remote hosts
    - `export NB_DDEV_HOSTS="10.0.0.2#bob,10.0.0.3#bob"`
    - This means: you can log in to these two hosts through ssh protocol with username bob without password
2. `nb ddev create`: create a new ENV
    - 4 nodes, the first one own all the initial validators
    - the el client is geth
    - the cl client is lighthouse
3. `nb ddev show`: show the information of the ENV
4. `nb ddev stop`: stop all nodes of the ENV
5. `nb ddev start`: restart all nodes of the ENV
6. `nb ddev push-node`: add a new node to the ENV
7. `nb ddev kick-node`: remove a node from the ENV
    - the first node can never be removed
8. `nb ddev destroy`: destroy the entire ENV

## == More Detailed Tutorials ==

- Check [**this page**](src/dev/README.md) for `nb dev`
- Check [**this page**](src/ddev/README.md) for `nb ddev`

## == ENV VARs ==
- `$NB_DDEV_HOSTS_JSON`
    - Specify hosts infomations in the json file path style
    - Check [**the help info**](src/cfg/hosts.format) for details
- `$NB_DDEV_HOSTS`
    - Specify hosts infomations in the custom expressions style
    - Check [**the help info**](src/cfg/hosts.format) for details
    - The priority is lower than `$NB_DDEV_HOSTS_JSON`
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
    - This mode is currently banned in `nb`
- `reth` will fail to restart without a finalized block
    - That is, reth nodes should be added after the first finalized block

## Q&A

##### 0. What systems can it run on?

Linux and macOS, Linux is more recommended.

##### 1. How to set a custom chain id?

```shell
echo 'export CHAIN_ID="1234"' > custom.env
nb dev create -g custom.env
# nb ddev create -g custom.env
```

##### 2. How to set a custom block time?

Method 1:
```shell
BLOCK_TIME=2 # 2 seconds
nb dev create -t $BLOCK_TIME
# nb ddev create -t $BLOCK_TIME
```

Method 2:
```shell
echo 'export SLOT_DURATION_IN_SECONDS="2"' > custom.env
nb dev create -g custom.env
# nb ddev create -g custom.env
```

'Method 1' has higher priority.

##### 3. Too slow when `nb dev/ddev create`

This is most likely a network problem.

During the `nb dev/ddev create` process, we need to clone the
[**EGG**](https://github.com/rust-util-collections/EGG)
repository from GitHub. If you live in a restricted country, such as North Korea, you can use a mirror source from your own country or a friendly country.

For example:
```shell
export CHAIN_DEV_EGG_REPO="https://gitee.com/kt10/EGG"
```

If you want to further shorten the startup time, you can build the
[**EGG**](https://github.com/rust-util-collections/EGG) manually and assign it to `nb`.
It should be noted that this method will cause all the custom settings of genesis from the command line to become invalid. All your customizations need to be configured directly in the
[**EGG**](https://github.com/rust-util-collections/EGG).

For example:
```shell
cd EGG
make build
nb dev create -G "data/genesis.tar.gz+data/vcdata.tar.gz"
# nb ddev create -G "data/genesis.tar.gz+data/vcdata.tar.gz"
```

##### 4. How to set multiple genesis parameters at the same time

```shell
echo 'export SLOT_DURATION_IN_SECONDS="2"' > custom.env
echo 'export CHAIN_ID="1234"' >> custom.env

# Many other VAR declarations ...

nb dev create -g custom.env
# nb ddev create -g custom.env
```

For all VARs that can be declared, please check the [**defaults.env**](config/defaults.env) file.

##### 5. I don't want to store `nb` data under `/tmp`, how should I do?

There are two recommended methods.

Assume your target path is declared by `$P`.

Method 1:
```shell
mkdir -p $P

# `ddev` is also mgmt under this path
export RUNTIME_CHAIN_DEV_BASE_DIR="$P"
```

Method 2:
> The `make ddev_docker_tuntime` may be a better choice for `ddev`,
> check [**here**](src/ddev/README.md) for more infomation.
```shell
mkdir -p $P
ln -svf $P /tmp/__CHAIN_DEV__

# Needed by `ddev` only!
nb ddev host-exec -c "mkdir -p $P && ln -svf $P /tmp/__CHAIN_DEV__"
```

##### 6. How to check the host information separately

Static configrations:
```shell
nb ddev show-hosts --json
```

Runtime load:
```shell
nb ddev | jq '.meta.remote_hosts'
```

##### 7. Issues like "Address/Port already in use...", etc.

When a large number of nodes are deployed on one or a small number of physical machines, there may be conflicts between `nb` allocated ports and ports dynamically binded by other processes.

The following commands can migigate this problem. If there are still nodes with this problem, run it a few more times and it will finally work.

```shell
nb dev start --ignore-failed
# nb ddev start --ignore-failed
```

![](https://avatars.githubusercontent.com/u/181968946?s=400&u=e6cd742236bfe7c80a2bcced70d05fe9f05ae260&v=4)
![](https://avatars.githubusercontent.com/u/181968946?s=400&u=e6cd742236bfe7c80a2bcced70d05fe9f05ae260&v=4)
![](https://avatars.githubusercontent.com/u/181968946?s=400&u=e6cd742236bfe7c80a2bcced70d05fe9f05ae260&v=4)
