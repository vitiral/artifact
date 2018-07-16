set -ex

main() {
    local target=
    if [ $TRAVIS_OS_NAME = linux ]; then
        target=x86_64-unknown-linux-musl
        sort=sort
    else
        target=x86_64-apple-darwin
        sort=gsort  # for `sort --sort-version`, from brew's coreutils.
    fi

    # Builds for iOS are done on OSX, but require the specific target to be
    # installed.
    case $TARGET in
        aarch64-apple-ios)
            rustup target install aarch64-apple-ios
            ;;
        armv7-apple-ios)
            rustup target install armv7-apple-ios
            ;;
        armv7s-apple-ios)
            rustup target install armv7s-apple-ios
            ;;
        i386-apple-ios)
            rustup target install i386-apple-ios
            ;;
        x86_64-apple-ios)
            rustup target install x86_64-apple-ios
            ;;
    esac

    # This fetches latest stable release
    local tag=$(git ls-remote --tags --refs --exit-code https://github.com/japaric/cross \
                       | cut -d/ -f3 \
                       | grep -E '^v[0.1.0-9.]+$' \
                       | $sort --version-sort \
                       | tail -n1)
    curl -LSfs https://japaric.github.io/trust/install.sh | \
        sh -s -- \
           --force \
           --git japaric/cross \
           --tag $tag \
           --target $target

    # CARGO WEB
    CARGO_WEB_RELEASE=$(curl -L -s -H 'Accept: application/json' https://github.com/koute/cargo-web/releases/latest)
    CARGO_WEB_VERSION=$(echo $CARGO_WEB_RELEASE | sed -e 's/.*"tag_name":"\([^"]*\)".*/\1/')
    CARGO_WEB_URL="https://github.com/koute/cargo-web/releases/download/$CARGO_WEB_VERSION/cargo-web-x86_64-unknown-linux-gnu.gz"

    echo "Downloading cargo-web from: $CARGO_WEB_URL"
    curl -L $CARGO_WEB_URL | gzip -d > cargo-web
    chmod +x cargo-web

    mkdir -p ~/.cargo/bin
    mv cargo-web ~/.cargo/bin

    # MDBOOK
    cargo install mdbook || echo "mdbook already installed"
}

main
