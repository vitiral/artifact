version = `sed -En 's/version = "([^"]+)"/\1/p' Cargo.toml`

target = "$PWD/target"

build:
	CARGO_TARGET_DIR={{target}}/stable rustup run stable cargo build --features "web"

test:
	RUST_BACKTRACE=1 CARGO_TARGET_DIR={{target}}/stable rustup run stable cargo test --lib --features "web"

filter PATTERN:
	RUST_BACKTRACE=1 CARGO_TARGET_DIR={{target}}/stable rustup run stable cargo test --lib {{PATTERN}} --features "web"

clippy:
	CARGO_TARGET_DIR={{target}}/nightly rustup run nightly cargo clippy --features "web"

server:
	CARGO_TARGET_DIR={{target}}/nightly rustup run nightly cargo run --features "web" -- -v server

check:
	# TODO: replace with just using the binary
	cargo run --features "web" -- check  # run's rst's check on the requirements

check-all: clippy test check
	echo "checked all"

publish: clippy test check build
	git branch | grep '* master'
	git diff --no-ext-diff --quiet --exit-code
	cargo publish
	git push origin master
	git tag -a "v{{version}}" -m "v{{version}}"
	git push origin --tags

install-clippy:
	rustup run nightly cargo install clippy -f

install-nightly:
	rustup install nightly

