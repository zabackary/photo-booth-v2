{
  description = "Photo Booth V2 - Rust project";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
  };

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      perSystem = nixpkgs.lib.genAttrs systems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };

          isLinux = pkgs.stdenv.isLinux;
          isDarwin = pkgs.stdenv.isDarwin;
          lib = pkgs.lib;

          cargoToml = lib.importTOML ./Cargo.toml;

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

          envVars = {
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
            BINDGEN_EXTRA_CLANG_ARGS =
              if isLinux then
                "-I${pkgs.linuxHeaders}/include -I${pkgs.glibc.dev}/include -I${pkgs.cups.dev}/include"
              else
                "";
          };
        in
        {
          packages.default = pkgs.rustPlatform.buildRustPackage (
            {
              pname = cargoToml.package.name;
              version = cargoToml.package.version;
              src = ./.;

              cargoLock = {
                lockFile = ./Cargo.lock;
                # nokhwa is pulled from git; every crate vendored from that
                # same repo+rev shares this one hash.
                outputHashes = {
                  "nokhwa-0.10.10" = "sha256-2mrGDmYOQom2UXc+8rJOkurWu4u/JL2/Jx6CA6mqrnU=";
                };
              };

              inherit nativeBuildInputs buildInputs;

              postInstall = ''
                wrapProgram "$out/bin/${cargoToml.package.name}" --prefix LD_LIBRARY_PATH : ${libPath} --prefix DYLD_LIBRARY_PATH : ${libPath}
              '';
            }
            // envVars
          );

          devShells.default = pkgs.mkShell (
            {
              inherit nativeBuildInputs;
              buildInputs = [
                pkgs.cargo
                pkgs.rustc
                pkgs.rustfmt
                pkgs.clippy
                pkgs.rust-analyzer
              ]
              ++ buildInputs;

              RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
              LD_LIBRARY_PATH = libPath;
            }
            // envVars
          );
        }
      );
    in
    {
      packages = nixpkgs.lib.mapAttrs (_: v: v.packages) perSystem;
      devShells = nixpkgs.lib.mapAttrs (_: v: v.devShells) perSystem;
    };
}
