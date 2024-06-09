import sys
import subprocess

print(sys.version)

poetry_version = subprocess.run(
    ["poetry", "--version"], capture_output=True, text=True
).stdout.strip()
print(poetry_version)
