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

        isLinux = pkgs.stdenv.isLinux;
        lib = pkgs.lib;

        # compile-time (native) tools
        nativeBuildInputs = lib.concatMap (x: x) [
          (lib.optionals isLinux [
            pkgs.linuxHeaders
            pkgs.glibc.dev
          ])
          [
            pkgs.pkg-config
            pkgs.libclang
            pkgs.mold-wrapped
          ]
        ];

        # runtime / link libraries (only pulled on Linux)
        runtimeLibs =
          if isLinux then
            with pkgs;
            [
              libgphoto2
              libGL
              libxkbcommon
              vulkan-loader
              wayland
              xorg.libXcursor
              xorg.libXrandr
              xorg.libXi
              xorg.libX11
            ]
          else
            [ ];

        # buildInputs contains libraries needed to compile/link and at runtime
        buildInputs =
          with pkgs;
          (
            [
              openssl
              libv4l
            ]
            ++ runtimeLibs
          );

        libPath =
          if isLinux then
            with pkgs;
            lib.makeLibraryPath [
              libGL
              libxkbcommon
              vulkan-loader
              wayland
              xorg.libXcursor
              xorg.libXrandr
              xorg.libXi
              xorg.libX11
            ]
          else
            "";

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
          BINDGEN_EXTRA_CLANG_ARGS =
            if isLinux then "-I${pkgs.linuxHeaders}/include -I${pkgs.glibc.dev}/include" else "";
          LD_LIBRARY_PATH = libPath;
        };
      }
    );
}
