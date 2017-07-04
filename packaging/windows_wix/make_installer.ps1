$ErrorActionPreference = "Stop"

$VersionOutput = $(cargo run --release --target=x86_64-pc-windows-msvc -- -V)
Copy-Item ..\..\target\x86_64-pc-windows-msvc\release\indentex.exe .

if (!($VersionOutput -match "indentex (\S+)")) {
    Throw "Could not parse indentex version"
}
$PackageVersion = $Matches[1]

(Get-Content .\indentex_template.wxs) `
    -replace '{{{version}}}', "$PackageVersion" `
    | Set-Content .\indentex.wxs

pandoc -f markdown -t rtf ..\..\LICENSE.md -s -o LICENSE.rtf

$TargetFile = "indentex_${PackageVersion}_amd64.msi"

& "$env:WIX\bin\candle" indentex.wxs
& "$env:WIX\bin\light" indentex.wixobj -ext WixUIExtension -sice:ICE91 -o $TargetFile

$FileHash = (Get-FileHash $TargetFile -Algorithm SHA512).Hash.ToLower()
Set-Content ".\$TargetFile.DIGEST" "$FileHash  $TargetFile"
