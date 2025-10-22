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
    pkgs.mold
    pkgs.rust-analyzer
    pkgs.devenv
    pkgs.nodejs
  ];

  languages.rust = {
    enable = true;
    channel = "nightly";
    targets = ["wasm32-unknown-unknown" "x86_64-unknown-linux-gnu"];
  };

  enterShell = ''
    rustupdate
    git --version
  '';
  
}
