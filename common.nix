
# common.nix
{ pkgs }:
with pkgs; {
  # Common packages needed for both development and runtime
  commonPackages = [
    wayland
    nitrogen
    picom
    haskellPackages.yeganesh
    xscreensaver
    dmenu-rs
    gnome.gnome-keyring

    # X11 and graphics libraries
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXxf86vm
    xorg.libXi
    xorg.libXft
    xorg.xmodmap
    libglvnd
  ];

  # Environment variables that might be needed in both contexts
  commonEnv = {
    RUSTFLAGS = builtins.map (a: "-C link-arg=${a}") [
      "-Wl,--push-state,--no-as-needed"
      "-lEGL"
      "-lwayland-client"
      "-Wl,--pop-state"
    ];

    LD_LIBRARY_PATH = lib.makeLibraryPath [
      libxkbcommon
      mesa.drivers
      vulkan-loader
      xorg.libX11
      xorg.libXcursor
      xorg.libXi
      xorg.libXrandr
    ];
  };

  # Common shell hook for font setup
  commonShellHook = ''
    mkdir -p $HOME/.local/share/fonts
    cp --update=none $(nix-build --no-out-link '<nixpkgs>' -A nerdfonts)/share/fonts/opentype/NerdFonts/*Hasklug*.otf ~/.local/share/fonts
    fc-cache
  '';
}

