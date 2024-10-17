all: fmt lint

export STATIC_CHAIN_DEV_BASE_DIR_SUFFIX = NBNET

build:
	cargo build --bins

release:
	cargo build --release --bins

install:
	cargo install --force --path .
	-@ nb -z > ~/.cargo/bin/zsh_nb.completion
	-@ sed -i '/zsh_nb.completion/d' ~/.zshrc
	-@ echo '. ~/.cargo/bin/zsh_nb.completion' >> ~/.zshrc
	-@ zsh ~/.zshrc

lint:
	cargo clippy

update:
	git pull
	git submodule update --init
	cargo update

fmt:
	cargo fmt

fmtall:
	bash tools/fmt.sh

deploy_bin_all: deploy_bin_geth deploy_bin_reth deploy_bin_lighthouse

deploy_bin_geth: bin_geth
	cd submodules/go-ethereum && make geth
	- nb ddev stop --geth
	nb ddev host-exec -c 'sudo su -c "rm -f /tmp/geth /usr/local/bin/geth"'
	nb ddev host-put-file --local-path=submodules/go-ethereum/build/bin/geth --remote-path=/tmp/geth
	nb ddev host-exec -c 'sudo su -c "mv /tmp/geth /usr/local/bin/geth && chmod +x /usr/local/bin/geth"'

deploy_bin_reth: bin_reth
	cd submodules/reth && make build
	- nb ddev stop --reth
	nb ddev host-exec -c 'sudo su -c "rm -f /tmp/reth /usr/local/bin/reth"'
	nb ddev host-put-file --local-path=submodules/reth/target/release/reth --remote-path=/tmp/reth
	nb ddev host-exec -c 'sudo su -c "mv /tmp/reth /usr/local/bin/reth && chmod +x /usr/local/bin/reth"'

deploy_bin_lighthouse: bin_lighthouse
	cd submodules/lighthouse && make
	- nb ddev stop
	nb ddev host-exec -c 'sudo su -c "rm -f /tmp/lighthouse /usr/local/bin/lighthouse"'
	nb ddev host-put-file --local-path=submodules/lighthouse/target/release/lighthouse --remote-path=/tmp/lighthouse
	nb ddev host-exec -c 'sudo su -c "mv /tmp/lighthouse /usr/local/bin/lighthouse && chmod +x /usr/local/bin/lighthouse"'

start_filter_reth:
	nb ddev start --reth

start_filter_geth:
	nb ddev start --geth

start_all:
	nb ddev start

bin_all: install bin_geth bin_reth bin_lighthouse

bin_geth: update_submods
	mkdir -p ~/.cargo/bin
	cd submodules/go-ethereum && make geth
	cp -f submodules/go-ethereum/build/bin/geth ~/.cargo/bin/

bin_reth: update_submods
	mkdir -p ~/.cargo/bin
	cd submodules/reth && make build
	cp -f submodules/reth/target/release/reth ~/.cargo/bin/

bin_lighthouse: update_submods
	mkdir -p ~/.cargo/bin
	cd submodules/lighthouse && make
	cp -f submodules/lighthouse/target/release/lighthouse ~/.cargo/bin/

docker_runtime:
	bash -x tools/ddev_docker_runtime.sh

ddev_docker_runtime: install
	nb ddev host-put-file -l Dockerfile -r /tmp/Dockerfile
	nb ddev host-put-file -l tools/ddev_docker_runtime.sh -r /tmp/ddr.sh
	nb ddev host-exec -c 'cd /tmp && bash -x /tmp/ddr.sh'

git_pull_force:
	git fetch
	git reset --hard origin/master

update_submods:
	git submodule update --init
	# git submodule update --init --recursive
