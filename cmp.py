import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import sys

if len(sys.argv) < 2:
  raise 'no plotting arguments provided'
  exit(1)

plot_type, *args = sys.argv[1:]


