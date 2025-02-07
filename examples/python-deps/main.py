import shutil
import sys


def is_installed(command):
    return shutil.which(command) is not None


def main():
    commands = {"ffmpeg": "FFmpeg", "pdftoppm": "Poppler (pdftoppm)"}

    missing = False
    for cmd, name in commands.items():
        if is_installed(cmd):
            print(f"{name} is installed.")
        else:
            print(f"{name} is NOT installed.")
            missing = True

    if missing:
        sys.exit(1)

    print("Hello from python-deps")


if __name__ == "__main__":
    main()
