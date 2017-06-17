# justfile
# see: https://github.com/casey/just

##################################################
# constants
version = `sed -En 's/version = "([^"]+)"/\1/p' Cargo.toml | head -n1`
target = "$PWD/target"
target_nightly = target + "/nightly"
target_nightly_app = target_nightly + "/debug/art"
rustup_nightly = " CARGO_INCREMENTAL=1 rustup run nightly"
nightly = "CARGO_TARGET_DIR=" + target_nightly + rustup_nightly
export_nightly = "export TARGET_BIN=" + target_nightly_app + " && "

python_dirs="web-ui/sel_tests scripts"

echo-version:
	echo {{version}}

##################################################
# build commands

build:
	just web-ui/build 
	just build-rust

build-rust:
	{{nightly}} cargo build --features server
	@echo "-- built binary to: {{target_nightly_app}}"

# current "release" build includes only exporting static html
build-static: 
	just web-ui/build-static
	just build


##################################################
# unit testing/linting commands
test TESTS="":
	@just web-ui/test
	{{nightly}} cargo test --lib --features server {{TESTS}}

lint: # run linter
	{{nightly}} cargo clippy --features server
	
test-sel: # run sel tests, still in development
	just build
	just test-sel-py

test-sel-py: # run sel tests, still in development
	{{export_nightly}} py.test web-ui/sel_tests/basic.py -sx

@test-all:
	just check-fmt
	art check
	just lint
	just test
	just test-sel


##################################################
# running commands

# run with `just run -- {{args}}`
run ARGS="": # run the api server (without the web-ui)
	{{nightly}} cargo run -- -v {{ARGS}}

serve-rust: 
	{{nightly}} cargo run --features server -- -vv serve

serve: # run the full frontend
	just web-ui/build
	just serve-rust

self-check: # build self and run `art check` using own binary
	{{nightly}} cargo run -- check


##################################################
# release command

fmt-rust:
	cargo fmt -- --write-mode overwrite  # don't generate *.bk files
	art fmt -w

fmt-py:
    autopep8 {{python_dirs}} -r --in-place
    docformatter {{python_dirs}} -r --in-place

fmt:
	just fmt-rust
	just fmt-py
	just web-ui/fmt

check-fmt:
	cargo fmt -- --write-mode=diff

git-verify: # make sure git is clean and on master
	git branch | grep '* master'
	git diff --no-ext-diff --quiet --exit-code

# publish to github and crates.io
publish: 
	just git-verify lint build-static
	just lint test self-check
	git commit -a -m "v{{version}} release"
	just publish-cargo publish-git

export-site: build-static
	rm -rf _gh-pages/index.html _gh-pages/css
	{{nightly}} cargo run -- export html && mv index.html css _gh-pages

publish-site: export-site
	rm -rf _gh-pages/index.html _gh-pages/css
	{{nightly}} cargo run -- export html && mv index.html css _gh-pages
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
	cargo install just -f
	rustup run nightly cargo install rustfmt-nightly -f
	rustup run nightly cargo install clippy -f

install-nightly:
	rustup install nightly
