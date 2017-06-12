#!/usr/bin/env python3
from cffi import FFI
from os import path
import sys

INPUT = b"""# equation*:
  # label: eq:test
  a + b"""

def main():
    ffi = FFI()
    ffi.cdef("""
    int indentex_transpile_flags(const void *input, size_t input_len,
        void *output, size_t output_len, int flags);
    """)

    if sys.platform == "win32":
        indentex = ffi.dlopen(path.join("target", "debug", "indentex.dll"))
    else:
        indentex = ffi.dlopen(path.join("target", "debug", "libindentex.so"))

    buf_len = 1000
    buf = ffi.new('char[]', buf_len)

    ret_val = indentex.indentex_transpile_flags(INPUT, len(INPUT), buf, buf_len, 0)

    print(ret_val)
    print(ffi.string(buf).decode("utf-8"))


if __name__ == "__main__":
    main()
