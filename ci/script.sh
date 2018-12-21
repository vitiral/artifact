# This script takes care of testing your crate

set -ex

main() {
    $BTOOL build --target $RTARGET

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    $BTOOL test --target $RTARGET

    $BTOOL run --target $RTARGET -- help
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
