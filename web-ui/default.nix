
with import <nixpkgs> {};

stdenv.mkDerivation {
 name = "artifact-elm";
 src = ./.;
 buildInputs = [ elmPackages.elm nodejs ];

 patchPhase = ''
   patchShebangs node_modules/webpack
 '';

 buildPhase = ''
   npm run build
 '';

 installPhase = ''
   mkdir $out
   cp dist/* $out/
 '';
}
