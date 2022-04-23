#!/bin/bash

if [[ $# -lt 1 ]]; then
  printf "usage: ./help.sh [simulation_type: 'hys' | 'phase']\n"
  exit 1
fi

cargo run --release --quiet -- "$1" --help