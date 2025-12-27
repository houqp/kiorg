{ selfPath, ... }:
{
  perSystem =
    {
      pkgs,
      config,
      lib,
      ...
    }:
    {
      packages =
        let
          mkProjectPath = path: (selfPath + builtins.unsafeDiscardStringContext path);
          mkPackage = (
            buildType:
            pkgs.rustPlatform.buildRustPackage (
              final:
              let
                # NOTE: SHOULD BE READ FROM Cargo.toml or some `./VERSION`
                version = "1.4.0-main";
              in
              {
                pname = "kiorg";
                inherit version;

                inherit buildType;

                src = lib.fileset.toSource {
                  root = selfPath;
                  fileset = (
                    lib.fileset.unions [
                      (mkProjectPath "/Cargo.toml")
                      (mkProjectPath "/Cargo.lock")
                      (mkProjectPath "/crates")
                      (mkProjectPath "/plugins")
                      (mkProjectPath "/assets")
                    ]
                  );
                };

                buildInputs = [
                  pkgs.openssl
                  pkgs.fontconfig
                  pkgs.libxkbcommon
                ];

                nativeBuildInputs = [
                  pkgs.pkg-config
                  pkgs.rustPlatform.bindgenHook
                  pkgs.autoPatchelfHook
                ];

                autoPatchelfIgnoreMissingDeps = [ "libgcc_s.so.1" ];
                runtimeDependencies = [
                  pkgs.wayland
                  pkgs.libxkbcommon
                  pkgs.libGL
                ];

                env = {
                  # Ugly
                  PDFIUM_DYNAMIC_LIB_PATH = "${pkgs.pdfium-binaries}/lib/libpdfium.so";
                  PDFIUM_INCLUDE_PATH = "${pkgs.pdfium-binaries}/include";
                };

                doCheck = false;

                cargoDeps = pkgs.rustPlatform.importCargoLock {
                  lockFile = (mkProjectPath "/Cargo.lock");
                  # NOTE: This is fine for a flake like this but should not be used in environments like nixpkgs.
                  allowBuiltinFetchGit = true;
                };

                meta = {
                  description = "A hacker's file manager with VIM inspired keybind.";
                  homepage = "https://github.com/houqp/kiorg";
                  license = lib.licenses.mit;
                  maintainers = [ ];
                };
              }
            )
          );
        in
        {
          debug = mkPackage "debug";
          release = mkPackage "release";
          default = config.packages.release;
        };
    };
}
