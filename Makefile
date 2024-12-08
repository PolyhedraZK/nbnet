all: fmt lint

export STATIC_CHAIN_DEV_BASE_DIR_SUFFIX = EXPCHAIN

build:
	cargo build --bins

release:
	cargo build --release --bins

musl_release:
	OPENSSL_DIR=/usr/local/musl cargo build --release --target=x86_64-unknown-linux-musl
	cp target/x86_64-unknown-linux-musl/release/exp ~/.cargo/bin/musl_exp
	mkdir -p /tmp/musl_binaries
	cp target/x86_64-unknown-linux-musl/release/exp /tmp/musl_binaries/

musl_build_openssl:
	sudo bash -x tools/build_openssl_musl.sh

install: release
	rm -f ~/.cargo/bin/exp
	cp target/release/exp ~/.cargo/bin/
	- exp -z > ~/.cargo/bin/zsh_exp.completion
	- sed -i '/zsh_exp.completion/d' ~/.zshrc
	- echo '. ~/.cargo/bin/zsh_exp.completion' >> ~/.zshrc

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
	cd submodules/scdc && cargo update

fmt:
	cargo fmt

fmtall:
	bash tools/fmt.sh

deploy_restart_bin_all: deploy_bin_all restart_all

deploy_restart_bin_geth: deploy_bin_geth restart_filter_geth

deploy_restart_bin_reth: deploy_bin_reth restart_filter_reth

deploy_restart_bin_lighthouse: deploy_bin_lighthouse restart_filter_lighthouse

restart_all: restart_filter_lighthouse

restart_filter_geth:
	exp ddev restart --geth -w 6

restart_filter_reth:
	exp ddev restart --reth -w 6

restart_filter_lighthouse:
	exp ddev restart -w 6

deploy_bin_all: deploy_bin_geth deploy_bin_reth deploy_bin_lighthouse

deploy_bin_geth: bin_geth
	exp ddev host-exec -c \
		'sudo su -c "rm -f /tmp/geth.txz /tmp/geth /usr/local/bin/geth"'
	exp ddev host-put-file \
		--local-path=submodules/go-ethereum/build/bin/geth.txz \
		--remote-path=/tmp/geth.txz
	exp ddev host-exec -c \
		'sudo su -c "cd /tmp && tar -xf geth.txz && mv geth /usr/local/bin/geth && chmod +x /usr/local/bin/geth"'

deploy_bin_reth: bin_reth
	exp ddev host-exec -c \
		'sudo su -c "rm -f /tmp/reth.txz /tmp/reth /usr/local/bin/reth"'
	exp ddev host-put-file \
		--local-path=submodules/reth/target/release/reth.txz \
		--remote-path=/tmp/reth.txz
	exp ddev host-exec -c \
		'sudo su -c "cd /tmp && tar -xf reth.txz && mv reth /usr/local/bin/reth && chmod +x /usr/local/bin/reth"'

deploy_bin_lighthouse: bin_lighthouse
	exp ddev host-exec -c \
		'sudo su -c "rm -f /tmp/lighthouse.txz /tmp/lighthouse /usr/local/bin/lighthouse"'
	exp ddev host-put-file \
		--local-path=submodules/lighthouse/target/release/lighthouse.txz \
		--remote-path=/tmp/lighthouse.txz
	exp ddev host-exec -c \
		'sudo su -c "cd /tmp && tar -xf lighthouse.txz && mv lighthouse /usr/local/bin/lighthouse && chmod +x /usr/local/bin/lighthouse"'

deploy_bin_scd: bin_scd
	exp ddev host-exec -c \
		'sudo su -c "rm -f /tmp/scd.txz /tmp/scd /usr/bin/scd; pkill scd"'
	exp ddev host-put-file \
		--local-path=submodules/scdc/target/release/scd.txz \
		--remote-path=/tmp/scd.txz
	exp ddev host-exec -c \
		'sudo su -c "cd /tmp && tar -xf scd.txz && mv scd /usr/bin/scd && chmod +x /usr/bin/scd && scd -d >>/tmp/scd.log 2>&1"'

deploy_bin_expander: bin_expander
	exp ddev host-exec -c \
		'sudo su -c "rm -f /tmp/expander-exec.txz /tmp/expander-exec /usr/bin/expander-exec"'
	exp ddev host-put-file \
		--local-path=submodules/expander/target/release/expander-exec.txz \
		--remote-path=/tmp/expander-exec.txz
	exp ddev host-exec -c \
		'sudo su -c "cd /tmp && tar -xf expander-exec.txz && mv expander-exec /usr/bin/expander-exec && chmod +x /usr/bin/expander-exec"'

bin_all: install bin_lighthouse bin_geth bin_reth

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
	cd submodules/scdc && make release
	cd submodules/scdc/target/release \
		&& rm -f scd.txz \
		&& tar -Jcf scd.txz scd \
		&& cp -f scd ~/.cargo/bin/

bin_expander: basic_prepare
	cd submodules/expander && \
		cargo build --release --bin expander-exec
	cd submodules/expander/target/release \
		&& rm -f expander-exec.txz \
		&& tar -Jcf expander-exec.txz expander-exec \
		&& cp -f expander-exec ~/.cargo/bin/

docker_runtime: bin_scd bin_expander
	bash -x tools/ddev_docker_runtime.sh \
		$(shell pwd)/tools/Dockerfile \
		$(shell pwd)/submodules/scdc/target/release/scd \
		$(shell pwd)/submodules/expander/target/release/expander-exec

ddev_docker_runtime: ddev_docker_runtime_prepare ddev_docker_runtime_alone

ddev_docker_runtime_alone:
	exp ddev host-exec -c \
		'bash -x /tmp/ddr.sh /tmp/Dockerfile /tmp/scd /tmp/expander-exec'
	@ printf '\n\x1b[0;33mThe new contents of the $${EXP_DDEV_HOSTS_JSON} file should be:\x1b[0m\n'
	@ exp ddev show-hosts --json \
		| sed -r 's/("ssh_port": )[0-9]+/\12222/g' \
		| sed -r 's/("ssh_user": ")\w*/\1exp/g'
	@ printf '\n\x1b[0;33mThe new value of the $${EXP_DDEV_HOSTS} should be:\x1b[0m\n'
	@ exp ddev show-hosts

ddev_docker_runtime_prepare: install bin_scd bin_expander
	# exp ddev host-exec -c \
	# 	'sudo su -c "apt update && apt install -y docker.io"'
	exp ddev host-put-file \
		-l submodules/scdc/target/release/scd \
		-r /tmp/scd
	exp ddev host-put-file \
		-l submodules/expander/target/release/expander-exec \
		-r /tmp/expander-exec
	exp ddev host-put-file -l tools/entrypoint.sh -r /tmp/entrypoint.sh
	exp ddev host-put-file -l tools/Dockerfile -r /tmp/Dockerfile
	exp ddev host-put-file -l tools/ddev_docker_runtime.sh -r /tmp/ddr.sh

git_fetch_reset:
	git fetch
	git reset --hard origin/$(shell git branch --show-current)

git_submods:
	git submodule update --init --recursive

basic_prepare:
	mkdir -p ~/.cargo/bin
