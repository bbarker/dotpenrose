# TODO

1. yeganesh or dmenu app launcher?
2. some .config was copied manually: either write a build script to ln -s this or manage with home-manager.
3. port over xmonad
4. NixOS install section

# My Penrose Window Manager Setup
A customized tiling window manger configuration for X11 linux, using [Penrose](https://github.com/sminez/penrose).
Also utilizing dmenu app launcher, alacritty, and nitrogen background manager.

![screenshot](readme_img/screenshot.png)

⚠️ As I'm moving mostly over to nixos so this repo receives little attention nowadays. I mean look at all those manual installation steps. 

## Installation:

### NixOS or Nix

You'll likely want to disable login managers for simplicity. On NixOS,
this config should suffice for `configuration.nix`:

```nix
  # Enable touchpad support (enabled default in most desktopManager).
  services.libinput.enable = true;

  services.xserver = {
    enable = true;
    displayManager.startx.enable = true;
    xkb.layout = "us";
    xkb.variant = "";
  };
```


```
nix develop
cargo build --release
```

Then follow the configuration section below. 

To run, `startx` from a shell where you have run `nix develop`.

### Arch
Install dependencies (arch):

```shell
sudo pacman -Syu
sudo pacman -S cmake pkg-config fontconfig python3 cairo pango xorg-xinit xorg-server nitrogen firefox dmenu acpilight fzf rust-analyzer picom htop barrier neofetch openssh tree clang nvtop
```

Now install rust based software, (assuming those software specific dependencies are satisfied)
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup install nightly
cargo install alacritty lsd ripgrep nu starship rusty-rain gitui skim tokei bottom zellij bat rua ttyper taplo-cli lfs consoletimer onefetch oxker
cargo install cargo-udeps cargo-multi cargo-outdated cargo-semver-checks cargo-expand
```


## Configuration


Place config files to appropriate location
```shell
cp -r .config ~
``` 

Prepare wallpapers:
```shell
cp -r wallpapers ~/
```

Build and compile in this repos directory
```shell
cargo build --release
```

Now copy the compiled file to where it can be executed globally
```shell
sudo cp ./target/release/penrose-personal /usr/bin/
```

Now link the (possibly modified) '.xinitrc' to '~/.xinitrc'
```shell
cd $HOME
ln -s /path/to/dotpenrose/.xinitrc
```


Now the desktop environment is ready for usage with the 'startx' command from a raw command line

### Keymap
Here are the most important keybindings to control the window manager

'Meta' + ['h', 'j', 'k', 'l'] to navigate between windows

'Meta' + [ 1 .. 9 ] to switch between workspaces

'Meta' + 'Enter' to spawn alacritty terminal

'Meta' + 'q' to quit window

'Meta' + ',' to spawn dmenu prompt for application runner

'Meta' + 'Shift' + ['Up', 'Down', 'Left', 'Right'] to change tiling layout

For a complete binding list, check out 'src/main.rs'


# Tips

## Development

- Don't run `cargo clean`, ideally (you shouldn't need to anyway, most likely).
- Related to that, to be extra safe, if you are working on an experimental branch,
  you could check that out to a different directory, e.g.:

  ```
  $ git worktree add $HOME/workspace/dotpenrose_dev penrose_issue_302
  ```

  You could even start this on a different X server if you wanted to experiment
  at runtime.

  Or, you can swap out the current binary with the new dev binary by running
  `./use_dev_penrose.sh` (still need to `pkill dotpenrose` after).

