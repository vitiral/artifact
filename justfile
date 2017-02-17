# https://github.com/casey/just

##################################################
# constants
version = `sed -En 's/version = "([^"]+)"/\1/p' Cargo.toml`
target = "$PWD/target"
nightly = "CARGO_TARGET_DIR=$TG/nightly CARGO_INCREMENTAL=1 rustup run nightly"

##################################################
# build commands
build: # build app with web=false
	cargo build
	echo "built binary to: target/stable/debug/art"

build-elm: # build just elm (not rust)
	(cd web-ui; npm run build)
	(cd web-ui/dist; tar -cvf ../../src/api/data/web-ui.tar *)

build-elm-static: # build and package elm as a static index.html
	(cd web-ui; elm make src/Main-Static.elm)
	rm -rf target/web
	mkdir target/web
	mkdir target/web/css
	cp web-ui/index.html target/web
	# copy and link the style sheets
	cp web-ui/node_modules/font-awesome web-ui/src/style.css target/web/css -r
	sed -e 's/<head>/<head><link rel="stylesheet" type="text\/css" href="css\/style.css" \/>/g' target/web/index.html -i
	sed -e 's/<head>/<head><link rel="stylesheet" type="text\/css" href="css\/font-awesome\/css\/font-awesome.css" \/>/g' target/web/index.html -i
	(cd target/web; tar -cvf ../../src/cmd/data/web-ui-static.tar *)

build-web: build-elm build-elm-static
	cargo build --features "web"

##################################################
# unit testing/linting commands
test: # do tests with web=false
	RUST_BACKTRACE=1 cargo test --lib

test-dev: # test using nightly and incremental compilation
	TG={{target}} {{nightly}} cargo test --lib

test-web: # do tests with web=true
	(cd web-ui; elm test)
	RUST_BACKTRACE=1 cargo test --lib --features "web"

filter PATTERN: # run only specific tests
	RUST_BACKTRACE=1 cargo test --lib {{PATTERN}} --features "web"

lint: # run linter
	CARGO_TARGET_DIR={{target}}/nightly rustup run nightly cargo clippy --features "web"
	
test-server: build-elm # run the test-server for e2e testing, still in development
	(cargo run --features "web" -- --work-tree web-ui/e2e_tests/ex_proj -v server)

test-e2e: # run e2e tests, still in development
	(cd web-ui; py2t e2e_tests/basic.py)

##################################################
# running commands

api: # run the api server (without the web-ui)
	cargo run -- -v server

serve: build-elm  # run the full frontend
	cargo run --features "web" -- -v server

self-check: # build self and run `art check` using own binary
	cargo run -- check

##################################################
# release command

clean: 
	find . -name "*.bk" -type f -delete

fmt: clean
	cargo fmt
	just clean

check-fmt: clean
	cargo fmt -- --write-mode=diff

check-art: clean
	art check

check: check-art check-fmt

git-verify: # make sure git is clean and on master
	git branch | grep '* master'
	git diff --no-ext-diff --quiet --exit-code

publish: git-verify lint build-web test-web check # publish to github and crates.io
	git commit -a -m "v{{version}} release"
	just publish-cargo
	just publish-git

publish-cargo: # publish cargo without verification
	cargo publish --no-verify

publish-git: # publish git without verification
	git push origin master
	git tag -a "v{{version}}" -m "v{{version}}"
	git push origin --tags


##################################################
# developer installation helpers

update: # update rust and tools used by this lib
	rustup update
	(cargo install just -f)
	(cargo install rustfmt -f)
	rustup run nightly cargo install clippy -f

install-nightly:
	rustup install nightly
