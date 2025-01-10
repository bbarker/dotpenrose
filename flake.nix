# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
        in
        {
          formatter = nixpkgs.legacyPackages.x86_64-linux.nixpkgs-fmt;
          devShells = {
            default = import ./dev.nix { inherit pkgs; };
            runtime = import ./runtime.nix { inherit pkgs; };
          };
        }
      );
}
