[CmdletBinding()]
param(
    [string]$InstallRoot = (Join-Path $env:LOCALAPPDATA 'WhaleLink'),
    [switch]$Force
)

$ErrorActionPreference = 'Stop'
$source = Join-Path $PSScriptRoot 'portable'
if (-not (Test-Path -LiteralPath $source)) { throw 'The portable payload is missing beside this installer.' }

$installPath = [System.IO.Path]::GetFullPath($InstallRoot)
$localAppDataPath = [System.IO.Path]::GetFullPath($env:LOCALAPPDATA)
if (-not $installPath.StartsWith($localAppDataPath + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw 'InstallRoot must be below the current user LocalAppData directory.'
}
if ((Test-Path -LiteralPath $installPath) -and -not $Force) {
    throw "WhaleLink is already installed at $installPath. Run the bundled uninstaller, or pass -Force after backing up local state."
}

$parent = Split-Path -Parent $installPath
$staging = Join-Path $parent ('.whalelink-staging-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force -Path $staging | Out-Null
try {
    Copy-Item -LiteralPath $source -Destination (Join-Path $staging 'app') -Recurse -Force
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'Uninstall-WhaleLink.ps1') -Destination $staging -Force
    if (Test-Path -LiteralPath $installPath) { Remove-Item -LiteralPath $installPath -Recurse -Force }
    Move-Item -LiteralPath $staging -Destination $installPath
}
finally {
    if (Test-Path -LiteralPath $staging) { Remove-Item -LiteralPath $staging -Recurse -Force }
}

Write-Output "PASS: WhaleLink installed for the current user at $installPath"
