#!/bin/bash

# /home/mavi/proyectos/rust/ising/data/irregular/phase/size=100_step=0.01_max=2.55_seed=222222/frames
base_dist="$1"
output_dist="$base_dist/anim.mp4"
framerate=${2:-25}

if cat $(find "$base_dist/frames" -name '*.png' | sort -V) \
  | ffmpeg \
    -framerate "$framerate" \
    -y -i - \
    -c:v libx264 \
    -profile:v high \
    -crf 20 \
    -pix_fmt yuv420p \
    "$output_dist" 1>&2 
then
    rm -r "$base_dist/frames"
fi


printf "%s\n" "$(realpath "$output_dist")"