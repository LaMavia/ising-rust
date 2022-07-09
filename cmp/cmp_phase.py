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
from typing import TypeVar, Generic, Union, Any

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

  def __getitem__(self, item):
    return getattr(self, item)

def mt_fit(t, m0, tc, b):
  v = (1 - t/tc)

  return m0 * np.sign(v) * np.abs(v) ** b

U = TypeVar('U')
V = TypeVar('V')
def uzip(ps: list[tuple[U, V]]) -> tuple[list[U], list[V]]:
  xs = []
  ys = []

  for x, y in ps:
    xs.append(x)
    ys.append(y)

  return xs, ys
  
def bin_up(xs: list[Union[float, tuple[float, float]]], bins=10, mn=None, mx=None) -> list[tuple[tuple[float, float], list[float]]]:
  xs = [(p, p) if isinstance(p, float) else p for p in xs]
  m, M = mn or min([x for x, _ in xs]), mx or max([x for x, _ in xs])

  db = (M - m)/bins
  boundaries = [((db*i, db*(i+1)), []) for i in range(bins)]

  for x,y in xs:
    for ((l, h), ys) in boundaries:
      if l <= x < h:
        ys.append(y)
        break

  return boundaries

def ext_avg(xs, length=None):
  m, s, M = reduce(
    lambda u, x: (min(u[0], x), u[1] + x, max(u[2], x)), 
    xs, 
    (float('inf'), 0, float('-inf'))
  )

  return m, s/(length or len(xs)), M

def fit_in_between(xs: list[float], min_curve: list[float], max_curve: list[float], n_range=100) -> tuple[list[float], list[float]]:
  def get_ns(i: int, ys: list[float]) -> list[tuple[float, float]]:
    l = max(0, i - n_range)
    r = min(len(ys), i + n_range)

    return list(zip(xs[l:], ys[l:r])) 

  def dist_sq(p, q):
    return (p[0] - q[0])**2 + (p[1] - q[1])**2

  min_points = list(zip(xs, min_curve))
  max_points = list(zip(xs, max_curve))
  output = []

  for i, p in enumerate(min_points):
    x, y = p

    ns = get_ns(i, max_curve)
    ds_ns = [(dist_sq(p, q), q) for q in ns]
    x_, y_ = min(ds_ns, key=lambda t: t[0])[1]

    mid_x = (x + x_)/2
    mid_y = (y + y_)/2

    output.append((mid_x, mid_y))

  return uzip(output)

def process_register(group: Group, reg: dict[str, ParamRegister], bounds: tuple[float, float]):
  for i, dp in enumerate(group.data):
    print(f"[process::{group.label}] fitting params {i+1}/{len(group.data)}")
    _, _, (m0, tc, beta) = fit_plot(dp.ts, dp.ms, bounds)

    reg[group.label].tc.append(tc)
    reg[group.label].beta.append(beta)
    reg[group.label].m0.append(m0)

"""
Plot the main phase transition diagram
"""
def plot_main(group: Group, ax: plt.Axes, colour: list[float], name: str, bounds: tuple[float]):
  data = group.data
  ts, mss = [], []
  min_curve, max_curve = [], []

  # group data
  for dp in data:
    ts = dp.ts if len(dp.ts) > len(ts) else ts
    mss.append(dp.ms)

  # process data
  for i in range(0, max(len(ms) for ms in mss)):
    mms = [ms[i] for ms in mss if i < len(ms)]

    inf, m_avg, sup = ext_avg(mms)

    max_curve.append(sup)
    if len(mms) == len(data):
      min_curve.append(inf)

  # set labels
  ax.set_xlabel('$k_BT/J$')
  ax.set_ylabel('$M$')

  # fin the average curve
  ts_between, ms_between = fit_in_between(ts, min_curve, max_curve)

  # plot the extrema curves
  ax.scatter(ts[:len(min_curve)], min_curve, color=(*colour, 0.05), marker='.')
  ax.scatter(ts, max_curve, color=(*colour, 0.05), marker='.')

  # plot the average curve
  ax.plot(ts_between, ms_between, color=(*colour, 0.9))

  # plot the fitted line
  ts_fit, ms_fit, fit_params = fit_plot(ts_between, ms_between, bounds)
  m0, tc, beta = fit_params

  ax.plot(ts_fit, ms_fit, linestyle='dashed',
    color=(*[max(c - 0.2, 0) for c in colour], 1),
    label=f'[{group.label}] fit($M_0$={round(m0, 4)}, $T_C$={round(tc, 4)}, $\\beta$={round(beta, 4)})'
  )

  ax.plot(
    [tc, tc], 
    [1, 0], 
    color=(*[max(c - 0.4, 0) for c in colour], 1), 
    linestyle='--',
  )

"""
Plot energy of time
"""
def plot_energy(ax: plt.Axes, group: Group, colour: list[float]):
  def flatten(xss):
    return [x for xs in xss for x in xs]

  data: list[DataPoint] = group.data

  ax.set_xlabel(r'$t$ [MC sweep]')
  ax.set_ylabel(r'$\mathcal{H}$')

  bin_density = 0.02
  min_curve, max_curve = [], []

  raw_bins = bin_up(
    ts_raw := flatten([zip(dp.time, dp.energy) for dp in data]), 
    bins=round(bin_density*len(ts_raw))
  )
  bins, ees = uzip(
    [
      b for b in raw_bins if len(b[1]) > 0
    ]
  )

  ts = [(l + r)/2 for l,r in bins]

  for es in ees:
    min_curve.append(min(es))
    max_curve.append(max(es))
  
  ts_between, es_between = fit_in_between(ts, min_curve, max_curve)
  
  ax.scatter(
    ts,
    min_curve,
    color=(*colour, 0.02),
    marker='.'
  )
  ax.scatter(
    ts,
    max_curve,
    color=(*colour, 0.02),
    marker='.'
  )
  ax.scatter(
    ts_between,
    es_between,
    color=(*colour, 0.5),
    marker='o',
    label=group.label
  )

def tagged_box(ax: plt.Axes, xs: list[float], positions: list[Any], labels: list[str]):
  params = ax.boxplot(
    xs,
    positions=positions,
    labels=labels
  )

  for line in params['medians']:
    (x_l, y),(x_r, _) = line.get_xydata()
    if not np.isnan(y): 
      x_line_center = x_l + (x_r - x_l)/2
      y_line_center = y  
      ax.text(x_line_center, y_line_center,
              '%.3f' % y, 
              verticalalignment='center', 
              fontsize=16, backgroundcolor="white")

"""
Plot param distribution
"""
def plot_dist(group: Group, ax: plt.Axes, name: str, reg: dict[str, ParamRegister], field: str, ylabel: str):
  ax.set_xlabel(r'rodzaj sieci')
  ax.set_ylabel(ylabel)
  tagged_box(
    ax=ax, 
    xs=[reg[group.label][field]], 
    positions=[name], 
    labels=[group.label])

def plot_deg_dist(group: Group, ax: plt.Axes, name: str):
  ax.set_xlabel(r'rodzaj sieci')
  ax.set_ylabel(r'$\langle k \rangle$')
  tagged_box(
    ax=ax,
    xs=[dp.deg_avg for dp in group.data],
    positions=[name],
    labels=[group.label]
  )

"""
Plot relaxation distribution
"""
def plot_relax(group: Group, ax: plt.Axes, colour: list[float]):
  ns = []
  for dp in group.data:
    ns.extend(np.log10(dp.n))

  xs, ys = uzip(bin_up(ns, mn=0, mx=4, bins=20))

  ax.set_xlabel(r'log(eq_steps)')
  ax.set_ylabel('liczba punktów')
  ax.scatter(
    [x[0] for x in xs],
    [len(y) for y in ys],
    color=(*colour, 0.8),
    label=group.label
  )

def main(paths: list[str]):
  plt.rcParams.update({'lines.markeredgewidth': 1})
  plt.rcParams['text.usetex'] = True
  plt.rcParams['axes.labelsize'] = 16

  colours = plot_constants.plot_colours
  colour_keys = list(colours.keys())

  bounds = (1e-12, 5)

  figs = [plt.figure(figsize=(13,8), dpi=300) for _ in range(7)]
  fig_main, fig_dist, fig_energy, fig_relax, fig_dist_beta, fig_dist_m0, fig_dist_deg = figs
  axs = [fig.subplots() for fig in figs]

  ax_main, ax_dist, ax_energy, ax_relax, ax_dist_beta, ax_dist_m0, ax_dist_deg = axs

  for ax in axs:
    ax.grid(which='both')
    ax.tick_params(which='both', labelsize=14)

  groups = [Group(pattern='/regular/', label='regular'), Group(pattern='/irregular/', label='irregular')]

  param_reg: dict[str, ParamRegister] = {
    'regular': ParamRegister(),
    'irregular': ParamRegister()
  }

  # make groups
  print(f"[process] processing desc.json paths ({len(paths)} paths)")
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
  print(f"[process] processing groups ({len(groups)} groups)")
  for group in groups:
    for i, path in enumerate(group.paths):
      print(f'[process::{group.label}] processing path {i+1}/{len(group.paths)}')
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

    process_register(group=group, reg=param_reg, bounds=bounds)
    
  # plot data
  for group, i in zip([g for g in groups if len(g.paths) > 0], range(1, len(paths) + 1)):
    colour = colours[colour_keys[(i - 1) % len(colour_keys)]]

    print(f'[plot] plotting {group.label} ({len(group.data)} data points)')

    for f, l, ax in [('tc', r'$T_C$', ax_dist), ('beta', r'$\beta$', ax_dist_beta), ('m0', r'$M_0$', ax_dist_m0)]:
      plot_dist(
        group=group,
        ax=ax,
        name=i,
        reg=param_reg,
        field=f,
        ylabel=l
      )

    plot_deg_dist(
      group=group,
      ax=ax_dist_deg,
      name=i
    )

    plot_main(
      group=group, 
      ax=ax_main, 
      colour=colour, 
      name=i, 
      bounds=bounds
    )

    plot_energy(
      ax=ax_energy, 
      group=group, 
      colour=colour
    )

    plot_relax(
      group=group, 
      ax=ax_relax, 
      colour=colour
    )

  title = f"""size: {
      groups[0].descs[0]['config']['size'] 
    }, \#samples/type: {
      len(groups[0].data)
    } \nregular: {
      param_reg['regular']
    }\n irregular: {
      param_reg['irregular']
    }"""

  for ax in axs:
    ax.legend(fontsize=18)

  for fig in figs:
    fig.suptitle(title, fontsize=20)
    fig.tight_layout()

  for i, fig in enumerate(figs):
    fig.savefig(f'figures/plot{i}.png', dpi=300)

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
