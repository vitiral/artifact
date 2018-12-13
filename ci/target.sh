if [ $TRAVIS_OS_NAME = linux ]; then
    export TARGET=x86_64-unknown-linux-gnu
else
    export TARGET=x86_64-apple-darwin
fi

export RELEASE_NAME="$CRATE_NAME-$TRAVIS_TAG-$TARGET"

echo TARGET=$TARGET
echo RELEASE_NAME=$RELEASE_NAME
