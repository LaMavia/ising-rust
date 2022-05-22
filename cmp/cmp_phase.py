#!/bin/python3

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import sys
import pathlib
import json
from scipy.optimize import curve_fit
import plot_constants
from fit_phase import curve_fit, fit_plot, mt_fit, slice_data

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

def plot(path, ax, colour, name, label, bounds):
  df = pd.read_csv(path)
  t_label, m_label, *_ = df.columns

  ts = list(df[t_label])
  ms = list(df[m_label])

  ax.set_xlabel(t_label)
  ax.set_ylabel(m_label)

  ts_reduced, ms_reduced = slice_data(ts, ms)

  # plot the data
  ax.scatter(ts, ms, marker='.', color=(*colour, 0.5), label=f'[{name}] {label}')

  # plot a fitted line
  ts_fit, ms_fit, fit_params = fit_plot(ts, ms, bounds)
  m0, tc, beta = fit_params

  ax.plot(ts_fit, ms_fit, linestyle='dashed',
    color=(*[max(c - 0.2, 0) for c in colour], 1),
    label=f'[{name}] fit(M_0={round(m0, 4)}, T_C={round(tc, 4)}, β={round(beta, 4)})'
  )





def main(paths: list[str]):
  colours = plot_constants.plot_colours
  colour_keys = list(colours.keys())

  fig, ax = plt.subplots(figsize=(10,7), dpi=300)
  ax.grid(which='both')

  for path, i in zip(paths, range(1, len(paths) + 1)):
    f = open(path, 'r')
    desc = json.loads(f.read())
    f.close()

    plot(
      path=desc['data_path'],
      ax=ax,
      colour=colours[colour_keys[(i - 1) % len(colour_keys)]],
      name=i,
      label=desc['data_path'],
      bounds=(0.1, 5)
    )

  ax.legend()
  fig.tight_layout()
  plt.savefig('plot.png', dpi=300)

  #plt.show()

def print_usage():
  print(f'usage: ./cmp_phase.py path_a path_b')

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
