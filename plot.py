import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import sys

f = pd.read_csv(f'{sys.argv[1]}.csv')

f.plot("H", "M")

plt.savefig(f'{sys.argv[1]}.png')
plt.show()

