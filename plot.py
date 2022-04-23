import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import sys

if len(sys.argv) < 2:
  raise 'no plotting arguments provided'
  exit(1)

plot_type, *args = sys.argv[1:]

if plot_type == 'hys':
  name, plot_title = args

  f = pd.read_csv(f'{name}.csv')

  f.plot("H", "M")
  plt.title(plot_title)

  plt.savefig(f'{name}.png')
  plt.show()

if plot_type == 'phase':
  name, plot_title = args

  f = pd.read_csv(f'{name}.csv')

  f.plot("T", "M")
  plt.title(plot_title)

  plt.savefig(f'{name}.png')
  plt.show()

else:
  print(f'unknown plot type {plot_type}')
  exit(1)

