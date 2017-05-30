python configure.py
pandoc -f markdown -t rtf ..\..\LICENSE.md -s -o LICENSE.rtf

& "$env:WIX\bin\candle" indentex.wxs
& "$env:WIX\bin\light" indentex.wixobj -ext WixUIExtension -sice:ICE91

python rename_installer.py
