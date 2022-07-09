import numpy as np
from scipy.optimize import curve_fit

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

  return xs[left_lim:pos_len], ys[left_lim:pos_len]

def fit_plot(xs, ys, bounds, f=mt_fit):
  xn, yn = slice_data(xs, ys)
  
  popt, _ = curve_fit(f, xn, yn, bounds=bounds, maxfev=1e12) 
  return xn, [f(t, *popt) for t in xn], popt
