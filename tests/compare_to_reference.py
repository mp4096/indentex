from __future__ import print_function
import filecmp
import fnmatch
import os
import sys


def find_reference_files():
    """Yield all reference files recursively."""
    for root, _, files in os.walk("./tests/references/"):
        for basename in fnmatch.filter(files, "*.tex"):
            yield os.path.join(root, basename)


def lookup_transpiled_file(reference_filename):
    file_stem = os.path.splitext(os.path.basename(reference_filename))[0]
    return "./tests/test_cases/{:s}_indentex.tex".format(file_stem)


if __name__ == "__main__":
    status = 0
    for r in find_reference_files():
        t = lookup_transpiled_file(r)
        print("Comparing '{:s}' and '{:s}'...".format(r, t))
        if not filecmp.cmp(r, t, shallow=False):
            status = 1
            print("    NOK\n")
        else:
            print("    OK\n")
    sys.exit(status)
