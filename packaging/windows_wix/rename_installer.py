import os
from package_info import get_version


ARCH = "amd64"


def rename_installer():
    target_file = "indentex_{}_{}.msi".format(get_version(), ARCH)
    os.remove(target_file)
    os.rename("indentex.msi", target_file)


if __name__ == "__main__":
    rename_installer()
