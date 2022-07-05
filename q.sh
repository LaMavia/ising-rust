#!/bin/bash

for a in $( seq "$2" "$3" ); 
do
  ./run.sh 'phase' \
    --t-max=2.55 \
    --size=100 \
    --t-step=0.001 \
    --seeds $(seq -s ' ' $(( "$1" * "$a" + 1 )) $(( "$1" * ("$a" + 1) ))) \
    --eq-steps 1

  sed 's/phase/ /' phase.args >> phase.agr.args
done
