# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    # $BTOOL rustc --bin art --target $TARGET --release -- -C lto
    $BTOOL build -p artifact-app --target $TARGET --release

    cp target/$TARGET/release/art $stage/

    cd $stage
    tar czf $src/${RELEASE_NAME}.tar.gz *
    cd $src
    ls -al

    rm -rf $stage
}

main
