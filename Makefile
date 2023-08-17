.PHONY: build-release 

cargo_check:
	cargo check
	cargo check --no-default-features -F json
	cargo check --no-default-features -F sqlite

run_test:
	cargo test

clippy:
	cargo clippy

build-release:
	cargo build --release

release-mac: build-release
	strip target/release/tjournal
	otool -L target/release/tjournal
	mkdir -p release
	tar -C ./target/release/ -czvf ./release/tjournal-mac.tar.gz ./tjournal
	ls -lisah ./release/tjournal-mac.tar.gz

release-win: build-release
	mkdir -p release
	7z -y a ./release/tjournal-win.zip ./target/release/tjournal.exe

release-linux: 
	cargo build --release --target=x86_64-unknown-linux-gnu
	strip target/x86_64-unknown-linux-gnu/release/tjournal
	mkdir -p release
	tar -C ./target/x86_64-unknown-linux-gnu/release/ -czvf ./release/tjournal-linux-gnu.tar.gz ./tjournal

install:
	cargo install --path "." 

install_sqlite:
	cargo install --path "." --no-default-features -F sqlite 

install_json:
	cargo install --path "." --no-default-features -F json 

