{
  description = "Development shell for Kiorg";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          rustc
          cargo
          rustfmt
          clippy
          rust-analyzer

          stdenv.cc
          binutils
          pkg-config
          cmake
          gnumake
          llvmPackages.clang
          llvmPackages.libclang
          perl

          wayland
          wayland-protocols
          libxkbcommon

          libGL
          libglvnd
          vulkan-loader

          fontconfig
          freetype
          openssl
          zlib

          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr

          git
          patchelf
          lld
        ];

        CC = "${pkgs.stdenv.cc}/bin/cc";
        CXX = "${pkgs.stdenv.cc}/bin/c++";
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

        RUST_BACKTRACE = "1";

        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
          wayland
          libxkbcommon
          libGL
          libglvnd
          vulkan-loader
          fontconfig
          freetype
          openssl
          zlib
          stdenv.cc.cc.lib
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ]);
      };
    };
}
