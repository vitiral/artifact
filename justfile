# this is a comment

test: build
	cargo test --lib

filter PATTERN:
	cargo test --lib {{PATTERN}}

backtrace:
	RUST_BACKTRACE=1 cargo test --lib

build:
	cargo build

clippy:
	rustup run nightly cargo clippy

check:
	cargo run -- check  # run's rst's check on the requirements

check-all: clippy test check
	echo "checked all"

version = `sed -En 's/version = "([^"]+)"/\1/p' Cargo.toml`

publish: clippy test check build
	git branch | grep '* master'
	git diff --no-ext-diff --quiet --exit-code
	#cargo publish
	git push origin master
	git tag -a "v{{version}}" -m "v{{version}}"
	git push origin --tags

install-clippy:
	rustup run nightly cargo install clippy -f

install-nightly:
	rustup install nightly

