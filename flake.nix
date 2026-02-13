{
  description = "Photo Booth V2 - Rust project";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      flake-utils,
      naersk,
      nixpkgs,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        nativeBuildInputs = with pkgs; [
          pkg-config
          libclang
          linuxHeaders
          glibc.dev
        ];
        buildInputs = with pkgs; [
          openssl
          libv4l
        ];
        naersk-lib = naersk.lib."${system}";
      in
      {
        packages.default = naersk-lib.buildPackage {
          inherit nativeBuildInputs buildInputs;
          src = ./.;
        };

        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs;
          buildInputs = [
            (pkgs.rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ]; # Required for rust-analyzer to work
            })
          ]
          ++ buildInputs;

          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.linuxHeaders}/include -I${pkgs.glibc.dev}/include";
        };
      }
    );
}
