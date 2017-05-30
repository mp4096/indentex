from os import path
import toml


def get_version():
    with open(path.join("..", "..", "Cargo.toml")) as f:
        cargo = toml.load(f)
        return cargo["package"]["version"]
