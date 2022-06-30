#!/bin/python3

import numpy as np
import matplotlib.pyplot as plt
import sys
import json

def pos_of_index(w: int, i: int) -> [int, int]:
  return [x := i % w, (i - x) / w]

def main(path: str):
  f = open(path, 'r')
  desc = json.loads(f.read())
  f.close()

  w = desc['lattice']['width']
  h = desc['lattice']['height']
  E = [[pos_of_index(w, i) for i in ns] for ns in desc['lattice']['xs']]

  

  print(E)

  
  

if __name__ == '__main__':
  if len(sys.argv) < 2:
    print(f'usage: ./lattice/network.py path')
    exit(1)

  main(sys.argv[1])