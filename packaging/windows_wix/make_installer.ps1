python configure.py
pandoc -f markdown -t rtf ..\..\LICENSE.md -s -o LICENSE.rtf

"%WIX%\candle" indentex.wxs
"%WIX%\light" indentex.wixobj -ext WixUIExtension -sice:ICE91
