# justfile
# see: https://github.com/casey/just

##################################################
# constants
version = `sed -En 's/version = "([^"]+)"/\1/p' Cargo.toml | head -n1`
target = "$PWD/target"
export_bin = "export TARGET_BIN=" + target + "/debug/art" + " &&"
python_dirs="web-ui/sel_tests scripts"

# just get the version
echo-version:
	@echo {{version}}

##################################################
# build commands

# do the standard build, including the full server and static web-ui
build:
	just web-ui/build 
	just build-rust

# build only rust
build-rust:
	cargo build --features server

# build with only static html (not server)
build-static: 
	just web-ui/build-static
	just build


##################################################
# unit testing/linting commands

# run all unit tests
test TESTS="":
	@just web-ui/test
	cargo test --lib --features server {{TESTS}}

# run all lints
lint:
	cargo clippy --features server
	
# build and run selenium tests
test-sel: 
	just build
	just test-sel-py

# run selenium tests
test-sel-py:
	# TODO: add browser type export
	{{export_bin}} py.test web-ui/sel_tests -sx

# run the full test suite
@test-all:
	#just check-fmt
	art check
	just lint
	just test
	just test-sel


##################################################
# running commands

# run the artifact binary with any args
run ARGS="":
	just web-ui/build
	cargo run --features server -- -v {{ARGS}}

##################################################
# release command

# run all formatters
fmt:
	just fmt-rust
	just fmt-py
	just web-ui/fmt

# run rustfmt
fmt-rust:
	cargo fmt -- --write-mode overwrite  # don't generate *.bk files
	art fmt -w

# run python formatters
fmt-py:
    autopep8 {{python_dirs}} -r --in-place
    docformatter {{python_dirs}} -r --in-place

# check that code has been formatted
check-fmt:
	cargo fmt -- --write-mode=diff

# make sure git is clean and on master
git-verify:
	git branch | grep '* master'
	git diff --no-ext-diff --quiet --exit-code

# publish to github and crates.io
publish: 
	# TODO: switch to build when web-ui done
	just git-verify lint build-static
	just lint test self-check
	git commit -a -m "v{{version}} release"
	@# push to cargo
	cargo publish --no-verify
	@#push to git
	git push origin master
	git tag -a "v{{version}}" -m "v{{version}}"
	git push origin --tags

# build the static html
build-site:
	cargo run --features server -- export html -o _gh-pages

# push the static html design docs to git-pages
publish-site: build-site
	(cd _gh-pages; git commit -am 'v{{version}}' && git push origin gh-pages)

##################################################
# developer installation helpers

update: # update rust and tools used by this lib
	cargo install-update -i just
	cargo install-update -i cargo-update
	cargo install-update -i rustfmt-nightly:$RUSTFMT_VERSION
	cargo install-update -i clippy:$RUSTCLIPPY_VERSION
	pip install -r scripts/requirements.txt
	npm install $NPM_PACKAGES --prefix $ENV_DIR
