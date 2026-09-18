[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$installPath = [System.IO.Path]::GetFullPath($PSScriptRoot)
$localAppDataPath = [System.IO.Path]::GetFullPath($env:LOCALAPPDATA)
if (-not $installPath.StartsWith($localAppDataPath + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw 'Refusing to remove a directory outside the current user LocalAppData directory.'
}
if (-not (Test-Path -LiteralPath (Join-Path $installPath 'app'))) {
    throw 'This does not look like a WhaleLink current-user installation.'
}
Remove-Item -LiteralPath $installPath -Recurse -Force
Write-Output "PASS: WhaleLink was removed from $installPath"
