{ pkgs ? import <nixpkgs> { } }:
let
  ledger = pkgs.rustPlatform.buildRustPackage rec {
    pname = "ledger-server";
    version = "0.13.0";

    src = pkgs.fetchFromGitHub {
      owner = "holahmeds";
      repo = "ledger-server";
      rev = "v${version}";
      sha256 = "sha256-qgsD9MUGhsUUCSshnUk6l3Y1xyjhd1jFSuPHJAI45X0=";
    };

    cargoHash = "sha256-hfUzVOVf886X68xXZmvwgPRwHTyPJweRFwHpqL0GX0M=";
  };
in
pkgs.dockerTools.buildLayeredImage {
  name = ledger.pname;
  tag = ledger.version;

  config.Cmd = [ "${ledger}/bin/ledger-server" ];
}
