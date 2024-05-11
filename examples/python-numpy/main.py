import sys
import numpy as np
import pandas as pd
import subprocess

print(np)
print(pd)

arr = np.random.rand(2, 3)
print(arr)

print("Hello from Python numpy and pandas")

# with the wrong LD_LIBRARY_PATH, this will fail with a GLIBC version mismatch
result = subprocess.run(["apt", "--version"], capture_output=True, text=True)
print(result.stdout)

# fail if subprocess fails!
sys.exit(result.returncode)
