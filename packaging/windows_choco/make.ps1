New-Item package -ItemType directory -Force | Out-Null
$VersionOutput = $(cargo run --release --target=x86_64-pc-windows-msvc -- -V)
Copy-Item ..\..\target\x86_64-pc-windows-msvc\release\indentex.exe .\package

if (!($VersionOutput -match "indentex (\S+)")) {
    Throw "Could not parse indentex version"
}
$PackageVersion = $Matches[1]

(Get-Content .\indentex_template.nuspec) `
    -replace '{{{version}}}', "$PackageVersion" `
    | Set-Content .\package\indentex.nuspec

choco pack .\package\indentex.nuspec
