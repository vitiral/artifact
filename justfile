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
	#rustup run stable cargo build --release --target=x86_64-unknown-linux-musl
	#cargo publish
	git push origin master
	git tag -a "v{{version}}" -m "v{{version}}"
	git push origin --tags
	#ci/github-release release --user vitiral --repo rst --tag "v{{version}}" --name "rst beta v{{version}}" --pre-release
	#ci/github-release upload --user vitiral --repo rst --tag v{{version}} --name "rst-x86_64-unknown-linux-musl" --file "target/x86_64-unknown-linux-musl/release/rst"

install-clippy:
	rustup run nightly cargo install clippy -f

install-nightly:
	rustup install nightly

