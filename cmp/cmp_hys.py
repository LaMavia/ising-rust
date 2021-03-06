#!/bin/python3

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import sys
import pathlib
import plot_constants
import json
from dataclasses import dataclass, field
from collections.abc import Iterable
from typing import TypeVar

def dict_of_path(path: str) -> dict:
  xs = path.split("/")
  main = dict([tuple([float(b) if all([c == '.' or c.isnumeric() for c in b]) else b for b in a.split("=")]) for a in xs[len(xs) - 2].split("_")])

  main['lattice_type'] = xs[1]

  return main

@dataclass
class DataPoint:
  lattice_type: str
  ts: list[float]
  ns: list[float]
  hs: list[float]
  ms: list[float]
  es: list[float]
  esa: list[float]
  desc: dict

@dataclass
class Group:
  seed: float
  temp: float
  data: list[DataPoint] = field(default_factory=list)

U = TypeVar('U')
V = TypeVar('V')
def uzip(ps: list[tuple[U, V]]) -> tuple[list[U], list[V]]:
  xs = []
  ys = []

  for x, y in ps:
    xs.append(x)
    ys.append(y)

  return xs, ys

def process_data(desc_paths: list[str]) -> list[Group]:
  buckets: dict[tuple[float, float], Group] = dict()

  for desc_path in desc_paths:
    desc_f = open(desc_path, 'r')
    desc = json.loads(desc_f.read())
    desc_f.close()

    params = dict_of_path(desc_path)
    key = (params['seed'], params['temp'])

    if key not in buckets:
      buckets[key] = Group(
        seed=params['seed'],
        temp=params['temp'],
        data=[]
      )

    df = pd.read_csv(desc['data_path'])

    buckets[key].data.append(
      DataPoint(
        lattice_type=params['lattice_type'],
        hs=df['H'],
        ms=df['M'],
        ts=df['t'],
        ns=df['n'],
        es=df['E'],
        esa=df['aE'],
        desc=desc
      )
    )

  return [e[1] for e in sorted(buckets.items(), key=lambda e: e[0])]

def style_of_type(lattice_type: str) -> tuple[str, float]:
  return ('--', 0.5) if lattice_type == 'regular' else ('-', 1)

def plot(group: Group, ax: plt.Axes, colour: list[float], name: int, label: str, doLegend: bool):
  ax.set_xlabel(r'$H$')
  ax.set_ylabel(r'$M$')
  ax.set_title(label, fontsize=18)

  for dp in group.data:
    linestyle, alpha = ('--', 0.5) if dp.lattice_type == 'regular' else ('-', 1)
    ax.plot(dp.hs, dp.ms, linestyle=linestyle, color=(*colour, alpha), label=f"""{dp.lattice_type}""")

def plot_energy(group: Group, ax: plt.Axes, colour: list[float], name: int, label: str):
  size = float(group.data[0].desc['config']['size'])**2
  max_t = max([t for dp in group.data for t in dp.ts])

  ax.set_xlabel(r'$t$ [MC sweep]')
  ax.set_ylabel(r'$\langle\mathcal{H}\rangle$')
  ax.set_title(label, fontsize=18)
 
  for dp in group.data:
    max_local_t = max(dp.ts)
    t_ratio = max_t/max_local_t

    linestyle, alpha = style_of_type(dp.lattice_type)

    ax.plot(
      [t * t_ratio for t in dp.ts], [e / size for e in dp.es], 
      linestyle=linestyle, 
      color=(*colour, alpha),
      label=f"""[{dp.lattice_type}] $t' = {(1/t_ratio):.3f}t$"""
      )

    ax.legend(fontsize=18)

def plot_energy_h(group: Group, ax: plt.Axes, colour: list[float], name: int, label: str):
  size = float(group.data[0].desc['config']['size'])**2

  ax.set_xlabel(r'$H$')
  ax.set_ylabel(r'$\langle\mathcal{H}\rangle$')
  ax.set_title(label, fontsize=18)
 
  for dp in group.data:
    linestyle, alpha = ('--', 0.5) if dp.lattice_type == 'regular' else ('-', 1)
    ax.plot(
      dp.hs, [e / size for e in dp.es], 
      linestyle=linestyle, 
      color=(*colour, alpha),
      label=f"""{dp.lattice_type}"""
      )

    ax.legend(fontsize=18)

def flatten(xss):
  if not isinstance(xss, Iterable):
    return [xss]

  try:
    return [x for xs in xss for x in xs]
  except:
    return xss

def main(paths):
  size = None
  h_step = None

  plt.rcParams['text.usetex'] = True
  plt.rcParams['axes.labelsize'] = 18

  row_size=6
  col_size=5

  groups = process_data(paths)
  temps = sorted(list(set(g.temp for g in groups)))
  seeds = sorted(list(set(g.seed for g in groups)))
  colours = plot_constants.plot_colours
  legend_handles: dict[tuple[float, float], list[plt.Line2D]] = dict()

  n_rows = len(seeds)
  n_cols = len(temps)

  figs = [plt.figure(figsize=(n_cols * col_size, n_rows * row_size), dpi=300) for _ in range(3)]
  figs_red = [plt.figure(figsize=(n_cols * col_size, n_rows * 2), dpi=300) for _ in range(3)]
  fig_hys, fig_energy, fig_energy_h = figs

  ax_groups = [flatten(fig.subplots(nrows=n_rows, ncols=n_cols)) for fig in figs]
  axes_hys, axes_energy, axes_energy_h = ax_groups

  ax_groups_red = [flatten(fig.subplots(nrows=2, ncols=n_cols)) for fig in figs_red]
  axes_hys_red, axes_energy_red, axes_energy_h_red = ax_groups_red

  for axes in [*ax_groups, *ax_groups_red]:
    for ax in axes:
      ax.grid(which='both')
      ax.tick_params(which='both', labelsize=16)

  colour_keys = list(colours.keys())

  for i, g in enumerate(groups):
    print(f"""[plot] plotting seed={g.seed}, temp={g.temp}""")
    i_colour = temps.index(g.temp)
    i_seed = seeds.index(g.seed)
    label = f'seed={g.seed} $T$={g.temp}'

    colour = colours[colour_keys[i_colour % len(colour_keys)]]

    if i_seed <= 1:
      plot(group=g, ax=axes_hys_red[i], colour=colour, name=i, label=label, doLegend=(i_colour == 0))
      plot_energy(group=g, ax=axes_energy_red[i], colour=colour, name=i, label=label)
      plot_energy_h(group=g, ax=axes_energy_h_red[i], colour=colour, name=i, label=label)

    plot(group=g, ax=axes_hys[i], colour=colour, name=i, label=label, doLegend=(i_colour == 0))
    plot_energy(group=g, ax=axes_energy[i], colour=colour, name=i, label=label)
    plot_energy_h(group=g, ax=axes_energy_h[i], colour=colour, name=i, label=label)

    if size == None or h_step == None:
      size = size or g.data[0].desc['config']['size']
      h_step = h_step or g.data[0].desc['config']['h_step']

  for i, fig in enumerate(figs):
    fig.suptitle(f'dashed - regular, solid - irregular\nN={size} $\\Delta H$={h_step}\n\n', fontsize=20)
    fig.tight_layout()

    fig.savefig(f'figures/plot_hys{i}.png', dpi=300)

  for i, fig in enumerate(figs_red):
    fig.suptitle(f'dashed - regular, solid - irregular\nN={size} $\\Delta H$={h_step}\n\n', fontsize=20)
    fig.tight_layout()

    fig.savefig(f'figures/plot_hys_red{i}.png', dpi=300)



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