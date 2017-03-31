# justfile
# see: https://github.com/casey/just

##################################################
# constants
version = `sed -En 's/version = "([^"]+)"/\1/p' Cargo.toml | head -n1`
target = "$PWD/target"
nightly = "CARGO_TARGET_DIR=$TG/nightly CARGO_INCREMENTAL=1 rustup run nightly"

echo-version:
	echo {{version}}

doc:
	cargo doc --open


##################################################
# build commands
build-dev: # build using nightly and incremental compilation
	TG={{target}} {{nightly}} cargo build
	echo "built binary to: target/nightly/debug/art"

build-elm: # build just elm (not rust)
	(cd web-ui; npm run build)
	(cd web-ui/dist; tar -cvf ../../src/api/data/web-ui.tar *)

build-static: # build and package elm as a static index.html
	(cd web-ui; elm make src/Main-Static.elm)
	rm -rf target/web
	mkdir target/web
	cp web-ui/index.html target/web
	cp -r web-ui/css target/web
	# copy and link the style sheets
	sed -e 's/<title>Main<\/title>/<title>Design Documents<\/title>/g' target/web/index.html -i
	sed -e 's/<head>/<head><link rel="stylesheet" type="text\/css" href="css\/index.css" \/>/g' target/web/index.html -i
	(cd target/web; tar -cvf ../../src/cmd/data/web-ui-static.tar *)

# full build for a std release. Currently doesn't include the server code
build-full: build-static
	just build-dev


##################################################
# unit testing/linting commands
test: # do tests with web=false
	RUST_BACKTRACE=1 cargo test --lib

test-dev: # test using nightly and incremental compilation
	TG={{target}} {{nightly}} cargo test --lib

test-elm: 
	(cd web-ui; elm test)

test-all: test-elm test

filter PATTERN: # run only specific tests
	RUST_BACKTRACE=1 cargo test --lib {{PATTERN}}

lint: # run linter
	CARGO_TARGET_DIR={{target}}/nightly rustup run nightly cargo clippy
	
test-server-only:
	RUST_BACKTRACE=1 cargo test --lib --features server

test-server: build-elm # run the test-server for e2e testing, still in development
	just test-server-only

test-e2e: # run e2e tests, still in development
	(cd web-ui; py2t e2e_tests/basic.py)


##################################################
# running commands

api: # run the api server (without the web-ui)
	cargo run -- -v server

serve-rust: 
	TG={{target}} {{nightly}} cargo run --features server -- -v serve

serve: build-elm  # run the full frontend
	just serve-rust

self-check: # build self and run `art check` using own binary
	TG={{target}} {{nightly}} cargo run -- check


##################################################
# release command

fmt:
	cargo fmt -- --write-mode overwrite  # don't generate *.bk files
	art fmt -w

check-fmt:
	cargo fmt -- --write-mode=diff

check: check-fmt
	art check

git-verify: # make sure git is clean and on master
	git branch | grep '* master'
	git diff --no-ext-diff --quiet --exit-code

publish: git-verify lint build-full test-all self-check # publish to github and crates.io
	git commit -a -m "v{{version}} release"
	just publish-cargo
	just publish-git

export-site: build-full
	rm -rf _gh-pages/index.html _gh-pages/css
	TG={{target}} {{nightly}} cargo run -- export html && mv index.html css _gh-pages

publish-site: export-site
	rm -rf _gh-pages/index.html _gh-pages/css
	TG={{target}} {{nightly}} cargo run -- export html && mv index.html css _gh-pages
	(cd _gh-pages; git commit -am 'v{{version}}' && git push origin gh-pages)

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
