{ pkgs ? import <nixpkgs> { } }:
let
  ledger = pkgs.rustPlatform.buildRustPackage rec {
    pname = "ledger-server";
    version = "0.12.0";

    src = pkgs.fetchFromGitHub {
      owner = "holahmeds";
      repo = "ledger-server";
      rev = "v${version}";
      sha256 = "sha256-Dy/jTWLwLUY3sXgLyKMQeAWaWVKsvBn2w/NHg4y0J9Y=";
    };

    cargoHash = "sha256-QBoElC9x4Nt0I0+KAjURc8nfU4ibAVm/bseKGiQIQA4=";
  };
in
pkgs.dockerTools.buildLayeredImage {
  name = ledger.pname;
  tag = ledger.version;

  config.Cmd = [ "${ledger}/bin/ledger-server" ];
}
