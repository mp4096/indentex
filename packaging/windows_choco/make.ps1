$ErrorActionPreference = "Stop"

$VersionOutput = $(cargo run --release --target=x86_64-pc-windows-msvc -- -V)
Copy-Item ..\..\target\x86_64-pc-windows-msvc\release\indentex.exe .\package

if (!($VersionOutput -match "indentex (\S+)")) {
    Throw "Could not parse indentex version"
}
$PackageVersion = $Matches[1]

Copy-Item ..\..\LICENSE.md .\package\LICENSE.txt
$Checksum = (Get-FileHash .\package\indentex.exe -Algorithm SHA256).Hash
(((Get-Content .\VERIFICATION_TEMPLATE.txt) `
    -replace '{{{tag}}}', "$env:APPVEYOR_REPO_TAG_NAME") `
    -replace '{{{checksum}}}', "$Checksum") `
    -replace '{{{commit}}}', "$env:APPVEYOR_REPO_COMMIT" `
    | Set-Content .\package\VERIFICATION.txt

choco pack .\package\indentex.nuspec --version $PackageVersion
