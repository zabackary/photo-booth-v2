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

        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

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
            with pkgs;
            [
              libgphoto2
            ];

        # buildInputs contains libraries needed to compile/link and at runtime
        buildInputs =
          with pkgs;
          (
            [
              openssl
            ]
            ++ (lib.optionals isLinux [

              libv4l
            ])
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

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };
      in
      {
        packages.default = naersk'.buildPackage {
          inherit nativeBuildInputs buildInputs;
          propagatedBuildInputs = runtimeLibs;
          src = ./.;
          postInstall = ''
            wrapProgram "$out/bin/photo-booth-v2" --prefix LD_LIBRARY_PATH : ${libPath}
          '';

          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS =
            if isLinux then "-I${pkgs.linuxHeaders}/include -I${pkgs.glibc.dev}/include" else "";
        };

        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs;
          buildInputs = [
            toolchain
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
