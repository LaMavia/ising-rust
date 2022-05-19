#!/bin/bash

# clean old data
# rm "$2.csv" 2> /dev/null

# run
plot_args=$(cargo run --release -- "$@")

printf "args: %s\n" "$plot_args"

# plot
if [[ -n "$plot_args" ]]; then ./cmp.py $plot_args; fi
