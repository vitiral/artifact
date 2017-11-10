# justfile
# see: https://github.com/casey/just

##################################################
# constants
version = `sed -En 's/version = "([^"]+)"/\1/p' Cargo.toml | head -n1`
target = "$PWD/target"
export_bin = "export TARGET_BIN=" + target + "/debug/art" + " &&"
repo_url = "https://github.com/vitiral/artifact"
build_site_args = '--path-url "' + repo_url + '/blob/$(git rev-parse --verify HEAD)/{path}#L{line}"'
pre = 'CARGO_INCREMENTAL=1'
art = 'target/debug/art'
beta = "--features beta"


# just get the version
echo-version:
	@echo {{version}}

##################################################
# build commands

# do the standard build, including the full server and static web-ui
build FEATURES="":
	just web-ui/build
	just build-rust -- "{{FEATURES}}"

# build in release mode
build-release FEATURES="":
	just web-ui/build
	cargo build {{FEATURES}} --release

# build only rust
build-rust FEATURES="":
	{{pre}} cargo build {{FEATURES}}

# build with only static html (not server)
build-static FEATURES="":
	just web-ui/build-static
	just build -- "{{FEATURES}}"

##################################################
# unit testing/linting commands

# run all unit tests
test FEATURES="" TESTS="":
	@just test-rust -- "{{FEATURES}}" "{{TESTS}}"

# test only the rust code
test-rust FEATURES="" TESTS="":
	{{pre}} cargo test --lib {{FEATURES}} {{TESTS}} -- --nocapture

# run all lints
lint FEATURES="":
	just lint-rust -- "{{FEATURES}}"
	just lint-py
	just web-ui/lint

# lint rust code
lint-rust FEATURES="":
	{{pre}} cargo clippy {{FEATURES}}

# lint python code
lint-py:
	@echo "pylint $PYTHON_CHECK"
	@pylint $PYTHON_CHECK

# build and run selenium tests
test-sel FEATURES="" TESTS="":
	just build -- "{{FEATURES}}"
	just test-sel-py -- "{{TESTS}}"

# run selenium tests
test-sel-py TESTS="":
	{{export_bin}} py.test web-ui/sel_tests/{{TESTS}} -svvv


# run the full test suite. both beta and non-beta are requied for merge
@test-all FEATURES="":
	just lint
	{{pre}} cargo test
	just check-fmt
	{{art}} check
	just test
	just test-sel

@test-all-beta:
	just test-all -- "{{beta}}"

# run all formatters in "check" mode to make sure code has been formatted
check-fmt:
	{{pre}} cargo fmt -- --write-mode=diff > /dev/null 2>&1
	case "$(autopep8 $PYTHON_CHECK -r --diff)" in ("") true;; (*) false;; esac
	case "$(docformatter $PYTHON_CHECK -r)" in ("") true;; (*) false;; esac
	just web-ui/check-fmt
	{{art}} fmt -d > /dev/null 2>&1

##################################################
# running commands

# run the artifact binary with any args
run FEATURES="" ARGS="":
	just web-ui/build
	{{pre}} cargo run {{FEATURES}} -- -v {{ARGS}}

serve:
	just web-ui/build
	just serve-rust

serve-rust:
	rm -rf /tmp/rust-serve && cp -r web-ui/sel_tests/ex_proj /tmp/rust-serve
	{{pre}} cargo run -- -vv --work-tree /tmp/rust-serve serve


##################################################
# release command

# run all formatters
fmt:
	just fmt-rust
	just fmt-py
	just web-ui/fmt
	{{art}} fmt -w

# run rust formatter
fmt-rust:
	{{pre}} cargo fmt -- --write-mode overwrite  # don't generate *.bk files

# run python formatters
fmt-py:
    autopep8 $PYTHON_CHECK -r --in-place
    docformatter $PYTHON_CHECK -r --in-place

# publish to github and crates.io
publish:
	@# make sure code is clean on master
	git branch | grep '* master'
	git diff --no-ext-diff --quiet --exit-code
	@# test all and commit
	just test-all
	{{pre}} rustup run stable cargo test
	just build
	git commit -a -m "relase {{version}}"
	@# push to cargo
	just publish-finish

publish-finish:
	{{pre}} cargo publish --no-verify
	@#push to git
	git push origin master
	git tag -a "{{version}}" -m "{{version}}"
	git push origin --tags
	@#update docs
	just publish-site || echo "no changes to site"

# build the static html
build-site:
	{{pre}} cargo run -- -vv export html -o _gh-pages {{build_site_args}}

# push the static html design docs to git-pages
publish-site:
	{{art}} -vv export html -o _gh-pages {{build_site_args}}
	(cd _gh-pages; git commit -am 'v{{version}}' && git push origin gh-pages)

##################################################
# developer installation helpers

# update all developer build/test/lint/etc tools
update:
	pip install -r scripts/requirements.txt
	just web-ui/update
	cargo install-update -i just
	cargo install-update -i cargo-update
	cargo install-update -i rustfmt-nightly:$RUSTFMT_VERSION
	cargo install-update -i clippy:$RUSTCLIPPY_VERSION
