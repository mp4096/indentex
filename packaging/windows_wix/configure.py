from os import path
from string import Template
import toml


def get_version():
    with open(path.join("..", "..", "Cargo.toml")) as f:
        cargo = toml.load(f)
        return cargo["package"]["version"]


def configure_wxs_file(**context):
    with open("indentex_template.wxs") as f:
        tmpl = Template(f.read())

    with open("indentex.wxs", "w") as f:
        f.write(tmpl.substitute(context))


if __name__ == "__main__":
    configure_wxs_file(version=get_version())
