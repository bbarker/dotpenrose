{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
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
          
          penroseBuildInputs = with pkgs; [
              wayland
              xorg.libX11
              xorg.libXcursor
              xorg.libXrandr
              xorg.libXxf86vm
              xorg.libXi
              xorg.libXft
              libxkbcommon
              mesa
              libglvnd
              vulkan-loader
          ];

          runtimeDeps = with pkgs; [
            nitrogen
            picom
            haskellPackages.yeganesh
            xscreensaver
            dmenu-rs
            gnome-keyring
            xorg.xmodmap
          ];

          
          # Common arguments can be set here to avoid repeating them later
          commonArgs = {
            src = craneLib.cleanCargoSource (craneLib.path ./.);
            
            buildInputs =  penroseBuildInputs ++ runtimeDeps;

            nativeBuildInputs = with pkgs; [
              pkg-config
              makeWrapper
            ];

            # Explicitly declare runtime dependencies
            propagatedBuildInputs = runtimeDeps;
          };
          runtimeDepsPath = pkgs.lib.makeBinPath runtimeDeps;
          ldLibraryPath = pkgs.lib.makeLibraryPath penroseBuildInputs;

          # Build dependencies
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          # Build the actual package
          penrose = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;

            postInstall = ''
              # Wrap the binary with necessary runtime dependencies
              wrapProgram $out/bin/dotpenrose \
                --prefix PATH : ${runtimeDepsPath} \
                --set LD_LIBRARY_PATH "${ldLibraryPath}"

              # Setup fonts
              mkdir -p $out/share/fonts
              cp --update=none ${pkgs.nerd-fonts.hasklug}/share/fonts/opentype/NerdFonts/Hasklug/*Hasklug*.otf $out/share/fonts/
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
              nerd-fonts.hasklug
            ] ++ runtimeDeps;  # Add runtime deps to devShell

            
            LD_LIBRARY_PATH = ldLibraryPath;

            RUSTFLAGS = map (a: "-C link-arg=${a}") [
              "-Wl,--push-state,--no-as-needed"
              "-lEGL"
              "-lwayland-client"
              "-Wl,--pop-state"
            ];

            shellHook = ''
              export WHICH_PENROSE=DEVELOP
              export PATH="${pkgs.bashInteractive}/bin:${runtimeDepsPath}:$PATH"
              export PENROSE_DIR="$HOME/workspace/dotpenrose"
              mkdir -p $HOME/.local/share/fonts
              cp --update=none $(nix-build --no-out-link '<nixpkgs>' -A nerd-fonts.hasklug)/share/fonts/opentype/NerdFonts/Hasklug/*Hasklug*.otf ~/.local/share/fonts
              fc-cache
            '';
          };
        }
      );
}
