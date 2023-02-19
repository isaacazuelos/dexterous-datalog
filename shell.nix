# Taken from the wiki at https://nixos.wiki/wiki/Rust

# The clang override fixes an issue with #[should_panic] tests breaking
# It comes from https://godot-rust.github.io/book/gdnative/recipes/nix-build-system.html

{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell.override { stdenv = pkgs.clangStdenv; } {
  nativeBuildInputs = with pkgs; [ rustc cargo rustfmt clippy libiconv ];
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
