#!/bin/bash

# clean old data
# rm "$2.csv" 2> /dev/null

# run
plot_args=$(cargo run --release -- "$@")


# plot
if [[ -n "$plot_args" ]]; then 
  printf "args: %s\n" "$plot_args"
  name=$(echo "$plot_args" | head -n1 | cut -d ' ' -f1)

  echo "$plot_args" > "$name.args"

  ./cmp/main.py $plot_args; 
fi
