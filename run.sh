#!/bin/bash

# clean old data
# rm "$2.csv" 2> /dev/null

# run
plot_args=$(cargo run --release -- "$@")


# plot
if [[ -n "$plot_args" ]]; then 
  printf "args: %s\n" "$plot_args"
  ./cmp/main.py $plot_args; 
fi
