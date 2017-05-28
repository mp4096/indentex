python configure.py
pandoc -f markdown -t rtf ..\..\LICENSE.md -s -o LICENSE.rtf

candle indentex.wxs
light indentex.wixobj -ext WixUIExtension -sice:ICE91
