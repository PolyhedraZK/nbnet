all: fmt lint

export STATIC_CHAIN_DEV_BASE_DIR_SUFFIX = NBNET

build:
	cargo build --bins

release:
	cargo build --release --bins

musl_release:
	OPENSSL_DIR=/usr/local/musl cargo build --release --target=x86_64-unknown-linux-musl
	cp target/x86_64-unknown-linux-musl/release/nb ~/.cargo/bin/musl_nb
	mkdir -p /tmp/musl_binaries
	cp target/x86_64-unknown-linux-musl/release/nb /tmp/musl_binaries/

musl_build_openssl:
	sudo bash -x tools/build_openssl_musl.sh

install: release
	rm -f ~/.cargo/bin/nb
	cp target/release/nb ~/.cargo/bin/
	- nb -z > ~/.cargo/bin/zsh_nb.completion
	- sed -i '/zsh_nb.completion/d' ~/.zshrc
	- echo '. ~/.cargo/bin/zsh_nb.completion' >> ~/.zshrc

lint:
	cargo clippy

test:
	cargo test

test_ignored:
	cargo test --release -- --ignored --nocapture

test_all: test test_ignored

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
	@- nb ddev stop --geth 2>/dev/null
	nb ddev host-exec -c 'sudo su -c "rm -f /tmp/geth /usr/local/bin/geth"'
	nb ddev host-put-file --local-path=submodules/go-ethereum/build/bin/geth --remote-path=/tmp/geth
	nb ddev host-exec -c 'sudo su -c "mv /tmp/geth /usr/local/bin/geth && chmod +x /usr/local/bin/geth"'

deploy_bin_reth: bin_reth
	@- nb ddev stop --reth 2>/dev/null
	nb ddev host-exec -c 'sudo su -c "rm -f /tmp/reth /usr/local/bin/reth"'
	nb ddev host-put-file --local-path=submodules/reth/target/release/reth --remote-path=/tmp/reth
	nb ddev host-exec -c 'sudo su -c "mv /tmp/reth /usr/local/bin/reth && chmod +x /usr/local/bin/reth"'

deploy_bin_lighthouse: bin_lighthouse
	@- nb ddev stop 2>/dev/null
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

bin_geth:
	mkdir -p ~/.cargo/bin
	cd submodules/go-ethereum && make geth
	cp -f submodules/go-ethereum/build/bin/geth ~/.cargo/bin/

bin_reth:
	mkdir -p ~/.cargo/bin
	cd submodules/reth && make build
	cp -f submodules/reth/target/release/reth ~/.cargo/bin/

bin_lighthouse:
	mkdir -p ~/.cargo/bin
	cd submodules/lighthouse && make
	cp -f submodules/lighthouse/target/release/lighthouse ~/.cargo/bin/

bin_scd:
	mkdir -p ~/.cargo/bin
	cd submodules/side-chain-data-collector && make release
	cp -f submodules/side-chain-data-collector/target/release/scd ~/.cargo/bin/

docker_runtime: bin_scd
	bash -x tools/ddev_docker_runtime.sh \
		$(shell pwd)/tools/Dockerfile \
		$(shell pwd)/submodules/side-chain-data-collector/target/release/scd

ddev_docker_runtime: install bin_scd
	nb ddev host-put-file -l submodules/side-chain-data-collector/target/release/scd -r /tmp/scd
	nb ddev host-put-file -l tools/entrypoint.sh -r /tmp/entrypoint.sh
	nb ddev host-put-file -l tools/Dockerfile -r /tmp/Dockerfile
	nb ddev host-put-file -l tools/ddev_docker_runtime.sh -r /tmp/ddr.sh
	nb ddev host-exec -c 'bash -x /tmp/ddr.sh /tmp/Dockerfile /tmp/scd'
	@ printf '\n\x1b[0;33mThe new contents of the $${NB_DDEV_HOSTS_JSON} file should be:\x1b[0m\n'
	@ nb ddev show-hosts --json \
		| sed -r 's/("ssh_port": )[0-9]+/\12222/g' \
		| sed -r 's/("ssh_user": ")\w*/\1nb/g'
	@ printf '\n\x1b[0;33mThe new value of the $${NB_DDEV_HOSTS} should be:\x1b[0m\n'
	@ nb ddev show-hosts

git_submods:
	git submodule update --init
	# git submodule update --init --recursive

git_fetch_reset:
	git fetch
	git reset --hard origin/master
