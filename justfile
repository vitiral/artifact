# this is a comment

test: build
	cargo test --lib

filter PATTERN:
	cargo test --lib {{PATTERN}}

backtrace:
	RUST_BACKTRACE=1 cargo test --lib

build:
	cargo build

check:
	cargo run -- check  # run's rst's check on the requirements

version = `sed -En 's/version = "([^"]+)"/\1/p' Cargo.toml`

publish: clippy check test build
	git branch | grep '* master'
	git diff --no-ext-diff --quiet --exit-code
	#cargo publish
	git push origin master
	git tag -a "v{{version}}" -m "v{{version}}"
	git push origin --tags

clippy:
	rustup run nightly cargo clippy

install-clippy:
	rustup run nightly cargo install clippy -f

install-nightly:
	rustup install nightly
