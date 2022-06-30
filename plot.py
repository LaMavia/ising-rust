import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import re
import sys
from functools import reduce

def plot(xi: int, yi: int, path: str):
  df = pd.read_csv(path, sep=' ')

  labels = list(df.columns)
  xs = df[labels[xi]]
  ys = df[labels[yi]]

  _, _, avgs = reduce(
    lambda u, y: (n_:= u[0] + 1, s_ := u[1] + y, [*u[2], s_ / n_]), 
    ys, 
    (0, 0, []))

  plt.scatter(xs, ys, label=path)
  plt.plot(xs, avgs, label=f'[rolling avg]{path}', color=(0, 0, 0, 0.7))

for path in sys.argv[3:]:
  plot(xi=int(sys.argv[1]), yi=int(sys.argv[2]), path=path)

plt.tight_layout()
plt.legend()

plt.savefig('nsq.png')
plt.show()
