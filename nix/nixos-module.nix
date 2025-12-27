{ withSystem, ... }:
{
  lib,
  config,
  pkgs,
  ...
}:
let
  cfg = config.programs.kiorg;
in
{
  # TODO: Allow for config, this requires kiorg to expose some CLI options
  # or env variable that can point at a config file.
  options.programs.kiorg = {
    enable = lib.mkEnableOption "Enable kiorg, the file manager for hackers.";
    package = lib.mkPackageOption pkgs "kiorg" {
      default = withSystem pkgs.stdenv.hostPlatform.system ({ config, ... }: config.packages.default);
    };
  };
  config = lib.mkIf cfg.enable {
    environment.systemPackages = [
      cfg.package
    ];
  };
}
