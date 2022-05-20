#!/bin/python3

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import sys
import pathlib
from scipy.optimize import curve_fit
import plot_constants

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

def slice_data(xs, ys):
  valid = []
  for y in ys:
    if y >= 0:
      valid.append(y)
    if y <= 0:
      break

  pos_len = len(valid)
  left_lim = len([v for v in valid if v > 0.96])
  d = 1

  return xs[left_lim:pos_len:d], ys[left_lim:pos_len:d]

def fit_plot(xs, ys, bounds):
  xn, yn = slice_data(xs, ys)
  
  popt, _ = curve_fit(mt_fit, xn, yn, bounds=bounds, maxfev=10000)  # ([1, 1, 0.1], [2, 2, 0.5])
  m0, tc, b = popt
  print(f'M_0={m0}, T_C={tc}, β={b}')
  return xn, [mt_fit(t, *popt) for t in xn], popt


def plot(ax1, xs, ys, bounds, label, name, colour):
  xaa, yaa = slice_data(xs, ys)
  ax1.scatter(xs, ys, marker='.', color=(*colour, 0.5), label=f'[{name}] {label}')

  fxa, fya, fpa = fit_plot(xaa, yaa, bounds)
  ax1.plot(fxa, fya, linestyle='dashed', 
    color=(*[max(c - 0.1, 0) for c in colour], 1), 
    label=f'[{name}] fit(M_0={round(fpa[0], 4)}, T_C={round(fpa[1], 4)}, β={round(fpa[2], 4)})'
    )

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
    plot(
      path=path,
      ax=ax,
      colour=colours[colour_keys[(i - 1) % len(colour_keys)]],
      name=i,
      label=path,
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
