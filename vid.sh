#!/bin/bash

paths=$(cat "$1" | tr ' ' '\n' | grep -i 'data/' | sed 's/\/desc.json//')

for path in $paths;
do
  ./frames_to_vid.sh "$path" "$2" 2> /dev/null
done