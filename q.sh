#!/bin/bash

n=$1
c=$2

for a in $(seq 0 "$n" "$c"); 
do
  #./run.sh 'phase' --t-max=2.55 --size=100 --t-step=0.001 --seeds $(seq -s ' ' (($a + ))) --eq-steps 1
done
