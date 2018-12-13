# This script takes care of testing your crate

set -ex

main() {
    btool build --target $TARGET

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    btool test --target $TARGET

    btool run --target $TARGET -- help
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
