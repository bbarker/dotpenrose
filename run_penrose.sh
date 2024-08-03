#!/usr/bin/env bash
export PENROSE_DIR=${PENROSE_DIR:-$(pwd)}

eval "$(/usr/bin/gnome-keyring-daemon --start --components=pkcs11,secrets,ssh)"
export SSH_AUTH_SOCK
eval "$(ssh-agent)"

cd "$PENROSE_DIR"
nix develop --command bash -c "
  while true; do
    # log out to a file
    \"$PENROSE_DIR\"/target/release/dotpenrose &> ~/.penrose.log
    # start a new log file if there's an error
    [[ $? > 0 ]] && mv ~/.penrose.log ~/prev-penrose.log
    export RESTARTED=true
  done
" &> ~/.penrose.log
