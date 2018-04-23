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

# just get the version
echo-version:
	@echo Version: {{version}}

build:
	cargo +nightly build
