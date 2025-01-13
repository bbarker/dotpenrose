# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }: 
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };

          craneLib = (crane.mkLib pkgs).overrideToolchain pkgs.rust-bin.stable.latest.default;
          
          # Common arguments can be set here to avoid repeating them later
          commonArgs = {
            src = craneLib.cleanCargoSource (craneLib.path ./.);
            
            buildInputs = with pkgs; [
              wayland
              xorg.libX11
              xorg.libXcursor
              xorg.libXrandr
              xorg.libXxf86vm
              xorg.libXi
              xorg.libXft
              libglvnd
            ];

            nativeBuildInputs = with pkgs; [
              pkg-config
              makeWrapper
            ];
          };

          # Build dependencies
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          # Build the actual package
          penrose = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;

            postInstall = ''
              # Wrap the binary with necessary runtime dependencies
              wrapProgram $out/bin/dotpenrose \
                --prefix PATH : ${pkgs.lib.makeBinPath (with pkgs; [
                  nitrogen
                  picom
                  haskellPackages.yeganesh
                  xscreensaver
                  dmenu-rs
                  gnome-keyring
                ])} \
                --set LD_LIBRARY_PATH "${pkgs.lib.makeLibraryPath (with pkgs; [
                  libxkbcommon
                  mesa.drivers
                  vulkan-loader
                  xorg.libX11
                  xorg.libXcursor
                  xorg.libXi
                  xorg.libXrandr
                ])}"

              # Setup fonts
              mkdir -p $out/share/fonts
              cp --update=none ${pkgs.nerdfonts}/share/fonts/opentype/NerdFonts/*Hasklug*.otf $out/share/fonts/
            '';
          });

        in
        {
          packages.default = penrose;
          
          formatter = nixpkgs.legacyPackages.x86_64-linux.nixpkgs-fmt;
          
          # Development shell
          devShells.default = pkgs.mkShell {
            inputsFrom = [ penrose ];

            buildInputs = with pkgs; [
              rust-bin.stable.latest.default
              rust-analyzer
              nerdfonts
            ];

            RUSTFLAGS = map (a: "-C link-arg=${a}") [
              "-Wl,--push-state,--no-as-needed"
              "-lEGL"
              "-lwayland-client"
              "-Wl,--pop-state"
            ];

            shellHook = ''
              mkdir -p $HOME/.local/share/fonts
              cp --update=none $(nix-build --no-out-link '<nixpkgs>' -A nerdfonts)/share/fonts/opentype/NerdFonts/*Hasklug*.otf ~/.local/share/fonts
              fc-cache
            '';
          };
        }
      );
}
