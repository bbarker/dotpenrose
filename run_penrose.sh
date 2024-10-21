#!/usr/bin/env bash
eval $(/usr/bin/gnome-keyring-daemon --start --components=pkcs11,secrets,ssh)
export SSH_AUTH_SOCK
eval $(ssh-agent)

while true; do
  "$PENROSE_DIR/target/release/dotpenrose" &> ~/.penrose.log
  # RUST_BACKTRACE=full "$PENROSE_DIR/target/debug/dotpenrose" &> ~/.penrose.log
  # start a new log file if there's an error
  [[ $? > 0 ]] && mv ~/.penrose.log ~/.penrose-prev.log
  export RESTARTED=true
done
