# This script takes care of testing your crate

set -ex

main() {
    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    if [ "$CI_BUILD" = "fast" ]; then
        echo "Only doing fast build and test"
        cargo test
        return 0
    fi

    export RUST_BACKTRACE=1
    # TODO: just lint
    # TODO: just lint -- "--features beta"
    cargo test
    # TODO: cargo test --features beta
    # same command that is used in release
    # TODO: cross rustc --bin art --target $TARGET --release -- -C lto
    # TODO: export TARGET_BIN="target/$TARGET/release/art"
    # test "$(uname)" = "Darwin" && echo "TODO: selenium timeout issue on mac" || \
    #     py.test web-ui/sel_tests
    # TODO: just check-fmt
    # eval "$TARGET_BIN check"
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
