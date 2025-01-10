
# dev.nix
{ pkgs }:
let
  common = import ./common.nix { inherit pkgs; };
in
with pkgs;
mkShell {
  buildInputs = common.commonPackages ++ [
    # Development-specific packages
    rust-bin.stable.latest.default
    rust-analyzer
    pkg-config
    nerdfonts
  ];

  nativeBuildInputs = [
    pkg-config
  ];

  inherit (common.commonEnv) RUSTFLAGS LD_LIBRARY_PATH;
  
  shellHook = common.commonShellHook;
}
