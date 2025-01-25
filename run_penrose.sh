#!/usr/bin/env bash
# TODO: only start this if on path
eval $(/usr/bin/gnome-keyring-daemon --start --components=pkcs11,secrets,ssh)
export SSH_AUTH_SOCK
eval $(ssh-agent)

rotate_log() {
    mv ~/.penrose.log ~/.penrose-prev.log
}

trap rotate_log SIGTERM

WHICH_PENROSE=${WHICH_PENROSE:-ON_PATH}

while true; do
    if [ "$WHICH_PENROSE" = "ON_PATH" ]; then
        dotpenrose &> ~/.penrose.log
    else
        "$PENROSE_DIR/target/release/dotpenrose" &>> ~/.penrose.log
    fi
    # RUST_BACKTRACE=full "$PENROSE_DIR/target/debug/dotpenrose" &> ~/.penrose.log
    # Rotate log if there's an error or if the process was terminated
    # (I think we don't want this with `trap` call above, but need to confirm):
    # [[ $? > 0 ]] && mv ~/.penrose.log ~/.penrose-prev.log
    export RESTARTED=true
    rotate_log
done
