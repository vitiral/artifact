# This script takes care of testing your crate

set -ex

main() {
    cargo build --target $TARGET

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cargo test --target $TARGET

    cargo run --target $TARGET -- help
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
