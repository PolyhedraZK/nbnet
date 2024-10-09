all: fmt lint

build:
	cargo build --bins

release:
	cargo build --release --bins

install:
	cargo install --force --path .

lint:
	cargo clippy

update:
	cargo update

fmt:
	cargo fmt

fmtall:
	bash tools/fmt.sh
