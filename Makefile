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
	cd submodules/expander && cargo update
	cd submodules/reth && cargo update
	cd submodules/lighthouse && cargo update
	cd submodules/side-chain-data-collector && cargo update

fmt:
	cargo fmt

fmtall:
	bash tools/fmt.sh

deploy_bin_all: deploy_bin_geth deploy_bin_reth deploy_bin_lighthouse

deploy_bin_geth: bin_geth
	@- nb ddev stop --geth 2>/dev/null
	nb ddev host-exec -c \
		'sudo su -c "rm -f /tmp/geth.txz /tmp/geth /usr/local/bin/geth"'
	nb ddev host-put-file \
		--local-path=submodules/go-ethereum/build/bin/geth.txz \
		--remote-path=/tmp/geth.txz
	nb ddev host-exec -c \
		'sudo su -c "cd /tmp && tar -xf geth.txz && mv geth /usr/local/bin/geth && chmod +x /usr/local/bin/geth"'

deploy_bin_reth: bin_reth
	@- nb ddev stop --reth 2>/dev/null
	nb ddev host-exec -c \
		'sudo su -c "rm -f /tmp/reth.txz /tmp/reth /usr/local/bin/reth"'
	nb ddev host-put-file \
		--local-path=submodules/reth/target/release/reth.txz \
		--remote-path=/tmp/reth.txz
	nb ddev host-exec -c \
		'sudo su -c "cd /tmp && tar -xf reth.txz && mv reth /usr/local/bin/reth && chmod +x /usr/local/bin/reth"'

deploy_bin_lighthouse: bin_lighthouse
	@- nb ddev stop 2>/dev/null
	nb ddev host-exec -c \
		'sudo su -c "rm -f /tmp/lighthouse.txz /tmp/lighthouse /usr/local/bin/lighthouse"'
	nb ddev host-put-file \
		--local-path=submodules/lighthouse/target/release/lighthouse.txz \
		--remote-path=/tmp/lighthouse.txz
	nb ddev host-exec -c \
		'sudo su -c "cd /tmp && tar -xf lighthouse.txz && mv lighthouse /usr/local/bin/lighthouse && chmod +x /usr/local/bin/lighthouse"'

start_filter_reth:
	nb ddev start --reth

start_filter_geth:
	nb ddev start --geth

start_all:
	nb ddev start

bin_all: install bin_geth bin_reth bin_lighthouse

bin_geth: basic_prepare
	cd submodules/go-ethereum && make geth
	cd submodules/go-ethereum/build/bin \
		&& rm -f geth.txz \
		&& tar -Jcf geth.txz geth \
		&& cp -f geth ~/.cargo/bin/

bin_reth: basic_prepare
	cd submodules/reth && make build_with_expander
	cd submodules/reth/target/release \
		&& rm -f reth.txz \
		&& tar -Jcf reth.txz reth \
		&& cp -f reth ~/.cargo/bin/

bin_lighthouse: basic_prepare
	cd submodules/lighthouse && make build_release
	cd submodules/lighthouse/target/release \
		&& rm -f lighthouse.txz \
		&& tar -Jcf lighthouse.txz lighthouse \
		&& cp -f lighthouse ~/.cargo/bin/

bin_scd: basic_prepare
	cd submodules/side-chain-data-collector && make release
	cp -f submodules/side-chain-data-collector/target/release/scd ~/.cargo/bin/

bin_expander: basic_prepare
	cd submodules/expander && \
		cargo build --release --bin expander-exec
	cp -f submodules/expander/target/release/expander-exec ~/.cargo/bin/

docker_runtime: bin_scd bin_expander
	bash -x tools/ddev_docker_runtime.sh \
		$(shell pwd)/tools/Dockerfile \
		$(shell pwd)/submodules/side-chain-data-collector/target/release/scd \
		$(shell pwd)/submodules/expander/target/release/expander-exec

ddev_docker_runtime: install bin_scd bin_expander
	nb ddev host-put-file \
		-l submodules/side-chain-data-collector/target/release/scd \
		-r /tmp/scd
	nb ddev host-put-file \
		-l submodules/expander/target/release/expander-exec \
		-r /tmp/expander-exec
	nb ddev host-put-file -l tools/entrypoint.sh -r /tmp/entrypoint.sh
	nb ddev host-put-file -l tools/Dockerfile -r /tmp/Dockerfile
	nb ddev host-put-file -l tools/ddev_docker_runtime.sh -r /tmp/ddr.sh
	nb ddev host-exec -c \
		'bash -x /tmp/ddr.sh /tmp/Dockerfile /tmp/scd /tmp/expander-exec'
	@ printf '\n\x1b[0;33mThe new contents of the $${NB_DDEV_HOSTS_JSON} file should be:\x1b[0m\n'
	@ nb ddev show-hosts --json \
		| sed -r 's/("ssh_port": )[0-9]+/\12222/g' \
		| sed -r 's/("ssh_user": ")\w*/\1nb/g'
	@ printf '\n\x1b[0;33mThe new value of the $${NB_DDEV_HOSTS} should be:\x1b[0m\n'
	@ nb ddev show-hosts

git_fetch_reset:
	git fetch
	git reset --hard origin/$(shell git branch --show-current)

git_submods:
	git submodule update --init --recursive

basic_prepare:
	mkdir -p ~/.cargo/bin
