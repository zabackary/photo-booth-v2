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
    flake-utils.lib.eachSystem
      [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" "x86_64-pc-windows-msvc" ]
      (
        system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };

          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

          isLinux = pkgs.stdenv.isLinux;
          isDarwin = pkgs.stdenv.isDarwin;
          lib = pkgs.lib;

          # compile-time (native) tools
          nativeBuildInputs =
            with pkgs;
            lib.concatMap (x: x) [
              (lib.optionals isLinux [
                linuxHeaders
                glibc.dev
              ])
              [
                pkg-config
                libclang
                mold-wrapped
                makeWrapper
              ]
            ];

          # buildInputs contains libraries needed to compile/link and at runtime
          buildInputs =
            with pkgs;
            (
              [
                openssl
                libgphoto2
                cups.dev
              ]
              ++ (lib.optionals isLinux [
                # graphics stuff
                libGL
                libxkbcommon
                vulkan-loader
                wayland
                xorg.libXcursor
                xorg.libXrandr
                xorg.libXi
                xorg.libX11

                # needed for video capture on Linux
                libv4l
              ])
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
            src = ./.;
            postInstall = ''
              wrapProgram "$out/bin/photo-booth-v2" --prefix LD_LIBRARY_PATH : ${libPath} --prefix DYLD_LIBRARY_PATH : ${libPath}
            '';

            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            BINDGEN_EXTRA_CLANG_ARGS =
              if isLinux then
                "-I${pkgs.linuxHeaders}/include -I${pkgs.glibc.dev}/include -I${pkgs.cups.dev}/include"
              else
                "";
          };

          devShells.default = pkgs.mkShell {
            inherit nativeBuildInputs;
            buildInputs = [
              toolchain
            ]
            ++ buildInputs;

            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            BINDGEN_EXTRA_CLANG_ARGS =
              if isLinux then
                "-I${pkgs.linuxHeaders}/include -I${pkgs.glibc.dev}/include -I${pkgs.cups.dev}/include"
              else
                "";
            LD_LIBRARY_PATH = libPath;
          };
        }
      );
}
