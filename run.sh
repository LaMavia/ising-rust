#!/bin/bash

# clean old data
rm "$1.csv" 2> /dev/null

# run
time cargo run --release "$1"

# plot
python ./plot.py "$1"