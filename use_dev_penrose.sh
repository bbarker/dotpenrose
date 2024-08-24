#!/usr/bin/env bash

PENROSE_RELEASE_DIR="$PENROSE_DIR"/target/release/

mv "$PENROSE_RELEASE_DIR"/dotpenrose "$PENROSE_RELEASE_DIR"/dotpenrose_old \
  && cp target/release/dotpenrose "$PENROSE_RELEASE_DIR"
