from package_info import get_version
from string import Template


def configure_wxs_file(**context):
    with open("indentex_template.wxs") as f:
        tmpl = Template(f.read())

    with open("indentex.wxs", "w") as f:
        f.write(tmpl.substitute(context))


if __name__ == "__main__":
    configure_wxs_file(version=get_version())
