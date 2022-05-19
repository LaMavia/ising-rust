#!/bin/python3

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import sys
import pathlib


def plot(path, ax, colour, name, label):
  df = pd.read_csv(path)
  hs = list(df[df.columns[0]])
  ms = list(df[df.columns[1]])

  ax.set_xlabel(df.columns[0])
  ax.set_ylabel(df.columns[1])
  ax.plot(hs, ms, color=colour, marker='.', label=f'''[{name}] {label}''')

def main(path_a, path_b):
  colours = {
    'orange': (235/255, 116/255, 52/255),
    'cyan': (52/255, 217/255, 235/255),
    'black': (0.2, 0.2, 0.2)
  }

  fig, ax = plt.subplots(figsize=(10,7), dpi=300)

  ax.grid(which='both')

  plot(path=path_a, ax=ax, colour=colours['cyan'], name='a', label=path_a)
  plot(path=path_b, ax=ax, colour=colours['orange'], name='b', label=path_b)

  ax.legend(loc='lower center', bbox_to_anchor=(0.5, -0.2))
  fig.tight_layout()

  plt.savefig('plot_hys.png', dpi=300)


def print_usage():
  print(f'usage: ./cmp_hys.py path_a path_b')

def are_arguments_valid() -> [bool, str]:
  if len(sys.argv) <= 2: 
    return [False, 'Too few arguments; expected 2']
  
  path_a, path_b = sys.argv[1:]

  for path_str in [path_a, path_b]:
    path = pathlib.Path(path_str)

    if not path.exists():
      return [False, f'''Path {path_str} doesn't exist''']
  return [True, '']

if __name__ == '__main__':
  valid, error_msg = are_arguments_valid()

  if not valid:
    print_usage()
    exit(1)

  path_a, path_b = sys.argv[1:]
  main(path_a=path_a, path_b=path_b)