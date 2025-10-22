{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

{
  env.GREET = "lang";
  packages = [
    pkgs.git
    pkgs.lld
    pkgs.rust-analyzer
    pkgs.devenv
    pkgs.nodejs
  ];

  languages.rust = {
    enable = true;
    channel = "nightly";
  };

  enterShell = ''
    rustupdate
    git --version
  '';
  
}
