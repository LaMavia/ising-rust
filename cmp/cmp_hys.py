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

def main(paths):
  colours = {
    'orange': (235/255, 116/255, 52/255),
    'cyan': (52/255, 217/255, 235/255),
    'red': (196/255, 55/255, 53/255),
    'blue': (49/255, 145/255, 204/255),
    'yellow': (230/255, 182/255, 53/255),
    'tl': (53/255, 230/255, 109/255),
    'purple': (191/255, 53/255, 230/255),
    'black': (0.2, 0.2, 0.2)
  }

  fig, ax = plt.subplots(figsize=(10,7), dpi=300)

  ax.grid(which='both')

  colour_keys = list(colours.keys())

  for path, i in zip(paths, range(1, len(paths) + 1)):
    plot(path=path, ax=ax, colour=colours[colour_keys[(i - 1) % len(colour_keys)]], name=i, label=path)

  ax.legend(loc='lower center', bbox_to_anchor=(0.5, -len(paths)/15))
  fig.tight_layout()

  plt.savefig('plot_hys.png', dpi=300)


def print_usage():
  print(f'usage: ./cmp_hys.py path_a path_b')

def are_arguments_valid() -> [bool, str]:
  if len(sys.argv) <= 1: 
    return [False, 'Too few arguments; expected at least 1']
  
  paths = sys.argv[1:]

  for path_str in paths:
    path = pathlib.Path(path_str)

    if not path.exists():
      return [False, f'''Path {path_str} doesn't exist''']
  return [True, '']

if __name__ == '__main__':
  valid, error_msg = are_arguments_valid()

  if not valid:
    print_usage()
    exit(1)

  paths = sys.argv[1:]
  main(paths)