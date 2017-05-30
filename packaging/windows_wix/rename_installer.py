import os
from package_info import get_version


ARCH = "amd64"


def rename_installer():
    os.rename("indentex.msi", "indentex_{}_{}.msi".format(get_version(), ARCH))


if __name__ == "__main__":
    rename_installer()
