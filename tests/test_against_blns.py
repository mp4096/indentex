from __future__ import print_function
from subprocess import run
import sys
from urllib.request import urlretrieve


def fetch_blns():
    print("Fetching the Big List of Naughty Strings...")
    try:
        urlretrieve(
            "https://raw.githubusercontent.com/minimaxir/"
            "big-list-of-naughty-strings/master/blns.txt",
            filename="./blns/blns.inden.tex"
            )
        status = 0
    except Exception as e:
        print(e)
        status = 1
    return status


def run_indentex():
    print("Running indentex...")
    return run(["indentex", "-v", "./blns/"]).returncode


if __name__ == "__main__":
    status = fetch_blns()
    if status != 0:
        sys.exit(status)
    sys.exit(run_indentex())
