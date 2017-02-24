# `script` phase: you usually build, test and generate docs in this phase

set -ex

. $(dirname $0)/utils.sh

# DONE modify this function as you see fit
# PROTIP Always pass `--target $TARGET` to cargo commands, this makes cargo output build artifacts
# to target/$TARGET/{debug,release} which can reduce the number of needed conditionals in the
# `before_deploy`/packaging phase
run_test_suite() {
	## DONE: disable anything related to quemu because it kills me
    #case $TARGET in
    #    # configure emulation for transparent execution of foreign binaries
    #    aarch64-unknown-linux-gnu)
    #        export QEMU_LD_PREFIX=/usr/aarch64-linux-gnu
    #        ;;
    #    arm*-unknown-linux-gnueabihf)
    #        export QEMU_LD_PREFIX=/usr/arm-linux-gnueabihf
    #        ;;
    #    *)
    #        ;;
    #esac

    #if [ ! -z "$QEMU_LD_PREFIX" ]; then
    #    # Run tests on a single thread when using QEMU user emulation
    #    export RUST_TEST_THREADS=1
    #fi

    cargo fmt -- --write-mode=diff
    cargo build --target $TARGET --verbose  # make sure it builds without web, but don't test
    cargo build --target $TARGET --verbose
    cargo run --target $TARGET -- check
    cargo test --target $TARGET

    # sanity check the file type
    file target/$TARGET/debug/art
}

main() {
    run_test_suite
}

main
