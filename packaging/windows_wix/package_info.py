from __future__ import print_function
from os import path
import toml


def get_version():
    with open(path.join("..", "..", "Cargo.toml")) as f:
        cargo = toml.load(f)
        return cargo["package"]["version"]


if __name__ == "__main__":
    print(get_version())
