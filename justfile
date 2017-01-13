version = `sed -En 's/version = "([^"]+)"/\1/p' Cargo.toml`

target = "$PWD/target"

build: # build with web=false
	CARGO_TARGET_DIR={{target}}/stable rustup run stable cargo build

api: 
	CARGO_TARGET_DIR={{target}}/stable rustup run stable cargo run -- -v server

build-elm:
	(cd web-ui; npm run build)
	(cd web-ui/dist; tar -cvf ../../src/api/web-ui.tar *)

build-web: build-elm
	CARGO_TARGET_DIR={{target}}/stable rustup run stable cargo build --features "web"

build-all: build build-web # just used for testing that you can build both

test: # do tests with web=false
	RUST_BACKTRACE=1 CARGO_TARGET_DIR={{target}}/stable rustup run stable cargo test --lib

test-web:
	(cd web-ui; elm test)
	RUST_BACKTRACE=1 CARGO_TARGET_DIR={{target}}/stable rustup run stable cargo test --lib --features "web"

test-all: test test-web

filter PATTERN:
	RUST_BACKTRACE=1 CARGO_TARGET_DIR={{target}}/stable rustup run stable cargo test --lib {{PATTERN}} --features "web"

clippy:
	CARGO_TARGET_DIR={{target}}/nightly rustup run nightly cargo clippy --features "web"

example-server: build-elm
	(CARGO_TARGET_DIR={{target}}/nightly rustup run nightly cargo run --features "web" -- --work-tree web-ui/e2e_tests/ex_proj -v server)

test-e2e:
	(cd web-ui; py2t e2e_tests/basic.py)

server: build-elm
	CARGO_TARGET_DIR={{target}}/nightly rustup run nightly cargo run --features "web" -- -v server

check:
	rst check

self-check:
	CARGO_TARGET_DIR={{target}}/nightly rustup run nightly cargo run -- check

check-all: clippy test-all check
	echo "checked all"

git-verify:
	git branch | grep '* master'
	git diff --no-ext-diff --quiet --exit-code

publish: git-verify clippy test-all build-all check
	git commit -a -m "v{{version}} release"
	just publish-fast

publish-fast:
	cargo publish --no-verify
	git push origin master
	git tag -a "v{{version}}" -m "v{{version}}"
	git push origin --tags

update: 
	rustup update
	rustup run nightly cargo install clippy -f

install-nightly:
	rustup install nightly

