{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  buildInputs = [
    pkgs.openssl.dev
    pkgs.pkg-config
    pkgs.rust-analyzer
    pkgs.rustup
  ];
}
