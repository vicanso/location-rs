lint:
	cargo clippy

fmt:
	cargo fmt

build-web:
	cd web \
	&& yarn install && yarn build \
	&& cp -rf dist ../

template:
	rm src/ip_data.rs \
	&& cp src/ip_data.tpl src/ip_data.rs 

dev:
	cargo watch -w src -x 'run'
dev-build:
	cargo run -- build 100

release:
	cargo run -- build
	cargo build --release
	ls -lh target/release

hooks:
	cp hooks/* .git/hooks/