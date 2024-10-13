![GitHub top language](https://img.shields.io/github/languages/top/nbnet/nbnet)
[![Rust](https://github.com/nbnet/nbnet/actions/workflows/rust.yml/badge.svg)](https://github.com/nbnet/nbnet/actions/workflows/rust.yml)
![Minimum rustc version](https://img.shields.io/badge/rustc-1.81+-lightgray.svg)

# nbnet

### Known issues

Reth related:
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

### ENV VARs

- `${NBNET_DDEV_HOSTS_JSON}`
    - Specify hosts infomations in the json file path style
    - Check [**the help info**](src/cfg/hosts.format) for details
- `${NBNET_DDEV_HOSTS}`
    - Specify hosts infomations in the custom expressions style
    - Check [**the help info**](src/cfg/hosts.format) for details
