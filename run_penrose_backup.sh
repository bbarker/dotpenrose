#!/usr/bin/env bash
eval $(/usr/bin/gnome-keyring-daemon --start --components=pkcs11,secrets,ssh)
export SSH_AUTH_SOCK
eval $(ssh-agent)

rotate_log() {
    mv ~/.penrose.log ~/.penrose-prev.log
}

trap rotate_log SIGTERM

while true; do
    "$HOME/.nix-profile/bin/dotpenrose" &> ~/.penrose.log
    # "$PENROSE_DIR/target/release/dotpenrose" &> ~/.penrose.log
    # RUST_BACKTRACE=full "$PENROSE_DIR/target/debug/dotpenrose" &> ~/.penrose.log
    # Rotate log if there's an error or if the process was terminated
    [[ $? > 0 ]] && mv ~/.penrose.log ~/.penrose-prev.log
    export RESTARTED=true
    rotate_log
done
