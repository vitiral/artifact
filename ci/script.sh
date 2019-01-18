# This script takes care of testing your crate

set -ex

main() {
    $BTOOL build -p artifact-app --target $TARGET

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    $BTOOL test -p artifact-app --target $TARGET

    $BTOOL run -p artifact-app --target $TARGET -- help
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
