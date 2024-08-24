#!/bin/bash

cargo run --release -- \
 --input-file $1 \
 --make-old-frames \
 --make-sizes \
 --frame-delta-threshold 0.04 \
 --start-color "#00457b" \
 --end-color "#00457b" \
 --hue-threshold 0.05 \
 --sl-threshold 0.5 \
 --rgb-threshold 1 \
 --size-overestimate 400 \
 --make-new-frames \
 --make-video \
 --threads 12
