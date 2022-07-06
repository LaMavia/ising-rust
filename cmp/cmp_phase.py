#!/bin/python3

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import sys
import pathlib
import json
from scipy.optimize import curve_fit
from scipy.stats.mstats import gmean
import plot_constants
from fit_phase import curve_fit, fit_plot, mt_fit, slice_data
from functools import reduce
import re
from dataclasses import dataclass, field

@dataclass
class DataPoint:
  ts: list[float]
  ms: list[float]
  deg_avg: float
  deg_mse: float
  seed: int
  energy: list[float]
  time: list[float]
  n: list[float] 
  desc: dict

@dataclass
class Group:
  pattern: str
  label: str
  paths: list[str] = field(default_factory=list)
  data: list[DataPoint] = field(default_factory=list)
  descs: list[dict] = field(default_factory=list)

@dataclass
class ParamRegister:
  tc: list[float] = field(default_factory=list)
  beta: list[float] = field(default_factory=list)
  m0: list[float] = field(default_factory=list)

  def __str__(self):
    return f"""$\\langle T_C \\rangle={
    np.average(self.tc)
    }, \\langle\\beta\\rangle={
      np.average(self.beta)
    }, \\langle M_0 \\rangle={
      np.average(self.m0)
    }$"""

def mt_fit(t, m0, tc, b):
  v = (1 - t/tc)

  return m0 * np.sign(v) * np.abs(v) ** b

def plot(path, ax, colour, name, label, bounds):
  df = pd.read_csv(path)
  t_label, m_label = 'T', 'M'

  ts = list(df[t_label])
  ms = list(df[m_label])

  ax.set_xlabel(t_label)
  ax.set_ylabel(m_label)

  ts_reduced, ms_reduced = slice_data(ts, ms)

  # plot the data
  ax.scatter(ts, ms, marker='.', color=(*colour, 0.3), label=f'[{name}] {label}')

  # plot a fitted line
  ts_fit, ms_fit, fit_params = fit_plot(ts, ms, bounds)
  m0, tc, beta = fit_params

  ax.plot(ts_fit, ms_fit, linestyle='dashed',
    color=(*[max(c - 0.2, 0) for c in colour], 1),
    label=f'[{name}] fit(M_0={round(m0, 4)}, T_C={round(tc, 4)}, β={round(beta, 4)})'
  )

def ext_avg(xs, length=None):
  m, s, M = reduce(lambda u, x: (min(u[0], x), u[1] + x, max(u[2], x)), xs, (float('inf'), 0, float('-inf')))
  return m, s/(length or len(xs)), M

def plot_avg_err(data, ax, colour, name, label, bounds):
  t_label, m_label = 'T', 'M'

  ts, mss = [], []
  ms = []
  errs = [[], []]

  for dp in data:
    ts_new = dp.ts
    ts = ts_new if len(ts_new) > len(ts) else ts

    mss.append(dp.ms)

  for i in range(0, max(len(ms) for ms in mss)):
    mms = [ms[i] for ms in mss if i < len(ms)]

    inf, m_avg, sup = ext_avg(mms)

    ms.append(m_avg)
    errs[0].append(abs(m_avg - inf))
    errs[1].append(abs(m_avg - sup))

  ax.set_xlabel('$k_BT/J$')
  ax.set_ylabel('$M$')

  # plot the data
  ax.errorbar(ts, ms, marker='.', color=(*colour, 0.1), label=f'[{name}] {label}', yerr=errs)

  # plot a fitted line
  ts_fit, ms_fit, fit_params = fit_plot(ts, ms, bounds)
  m0, tc, beta = fit_params

  ax.plot(ts_fit, ms_fit, linestyle='dashed',
    color=(*[max(c - 0.2, 0) for c in colour], 1),
    label=f'[{name}] fit($M_0$={round(m0, 4)}, $T_C$={round(tc, 4)}, $\\beta$={round(beta, 4)})'
  )

def log_dist(xs, ys):
  buckets = dict()

  for (x, y) in zip(xs, ys):
    k = np.floor(np.log10(x))

    if k not in buckets:
      buckets[k] = list()
    
    buckets[k].append(y)

  return sorted(list(buckets.items()), key=lambda p: p[0])

def uzip(ps):
  xs = []
  ys = []

  for x, y in ps:
    xs.append(x)
    ys.append(y)

  return xs, ys
  
def bin_up(xs: list[float], bins=10, mn=None, mx=None):
  m, M = mn or min(xs), mx or max(xs)

  db = (M - m)/bins
  boundaries = [((db*i, db*(i+1)), []) for i in range(bins)]

  for x in xs:
    for ((l, h), ys) in boundaries:
      if l <= x < h:
        ys.append(x)

  return boundaries

def plot_dist(group, ax, fig, colour, name, label, bounds, reg: dict[str, ParamRegister], data: list[DataPoint]):
  ax_main, ax_dist, ax_energy, ax_relax = fig.get_axes()
  t_label, m_label = 'T', 'M'

  ax_main.set_xlabel('$k_BT/J$')
  ax_main.set_ylabel('$M$')

  for i, dp in enumerate(data):
    print(f'{chr(27)}[2J{i}/{len(data)}')

    # plot the data
    label = f'[{name}] seed={dp.seed} deg_avg={dp.deg_avg}'
    # ax_main.scatter(dp.ts, dp.ms, marker='.', color=(*colour, 0.1), label=label)

    try:
      # plot a fitted line
      ts_fit, ms_fit, fit_params = fit_plot(dp.ts, dp.ms, bounds)
      m0, tc, beta = fit_params

      reg[group.label].tc.append(tc)
      reg[group.label].beta.append(beta)
      reg[group.label].m0.append(m0)

      # ax_main.plot(ts_fit, ms_fit, linestyle='dashed',
      #   color=(*[max(c - 0.2, 0) for c in colour], 0.8),
      #   label=f'[{name}] fit($M_0$={round(m0, 4)}, $T_C$={round(tc, 4)}, $\\beta$={round(beta, 4)})'
      # )

      # Plot energy of time
      ax_energy.set_xlabel(r'$t$ [MC sweep]')
      ax_energy.set_ylabel(r'$\mathcal{H}$')
      ax_energy.scatter(
        dp.time,
        dp.energy,
        color=(*colour, 0.5),
        marker='.'
      )
    except:
      print(f'Failed to fit a curve; seed={dp.seed}')

  # Plot param dist
  ax_dist.set_xlabel(r'seed')
  ax_dist.set_ylabel(r'$T_C$')
  ax_dist.boxplot(
    [reg[group.label].tc],
    positions=[name],
    labels=[group.label]
  )

def main(paths: list[str]):
  plt.rcParams.update({'lines.markeredgewidth': 1})
  plt.rcParams['text.usetex'] = True
  plt.rcParams['axes.labelsize'] = 16

  colours = plot_constants.plot_colours
  colour_keys = list(colours.keys())

  fig, ((ax_main, ax_dist), (ax_energy, ax_relax)) = plt.subplots(2, 2, figsize=(16,14), dpi=300)
  ax_main.grid(which='both')
  ax_dist.grid(which='both')
  ax_energy.grid(which='both')
  ax_relax.grid(which='both')

  groups = [Group(pattern='/regular/', label='regular'), Group(pattern='/irregular/', label='irregular')]

  param_reg: dict[str, ParamRegister] = {
    'regular': ParamRegister(),
    'irregular': ParamRegister()
  }


  # make groups
  for path in paths:
    for group in groups:
      if re.search(group.pattern, path) != None:
        f = open(path, 'r')
        desc = json.loads(f.read())
        f.close()

        group.paths.append(desc['data_path'])
        group.descs.append(desc)
        break

  # make data points
  for group in groups:
    for i, path in enumerate(group.paths):
      df = pd.read_csv(path)
      desc = group.descs[i]

      group.data.append(DataPoint(
        ts=df['T'], 
        ms=df['M'], 
        deg_avg=desc['deg_avg'], 
        deg_mse=desc['deg_mse'], 
        seed=desc['seed'], 
        energy=df['E'], 
        time=df['t'], 
        n=df['n'],
        desc=desc))

  # plot data
  for group, i in zip([g for g in groups if len(g.paths) > 0], range(1, len(paths) + 1)):
    colour = colours[colour_keys[(i - 1) % len(colour_keys)]]
    bounds = (1e-12, 5)

    plot_dist(
      group=group,
      ax=ax_main,
      fig=fig,
      colour=colour,
      name=i,
      label=group.label,
      bounds=bounds,
      reg=param_reg,
      data=group.data
    )

    plot_avg_err(
      data=group.data, 
      ax=ax_main, 
      colour=colour, 
      name=i, 
      label=group.label, 
      bounds=bounds
    )

    ns = []
    for dp in group.data:
      ns.extend(np.log10(dp.n))

    xs, ys = uzip(bin_up(ns, mn=0, mx=4))

    ax_relax.set_xlabel(r'log(eq_steps)')
    ax_relax.set_ylabel('liczba punktów')
    ax_relax.scatter(
      [x[0] for x in xs],
      [len(y) for y in ys],
      color=(*colour, 0.8)
    )

  fig.suptitle(f"""regular: {param_reg['regular']}\n irregular: {param_reg['irregular']}""", fontsize=20)
  ax_main.legend()
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
