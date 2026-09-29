#!/usr/bin/env python3
"""
JARVIS OpenWorker environment setup script.

Usage:
    python scripts/setup_openworker.py

Sets up the Python virtual environment for the OpenWorker specialist server
inside vendor/openworker/. Run this once before first use, or after updating
the openworker submodule.
"""

import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
OPENWORKER_DIR = REPO_ROOT / "vendor" / "openworker"
VENV_DIR = OPENWORKER_DIR / ".venv"


def run(args, cwd=None):
    print(f"  $ {' '.join(str(a) for a in args)}")
    result = subprocess.run(args, cwd=cwd, capture_output=False)
    if result.returncode != 0:
        sys.exit(f"Command failed with exit code {result.returncode}")


def main():
    print("=== JARVIS OpenWorker Setup ===")

    if not OPENWORKER_DIR.exists():
        sys.exit(
            f"OpenWorker source not found at {OPENWORKER_DIR}.\n"
            "Run: git submodule update --init vendor/openworker"
        )

    print(f"\n[1/3] Checking Python version (need >=3.10)")
    run([sys.executable, "--version"])

    print(f"\n[2/3] Creating virtual environment at {VENV_DIR}")
    if VENV_DIR.exists():
        print("  venv already exists, skipping creation")
    else:
        run([sys.executable, "-m", "venv", str(VENV_DIR)], cwd=OPENWORKER_DIR)

    pip = (
        VENV_DIR / "Scripts" / "pip.exe"
        if sys.platform == "win32"
        else VENV_DIR / "bin" / "pip"
    )

    print(f"\n[3/3] Installing OpenWorker into venv")
    run([str(pip), "install", "-e", "."], cwd=OPENWORKER_DIR)

    python = (
        VENV_DIR / "Scripts" / "python.exe"
        if sys.platform == "win32"
        else VENV_DIR / "bin" / "python"
    )

    print("\n=== Setup complete ===")
    print(f"Test with: {python} -m openworker --help")
    print("Then start JARVIS and hit the OpenWorker panel to launch the server.")


if __name__ == "__main__":
    main()
