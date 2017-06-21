$ErrorActionPreference = "Stop"

$VersionOutput = $(cargo run --release --target=x86_64-pc-windows-msvc -- -V)
Copy-Item ..\..\target\x86_64-pc-windows-msvc\release\indentex.exe .\package

if (!($VersionOutput -match "indentex (\S+)")) {
    Throw "Could not parse indentex version"
}
$PackageVersion = $Matches[1]

Copy-Item ..\..\LICENSE.md .\package\LICENSE.txt

choco pack .\package\indentex.nuspec --version $PackageVersion
