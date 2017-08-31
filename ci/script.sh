# This script takes care of testing your crate

set -ex

main() {
    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    if [ -z "$CI_BUILD_FAST" ]; then
        echo "Only doing fast build and test"
        cargo test --features server
        return 0
    fi

    export RUST_BACKTRACE=1
    just lint
    cargo test
    just test
    just build-release
    export TARGET_BIN="$PWD/target/release/art"
    # test "$(uname)" = "Darwin" && echo "TODO: selenium timeout issue on mac" || \
    #     py.test web-ui/sel_tests
    just check-fmt
    art check
    just run -- check
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
