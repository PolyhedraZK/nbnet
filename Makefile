all: fmt lint build

build:
	cargo build --bins

release:
	cargo build --release --bins

fmt:
	cargo fmt

fmtall:
	bash -x tools/fmt.sh

lint:
	cargo clippy
