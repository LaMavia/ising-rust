#!/bin/python3
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import sys
import pathlib
import json
from scipy.optimize import curve_fit
from fit_phase import fit_plot
from plot_constants import *

def map_prop(path: str, prop: str) -> [float, [float, float, float]]: # (desc[prop], (m0, tc, β))
  f = open(path, 'r')
  desc = json.loads(f.read())
  f.close()

  df = pd.read_csv(desc['data_path'])
  t_label, m_label, *_ = df.columns


  ts = list(df[t_label])
  ms = list(df[m_label])

  _, _, p = fit_plot(ts, ms, (0.01, 5))
  return desc[prop], p

def main(paths):
  prop = 'deg_avg'
  xs = []
  ys = []

  for deg_p, fit_params in [map_prop(prop=prop, path=path) for path in paths]:
    xs.append(deg_p)
    ys.append(fit_params[2])

  fig, ax = plt.subplots(figsize=(7, 10))
  print(xs, ys)
  ax.scatter(xs, ys)

  fig.tight_layout()
  plt.show()

if __name__ == '__main__':
  main(sys.argv[1:])