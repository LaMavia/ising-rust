#!/bin/bash

for a in $( seq "$1" "$2" ); 
do
  ./run.sh 'hys' \
    --h-max=2 \
    --size=100 \
    --h-step=0.01 \
    --seeds "$a" \
    --temps 0.5 1.0 1.5 2

  sed 's/hys/ /' hys.args >> hys.agr.args
done
