param(
    [string]$OutputDirectory = (Join-Path $PSScriptRoot '..\artifacts\windows-x64')
)

$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$outputPath = [System.IO.Path]::GetFullPath($OutputDirectory)
$artifactsRoot = [System.IO.Path]::GetFullPath((Join-Path $root 'artifacts'))
if (-not $outputPath.StartsWith($artifactsRoot + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw 'OutputDirectory must be inside the repository artifacts directory.'
}
$lock = Get-Content (Join-Path $root 'docs\easytier-lock.toml') -Raw
$asset = Join-Path $root 'easytier-windows-x86_64-v2.6.4.zip'
$licenseDirectory = Join-Path $root 'vendor\easytier\licenses\v2.6.4'
$nugetConfig = Join-Path $root 'NuGet.Config'
$nugetPackages = Join-Path $root '.nuget\packages'
$nugetHome = Join-Path $root '.nuget\home'
$expected = '27af91e270e554709b048bd32327fefd2dfce5062ae1e8701af7550c6f525f84'
if (-not (Test-Path $asset)) { throw "Missing locked EasyTier asset: $asset" }
if ((Get-FileHash -Algorithm SHA256 $asset).Hash.ToLowerInvariant() -ne $expected) { throw 'EasyTier SHA-256 does not match docs/easytier-lock.toml.' }
foreach ($license in @(
    @{ File = 'LICENSE'; Sha256 = 'e3a994d82e644b03a792a930f574002658412f62407f5fee083f2555c5f23118' },
    @{ File = 'GPL-3.0.txt'; Sha256 = '3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986' }
)) {
    $licensePath = Join-Path $licenseDirectory $license.File
    if (-not (Test-Path -LiteralPath $licensePath) -or (Get-FileHash -LiteralPath $licensePath -Algorithm SHA256).Hash.ToLowerInvariant() -ne $license.Sha256) {
        throw "Missing or invalid locked license text: $($license.File). Run scripts/stage-easytier-license.ps1."
    }
}

# A restricted service account can be unable to read the interactive user's
# AppData NuGet.Config. Keep every NuGet lookup used for the release inside
# the ignored repository cache, without changing the user's configuration.
New-Item -ItemType Directory -Force -Path $nugetPackages, $nugetHome | Out-Null
$env:APPDATA = $nugetHome
$env:LOCALAPPDATA = $nugetHome
$env:DOTNET_CLI_HOME = $nugetHome
$env:NUGET_PACKAGES = $nugetPackages

# A release directory must never retain an old file that was not rebuilt in
# this run. The path validation above limits cleanup to generated artifacts.
if (Test-Path $outputPath) { Remove-Item -LiteralPath $outputPath -Recurse -Force }
New-Item -ItemType Directory -Force -Path $outputPath | Out-Null

$cargo = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
& $cargo build --release --package whalelinkd --package whalelink-cli
if ($LASTEXITCODE -ne 0) { throw 'Rust release build failed.' }
# Use repository-scoped restore state rather than a build-account's private
# NuGet settings. The runtime graph must be restored before a RID publish.
dotnet restore (Join-Path $root 'apps\WhaleLink.Desktop\WhaleLink.Desktop.csproj') --runtime win-x64 --locked-mode --ignore-failed-sources --configfile $nugetConfig --packages $nugetPackages
if ($LASTEXITCODE -ne 0) { throw 'Desktop runtime restore failed.' }
New-Item -ItemType Directory -Force -Path (Join-Path $outputPath 'portable\bin') | Out-Null
dotnet publish (Join-Path $root 'apps\WhaleLink.Desktop\WhaleLink.Desktop.csproj') --configuration Release --runtime win-x64 --self-contained false --no-restore --output (Join-Path $outputPath 'portable\desktop')
if ($LASTEXITCODE -ne 0) { throw 'Desktop publish failed.' }

Copy-Item (Join-Path $root 'target\release\whalelinkd.exe') (Join-Path $outputPath 'portable\bin')
Copy-Item (Join-Path $root 'target\release\whalelink-cli.exe') (Join-Path $outputPath 'portable\bin')
Expand-Archive -LiteralPath $asset -DestinationPath (Join-Path $outputPath 'portable\easytier') -Force
Copy-Item (Join-Path $root 'LICENSE') (Join-Path $outputPath 'portable')
Copy-Item (Join-Path $root 'THIRD_PARTY_NOTICES.md') (Join-Path $outputPath 'portable')
Copy-Item (Join-Path $root 'README.md') (Join-Path $outputPath 'portable')
Copy-Item (Join-Path $root 'scripts\Connect-WhaleLink.ps1') (Join-Path $outputPath 'portable')
New-Item -ItemType Directory -Force -Path (Join-Path $outputPath 'portable\docs') | Out-Null
Copy-Item (Join-Path $root 'docs\DEPLOYMENT.md') (Join-Path $outputPath 'portable\docs')
Copy-Item -LiteralPath (Join-Path $root 'docs\USER_GUIDE.md') -Destination (Join-Path $outputPath 'portable\docs\WhaleLink-User-Guide.md')
New-Item -ItemType Directory -Force -Path (Join-Path $outputPath 'portable\licenses') | Out-Null
Copy-Item -LiteralPath (Join-Path $licenseDirectory 'LICENSE') -Destination (Join-Path $outputPath 'portable\licenses\EasyTier-LGPL-3.0.txt')
Copy-Item -LiteralPath (Join-Path $licenseDirectory 'GPL-3.0.txt') -Destination (Join-Path $outputPath 'portable\licenses\GPL-3.0.txt')
& (Join-Path $root 'scripts\generate-sbom.ps1') -OutputPath (Join-Path $outputPath 'portable\SBOM.spdx.json') -PackageName 'WhaleLink-v3-win-x64'
if ($LASTEXITCODE -ne 0) { throw 'SBOM generation failed.' }

$portableZip = Join-Path $outputPath 'WhaleLink-v3-win-x64-portable.zip'
Compress-Archive -Path (Join-Path $outputPath 'portable\*') -DestinationPath $portableZip -CompressionLevel Optimal
$installer = Join-Path $outputPath 'installer'
New-Item -ItemType Directory -Force -Path $installer | Out-Null
Copy-Item -LiteralPath (Join-Path $outputPath 'portable') -Destination $installer -Recurse -Force
Copy-Item -LiteralPath (Join-Path $root 'scripts\Install-WhaleLink.ps1') -Destination $installer
Copy-Item -LiteralPath (Join-Path $root 'scripts\Uninstall-WhaleLink.ps1') -Destination $installer
$installerZip = Join-Path $outputPath 'WhaleLink-v3-win-x64-installer.zip'
Compress-Archive -Path (Join-Path $installer '*') -DestinationPath $installerZip -CompressionLevel Optimal
Get-ChildItem -File -Recurse $outputPath | Get-FileHash -Algorithm SHA256 | ForEach-Object { "{0}  {1}" -f $_.Hash.ToLowerInvariant(), $_.Path } | Set-Content (Join-Path $outputPath 'SHA256SUMS.txt')
Write-Output "PASS: Windows portable and current-user installer artifacts written to $outputPath"
