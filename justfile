version = `sed -En 's/version = "([^"]+)"/\1/p' Cargo.toml`

target = "$PWD/target"

build-rust:
	CARGO_TARGET_DIR={{target}}/stable rustup run stable cargo build --features "web"

build-web:
	(cd web-ui; npm run build)
	(cd web-ui/dist; tar -cvf ../../src/api/web-ui.tar *)

build: build-web build-rust 

test-rust:
	RUST_BACKTRACE=1 CARGO_TARGET_DIR={{target}}/stable rustup run stable cargo test --lib --features "web"

test-web: build-web
	(cd web-ui; elm test)

test: test-web test-rust

filter PATTERN:
	RUST_BACKTRACE=1 CARGO_TARGET_DIR={{target}}/stable rustup run stable cargo test --lib {{PATTERN}} --features "web"

clippy:
	CARGO_TARGET_DIR={{target}}/nightly rustup run nightly cargo clippy --features "web"

example-server: build-web
	(CARGO_TARGET_DIR={{target}}/nightly rustup run nightly cargo run --features "web" -- --work-tree web-ui/e2e_tests/ex_proj -v server)

test-e2e:
	(cd web-ui; py2t e2e_tests/basic.py)

server: build-web
	CARGO_TARGET_DIR={{target}}/nightly rustup run nightly cargo run --features "web" -- -v server

check:
	# TODO: replace with just using the binary
	cargo run --features "web" -- check  # run's rst's check on the requirements

check-all: clippy test check
	echo "checked all"

git-verify:
	git branch | grep '* master'
	git diff --no-ext-diff --quiet --exit-code

publish: git-verify build-web clippy test check build
	git commit -a -m "v{{version}} release"
	cargo publish --no-verify
	git push origin master
	git tag -a "v{{version}}" -m "v{{version}}"
	git push origin --tags

publish-fast:
	cargo publish --no-verify
	git push origin master
	git tag -a "v{{version}}" -m "v{{version}}"
	git push origin --tags

install-clippy:
	rustup run nightly cargo install clippy -f

install-nightly:
	rustup install nightly

