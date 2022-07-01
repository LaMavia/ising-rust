#!/bin/python3

import matplotlib.pyplot as plt
from matplotlib import cm
import numpy as np
import pandas as pd
import sys
import pathlib
import json
from scipy.optimize import curve_fit
import plot_constants
from fit_phase import curve_fit, fit_plot, mt_fit, slice_data
from itertools import groupby

# calcs b - a
def calc_diff(a, b):
  return [y_b - y_a for (y_a, y_b) in zip(a, b)]
  

def calc_rate_diff(xs, a, b):
  def calc_rate(xs, ys):
    return [(y_1 - y_0) / (x_1 - x_0)
            for ((x_0, x_1), (y_0, y_1)) 
            in zip(zip(xs[1:], xs[:-1]), zip(ys[1:], ys[:-1]))
           ]

  return calc_diff(calc_rate(xs, a), calc_rate(xs, b))

def mt_fit(t, m0, tc, b):
  v = (1 - t/tc)

  return m0 * np.sign(v) * np.abs(v) ** b

def ext_avg(xs):
  m, s, M = reduce(lambda u, x: (min(u[0], x), u[1] + x, max(u[2], x)), xs, (float('inf'), 0, float('-inf')))
  return m, s/len(xs), M

def plot(path, ax, fig, colour, name, label, bounds):
  df = pd.read_csv(path)

  ts, etas = [], []
  for t, e in [(t, min(*[p[1] for p in g])) for t, g in groupby(zip(list(df['T']), list(df["η"])), key=lambda p: p[0])]:
    if e > 0:
      ts.append(t)
      etas.append(e)

  ax.set_xlabel("T")
  ax.set_ylabel("η")

  ax.scatter(ts, etas, marker='.', color=(*colour, 0.3), label=f'[{name}] {label}')
  print(path)

  # plot a fitted line
  # ts_fit, etas_fit, fit_params = fit_plot(ts, etas, bounds, f=lambda x, b, c: np.arctan(x - b) + c)
# 
  # ax.plot(ts_fit, etas_fit, linestyle='dashed',
  #   color=(*[max(c - 0.2, 0) for c in colour], 1),
  #   label=f'[{name}] fit({fit_params})'
  # )





def main(paths: list[str]):
  colours = plot_constants.plot_colours
  colour_keys = list(colours.keys())

  fig = plt.figure(figsize=(10,7), dpi=300)
  ax = fig.add_subplot()
  
  ax.grid(which='both')

  for path, i in zip(paths, range(1, len(paths) + 1)):
    f = open(path, 'r')
    desc = json.loads(f.read())
    f.close()

    plot(
      path=desc['data_path'],
      ax=ax,
      fig=fig,
      colour=colours[colour_keys[(i - 1) % len(colour_keys)]],
      name=i,
      label=desc['data_path'],
      bounds=(0, float('inf'))
    )

  # ax.legend()
  fig.tight_layout()

  plt.savefig('plot_relax.png', dpi=300)

  #plt.show()

  #plt.show()

def print_usage():
  print('usage: ./cmp_relax.py {paths}')

def are_arguments_valid() -> [bool, str]:
  if len(sys.argv) <= 2: 
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

  main(paths=sys.argv[1:])
