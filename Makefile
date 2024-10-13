all: fmt lint

export STATIC_CHAIN_DEV_BASE_DIR_SUFFIX = NBNET

build:
	cargo build --bins

release:
	cargo build --release --bins

install: update
	cargo install --force --path .

lint:
	cargo clippy

update:
	cargo update

fmt:
	cargo fmt

fmtall:
	bash tools/fmt.sh

deploy_bin_all: deploy_bin_geth deploy_bin_reth deploy_bin_lighthouse

deploy_bin_reth:
	cd submodules/reth && make build
	nbnet ddev stop --reth
	nbnet ddev host-exec -c 'sudo su -c "rm -f /tmp/reth /usr/local/bin/reth"'
	nbnet ddev host-put-file --local-path=submodules/reth/target/release/reth --remote-path=/tmp/reth
	nbnet ddev host-exec -c 'sudo su -c "mv /tmp/reth /usr/local/bin/reth && chmod +x /usr/local/bin/reth"'

deploy_bin_geth:
	cd submodules/go-ethereum && make geth
	nbnet ddev stop --geth
	nbnet ddev host-exec -c 'sudo su -c "rm -f /tmp/geth /usr/local/bin/geth"'
	nbnet ddev host-put-file --local-path=submodules/geth/build/bin/geth --remote-path=/tmp/geth
	nbnet ddev host-exec -c 'sudo su -c "mv /tmp/geth /usr/local/bin/geth && chmod +x /usr/local/bin/geth"'

deploy_bin_lighthouse:
	cd submodules/lighthouse && make
	nbnet ddev stop
	nbnet ddev host-exec -c 'sudo su -c "rm -f /tmp/lighthouse /usr/local/bin/lighthouse"'
	nbnet ddev host-put-file --local-path=submodules/reth/target/release/lighthouse --remote-path=/tmp/lighthouse
	nbnet ddev host-exec -c 'sudo su -c "mv /tmp/lighthouse /usr/local/bin/lighthouse && chmod +x /usr/local/bin/lighthouse"'

start_filter_reth:
	nbnet ddev start --reth

start_filter_geth:
	nbnet ddev start --geth

start_all:
	nbnet ddev start

docker_runtime:
	- docker images --format=json | grep '"Tag":"\\u003cnone\\u003e"' | jq '.ID' | xargs docker image rm
	if [ 0 -eq $(shell docker images --format json | jq '.Tag' | grep -c 'nbnet_24.04') ]; then \
		docker pull ubuntu:24.04 || exit 1 ; \
		docker tag ubuntu:24.04 ubuntu:nbnet_24.04 || exit 1 ; \
	fi
	cp ~/.ssh/authorized_keys ./
	docker build -t ubuntu:nbnet_runtime_v0 .
	- docker rm -f nbnet_runtime
	mkdir -p ${HOME}/__NB_DATA__/usr_local_bin
	chmod -R 1777 ${HOME}/__NB_DATA__
	docker run --rm -d --network=host \
		-v ${HOME}/__NB_DATA__:/tmp \
		--name nbnet_runtime \
		ubuntu:nbnet_runtime_v0
	docker ps

git_pull_force:
	git fetch
	git reset --hard origin/master
