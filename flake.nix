{
    description = "A command-line tool for parsing and validating journal entries";

    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
        flake-utils.url = "github:numtide/flake-utils";
    };

    outputs = { self, nixpkgs, flake-utils }:
        flake-utils.lib.eachDefaultSystem (system: let
            pkgs = nixpkgs.legacyPackages.${system};
        in {
            defaultPackage = pkgs.rustPlatform.buildRustPackage {
                pname = "logbook-integrity";
                version = "0.1.0";
                src = ./.;
                cargoLock.lockFile = ./Cargo.lock;
            };
        });
}
