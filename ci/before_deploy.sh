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

    unset CARGO_INCREMENTAL
    rustup default stable
    test -f Cargo.lock || cargo generate-lockfile

    # build the artifacts that matter to you
    cross rustc --bin art --target $TARGET --release -- -C lto

    # package the right artifacts
    cp target/$TARGET/release/art $stage/

    cd $stage
    tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
    cd $src

    rm -rf $stage

    # switch back
    rustup default $RUST_VERSION
}

main
