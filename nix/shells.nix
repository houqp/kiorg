{ ... }:
{
  perSystem =
    {
      pkgs,
      config,
      lib,
      ...
    }:
    {
      devShells.default = pkgs.mkShell {
        strictDeps = true;
        nativeBuildInputs = [
          pkgs.cargo
          pkgs.rustc
          pkgs.llvmPackages.bintools
          pkgs.rustPlatform.bindgenHook
          pkgs.pkg-config
        ];

        inherit (config.packages.default) buildInputs;

        env =
          let
            runtimeInputs = [
              pkgs.xorg.libX11
              pkgs.xorg.libXcursor
              pkgs.xorg.libXi
              pkgs.libxkbcommon
              pkgs.libGL
              pkgs.freetype
              pkgs.wayland
            ];
          in
          {
            LD_LIBRARY_PATH = lib.makeLibraryPath runtimeInputs;
          };
      };
    };
}
