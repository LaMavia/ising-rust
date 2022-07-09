#!/bin/bash

for a in $( seq "$1" "$2" ); 
do
  ./run.sh 'hys' \
    --h-max=2.55 \
    --size=100 \
    --h-step=0.01 \
    --seeds "$a" \
    --temps 0.5 0.75 1.0

  sed 's/hys/ /' hys.args >> hys.agr.args
done
