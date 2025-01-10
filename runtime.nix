
# runtime.nix
{ pkgs }:
let
  common = import ./common.nix { inherit pkgs; };
in
with pkgs;
mkShell {
  buildInputs = common.commonPackages;
  
  shellHook = common.commonShellHook;
}
