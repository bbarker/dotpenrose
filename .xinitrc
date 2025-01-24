#!/bin/sh
#
# ~/.xinitrc
#
# Executed by startx (run your window manager from here)

export PENROSE_DIR=${PENROSE_DIR:-"$HOME/workspace/dotpenrose"}
LOG_FILE="$HOME/.xinitrc.log"
log_msg() {
  echo "$1" >> "$LOG_FILE"
}

rm "$LOG_FILE"

log_msg "starting xinitrc"

if [ -d /etc/X11/xinit/xinitrc.d ]; then
  for f in /etc/X11/xinit/xinitrc.d/*; do
    [ -x "$f" ] && . "$f"
  done
  unset f
fi

log_msg "begin xrandr commands"
xrandr --output DP-2 --rotate left && 
xrandr --output DP-0 --rotate left --right-of DP-2 &&
xrandr --output DP-4 --rotate left --left-of DP-2 &&
log_msg "end xrandr commands"

# polybar & log_msg "started polybar" && # switching to start from penrose
picom & log_msg "started picom" &&
nitrogen --set-zoom-fill --random "$PENROSE_DIR/wallpapers" & log_msg "started nitrogen" &&
exec "$PENROSE_DIR/run_penrose.sh"
# exec "$PENROSE_DIR/target/release/dotpenrose" >> "$LOG_FILE" 2>&1 && log_msg "finished running penrose!"
# exec xterm 
