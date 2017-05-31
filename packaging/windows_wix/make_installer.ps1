cargo build --release --target=x86_64-pc-windows-msvc
Copy-Item ..\..\target\x86_64-pc-windows-msvc\release\indentex.exe .

$PackageVersion = $(python package_info.py)

(Get-Content .\indentex_template.wxs) `
    -replace '{{{version}}}', "$PackageVersion" `
    | Set-Content .\indentex.wxs

pandoc -f markdown -t rtf ..\..\LICENSE.md -s -o LICENSE.rtf

& "$env:WIX\bin\candle" indentex.wxs
& "$env:WIX\bin\light" indentex.wixobj -ext WixUIExtension -sice:ICE91

$TargetFile = "indentex_${PackageVersion}_amd64.msi"
Remove-Item $TargetFile -ErrorAction Ignore
Rename-Item indentex.msi $TargetFile
