cargo build --release --target=x86_64-pc-windows-msvc
Copy-Item ..\..\target\x86_64-pc-windows-msvc\release\indentex.exe .

python configure.py
pandoc -f markdown -t rtf ..\..\LICENSE.md -s -o LICENSE.rtf

& "$env:WIX\bin\candle" indentex.wxs
& "$env:WIX\bin\light" indentex.wixobj -ext WixUIExtension -sice:ICE91

$TargetFile = "indentex_$(python package_info.py)_amd64.msi"
Remove-Item $TargetFile -ErrorAction Ignore
Rename-Item indentex.msi $TargetFile
