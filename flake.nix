{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
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
          isNixOS = pkgs.lib.hasPrefix "nixos" pkgs.stdenv.hostPlatform.system;
        in
        with pkgs;
        {
          formatter = nixpkgs.legacyPackages.x86_64-linux.nixpkgs-fmt;
          devShells.default = mkShell {
            buildInputs = [
              rust-bin.stable.latest.default
              wayland
              pkg-config
              # TODO: don't install all nerdfonts
              nerdfonts
              nitrogen
              picom
              haskellPackages.yeganesh
              xscreensaver
              dmenu-rs
              gnome.gnome-keyring # TODO: move to home-manager?

              rust-analyzer
            ] ++ (if isNixOS then [
              xorg.libX11
              xorg.libXcursor
              xorg.libXrandr
              xorg.libXxf86vm
              xorg.libXi
              xorg.xmodmap
              libglvnd
              xorg.libXft # for penrose_ui
            ] else []);

            nativeBuildInputs = [
              pkg-config
            ];

            RUSTFLAGS = map (a: "-C link-arg=${a}") [
              "-Wl,--push-state,--no-as-needed"
              "-lEGL"
              "-lwayland-client"
              "-Wl,--pop-state"
            ];

            LD_LIBRARY_PATH = (if isNixOS then 
              lib.makeLibraryPath [
                libxkbcommon
                mesa.drivers
                vulkan-loader
                xorg.libX11
                xorg.libXcursor
                xorg.libXi
                xorg.libXrandr
              ] else []);

            shellHook = ''
              # Ideally fonts are installed via the system, but here's
              # a hack to not need to do that ( https://nixos.wiki/wiki/Fonts ):
              mkdir -p $HOME/.local/share/fonts
              cp --update=none $(nix-build --no-out-link '<nixpkgs>' -A nerdfonts)/share/fonts/opentype/NerdFonts/*Hasklug*.otf ~/.local/share/fonts
              fc-cache
            '';
          };
        }
      );
}
