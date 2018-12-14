set -ex

# install_github_release USER NAME EXT
install_github_release() {
    USER=$1
    NAME=$2
    EXT=$3

    URL=https://github.com/$USER/$NAME/releases

    APP_RELEASE=$(curl -L -s -H 'Accept: application/json' $URL/latest)
    APP_VERSION=$(echo $APP_RELEASE | sed -e 's/.*"tag_name":"\([^"]*\)".*/\1/')
    APP_FNAME=$NAME-$TARGET.$EXT
    APP_URL="$URL/download/$APP_VERSION/$APP_FNAME"

    echo "Downloading $NAME from: $APP_URL"
    curl -L $APP_URL | gzip -d > $NAME
    chmod +x $NAME

    mkdir -p ~/.cargo/bin
    mv $NAME ~/.cargo/bin
}

install_cargo_web() {
    CARGO_WEB_RELEASE=$(curl -L -s -H 'Accept: application/json' https://github.com/koute/cargo-web/releases/latest)
    CARGO_WEB_VERSION=$(echo $CARGO_WEB_RELEASE | sed -e 's/.*"tag_name":"\([^"]*\)".*/\1/')
    CARGO_WEB_URL="https://github.com/koute/cargo-web/releases/download/$CARGO_WEB_VERSION/cargo-web-x86_64-unknown-linux-gnu.gz"

    echo "Downloading cargo-web from: $CARGO_WEB_URL"
    curl -L $CARGO_WEB_URL | gzip -d > cargo-web
    chmod +x cargo-web

    mkdir -p ~/.cargo/bin
    mv cargo-web ~/.cargo/bin
}


main() {
    # local target=
    # if [ $TRAVIS_OS_NAME = linux ]; then
    #     target=x86_64-unknown-linux-musl
    #     sort=sort
    # else
    #     target=x86_64-apple-darwin
    #     sort=gsort  # for `sort --sort-version`, from brew's coreutils.
    # fi

    # # Builds for iOS are done on OSX, but require the specific target to be
    # # installed.
    # case $TARGET in
    #     aarch64-apple-ios)
    #         rustup target install aarch64-apple-ios
    #         ;;
    #     armv7-apple-ios)
    #         rustup target install armv7-apple-ios
    #         ;;
    #     armv7s-apple-ios)
    #         rustup target install armv7s-apple-ios
    #         ;;
    #     i386-apple-ios)
    #         rustup target install i386-apple-ios
    #         ;;
    #     x86_64-apple-ios)
    #         rustup target install x86_64-apple-ios
    #         ;;
    #     *)
    #         rustup target install $TARGET
    #         ;;
    # esac
    rustup target add $TARGET
    rustup target add wasm32-unknown-unknown

    # # This fetches latest stable release
    # local tag=$(git ls-remote --tags --refs --exit-code https://github.com/japaric/cross \
    #                    | cut -d/ -f3 \
    #                    | grep -E '^v[0.1.0-9.]+$' \
    #                    | $sort --version-sort \
    #                    | tail -n1)
    # curl -LSfs https://japaric.github.io/trust/install.sh | \
    #     sh -s -- \
    #        --force \
    #        --git japaric/cross \
    #        --tag $tag \
    #        --target $target

    cargo install cargo-web --debug || echo "cargo-web already installed"
    cargo install mdbook --debug || echo "mdbook already installed"

    # if [ $TRAVIS_OS_NAME = windows ]; then
    #     install_github_release rust-lang-nursery mdbook "zip"
    # else
    #     install_github_release koute cargo-web "gz"
    #     install_github_release rust-lang-nursery mdbook "tgz"
    # fi
}

main
