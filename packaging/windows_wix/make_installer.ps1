cargo build --release --target=x86_64-pc-windows-msvc
cp ..\..\target\x86_64-pc-windows-msvc\release\indentex.exe .

python configure.py
pandoc -f markdown -t rtf ..\..\LICENSE.md -s -o LICENSE.rtf

& "$env:WIX\bin\candle" indentex.wxs
& "$env:WIX\bin\light" indentex.wixobj -ext WixUIExtension -sice:ICE91

python rename_installer.py
