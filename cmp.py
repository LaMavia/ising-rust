#!/bin/python3

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import sys
import pathlib
from scipy.optimize import curve_fit

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
  v = np.max(1 - t/tc, 0)

  return m0 * np.power(v, b)

def slice_data(xs, ys):
  valid = []
  for y in ys:
    if y < 0:
      break
    valid.append(y)

  pos_len = len(valid)
  left_lim = len([v for v in valid if v > 0.9])
  d = 1

  return xs[left_lim:pos_len:d], ys[left_lim:pos_len:d]

def fit_plot(xs, ys, bounds):
  xn, yn = slice_data(xs, ys)
  
  popt, _ = curve_fit(mt_fit, xn, yn, bounds=bounds, maxfev=10000)  # ([1, 1, 0.1], [2, 2, 0.5])
  m0, tc, b = popt
  print(f'M_0={m0}, T_C={tc}, β={b}')
  return xn, [mt_fit(t, *popt) for t in xn], popt

def main(path_a: str, path_b: str):
  dfa = pd.read_csv(path_a)
  dfb = pd.read_csv(path_b)

  x_a = list(dfa[dfa.columns[0]])
  x_b = list(dfb[dfb.columns[0]])

  if x_a != x_b:
    print(f'''unmatching data arguments: (i, xa, xb) \n{
      [(i, xa, xb) for (i, (xa, xb)) 
        in enumerate(zip(x_a, x_b)) 
        if x_a != x_b
      ]
      }''')
    exit(1)

  y_a = list(dfa[dfa.columns[1]])
  y_b = list(dfb[dfb.columns[1]])

  fig, ax1 = plt.subplots()
  ax1.set_xlabel(dfa.columns[0])
  ax1.set_ylabel(dfa.columns[1])
  ax1.grid(which='both')

  colours = {
    'orange': (235/255, 116/255, 52/255),
    'cyan': (52/255, 217/255, 235/255),
    'black': (0, 0, 0)
  }

  xaa, yaa = slice_data(x_a, y_a)
  xbb, ybb = slice_data(x_b, y_b)

  ax1.plot(xaa, yaa, marker='.', color=(*colours['cyan'], 0.5), label=f'[a] {path_a}')
  ax1.plot(xbb, ybb, marker='.', color=(*colours['orange'], 0.5), label=f'[b]  {path_b}')
  # ax1.plot(x_a, calc_diff(y_a, y_b), label="difference (b - a)", color=(0.2, 0.6, 0.6, 0.6))
  
  # m0, tc, β
  fxa, fya, fpa = fit_plot(xaa, yaa, ([1, 2.3, 0.2], [5, 2.6, 0.3]))
  ax1.plot(fxa, fya, linestyle='dashed', color=(*colours['cyan'], 1), label=f'[a] fit(M_0={fpa[0]}, T_C={fpa[1]}, β={fpa[2]})')

  fxb, fyb, fpb = fit_plot(xbb, ybb, ([1, 1.85, 0.2], [1.5, 2, 0.5]))
  ax1.plot(fxb, fyb, linestyle='dashed', color=(*colours['orange'], 1), label=f'[b] fit(M_0={fpb[0]}, T_C={fpb[1]}, β={fpb[2]})')


  ax1.legend()

  # ax2 = ax1.twinx()
  # ax2.set_ylabel(f'{dfa.columns[1]}·{dfa.columns[0]}^-1')

  # ax2.plot(x_a[:-1], calc_rate_diff(x_a, y_a, y_b), label="rate difference (b - a)", color=(0.2, 0.6, 0.6, 0.2))
  # ax2.legend()

  fig.tight_layout()

  plt.show()

def print_usage():
  print(f'usage: ./cmp.py path_a path_b')

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