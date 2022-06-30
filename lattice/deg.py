#!/bin/python3

import numpy as np
import matplotlib.pyplot as plt
import sys
import json


def main(path: str):
  f = open(path, 'r')
  desc = json.loads(f.read())
  f.close()

  w = desc['lattice']['width']
  h = desc['lattice']['height']
  E = desc['lattice']['xs']

  data = np.array([len(v) for v in E]).reshape((w, h))

  plt.imshow(data, interpolation='nearest')
  plt.show()

if __name__ == '__main__':
  if len(sys.argv) < 2:
    print(f'usage: ./lattice/deg.py path')
    exit(1)

  main(sys.argv[1])