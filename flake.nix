{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.11.0";
    cargo2nix.inputs.rust-overlay.follows = "rust-overlay";
    flake-utils.follows = "cargo2nix/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.follows = "rust-overlay/nixpkgs";
  };

  outputs = inputs:
    with inputs;
      flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [cargo2nix.overlays.default];
          };

          rustPkgs = pkgs.rustBuilder.makePackageSet {
            rustVersion = "1.83.0";
            packageFun = import ./Cargo.nix;
          };
        in rec {
          packages = {
            sudoku = rustPkgs.workspace.sudoku {};
            default = packages.sudoku;
            cargoTests = pkgs.rustBuilder.runTests rustPkgs.workspace.sudoku {};
          };
          devShells = {
              default = pkgs.mkShell {
      packages = [
        pkgs.cargo-insta
      ];
          };
          };
        }
      );
}
