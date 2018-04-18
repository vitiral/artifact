
with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "rust-env";
  buildInputs = [
    rustc cargo

    # dependencies
    graphviz
    emscripten
  ];

  RUST_BACKTRACE=1;
}

