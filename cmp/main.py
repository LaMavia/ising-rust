#!/bin/python3
import cmp_hys
import cmp_phase
import cmp_relax
import sys
import pathlib

modes = {
  'hys': cmp_hys.main,
  'phase': cmp_phase.main,
  'relax': cmp_relax.main
}

def main(mode: str, *paths: list[str]):
  return modes[mode](paths)


def print_usage():
  print(f'usage: ./main.py mode path [paths]\nmode : phase, hys')

def are_arguments_valid() -> [bool, str]:
  if len(sys.argv) < 3: 
    return [False, 'Too few arguments; expected at least 2']
  
  mode = sys.argv[1]
  if mode not in modes:
    return [False, f'''Invalid mode "{mode}"''']

  paths = sys.argv[2:]
  for path_str in paths:
    path = pathlib.Path(path_str)

    if not path.exists():
      return [False, f'''Path {path_str} doesn't exist''']
  return [True, '']

if __name__ == '__main__':
  valid, error_msg = are_arguments_valid()

  if not valid:
    print(error_msg)
    print_usage()
    exit(1)

  main(*sys.argv[1:])
