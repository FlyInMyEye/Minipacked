{
  description = "Simple tool to pack files and directories into portable (or even encrypted) containers written in rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        minipacked = pkgs.callPackage ./packaging/nix/package.nix { };
      in {
        packages.default = minipacked;
        packages.minipacked = minipacked;

        apps.default = {
          type = "app";
          program = "${minipacked}/bin/minipacked";
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [ rustc cargo ];
        };
      });
}
