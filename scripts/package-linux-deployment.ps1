param(
    [string]$OutputDirectory = (Join-Path $PSScriptRoot '..\artifacts\linux-x64-deployment')
)

$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$outputPath = [System.IO.Path]::GetFullPath($OutputDirectory)
$artifactsRoot = [System.IO.Path]::GetFullPath((Join-Path $root 'artifacts'))
if (-not $outputPath.StartsWith($artifactsRoot + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw 'OutputDirectory must be inside the repository artifacts directory.'
}
$asset = Join-Path $root 'easytier-linux-x86_64-v2.6.4.zip'
$licenseDirectory = Join-Path $root 'vendor\easytier\licenses\v2.6.4'
$expected = '61b659eaedba658fa66fe47d17e1426cdd77e5d02fa15fed447bb4357c09dfd6'
if (-not (Test-Path -LiteralPath $asset)) { throw "Missing locked Linux EasyTier asset: $asset" }
if ((Get-FileHash -Algorithm SHA256 -LiteralPath $asset).Hash.ToLowerInvariant() -ne $expected) {
    throw 'Linux EasyTier SHA-256 does not match docs/easytier-lock.toml.'
}
foreach ($license in @(
    @{ File = 'LICENSE'; Sha256 = 'e3a994d82e644b03a792a930f574002658412f62407f5fee083f2555c5f23118' },
    @{ File = 'GPL-3.0.txt'; Sha256 = '3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986' }
)) {
    $licensePath = Join-Path $licenseDirectory $license.File
    if (-not (Test-Path -LiteralPath $licensePath) -or (Get-FileHash -LiteralPath $licensePath -Algorithm SHA256).Hash.ToLowerInvariant() -ne $license.Sha256) {
        throw "Missing or invalid locked license text: $($license.File). Run scripts/stage-easytier-license.ps1."
    }
}

if (Test-Path -LiteralPath $outputPath) { Remove-Item -LiteralPath $outputPath -Recurse -Force }
$bundle = Join-Path $outputPath 'WhaleLink-v3-linux-x64-deployment'
New-Item -ItemType Directory -Force -Path $bundle | Out-Null

Copy-Item -LiteralPath (Join-Path $root 'Cargo.toml'), (Join-Path $root 'Cargo.lock'), (Join-Path $root 'rust-toolchain.toml'), (Join-Path $root 'LICENSE'), (Join-Path $root 'THIRD_PARTY_NOTICES.md') -Destination $bundle
foreach ($directory in @('apps', 'crates', 'deploy', 'config', 'docs')) {
    $source = Join-Path $root $directory
    $destination = Join-Path $bundle $directory
    # A source deployment bundle must not contain local compiler caches or
    # application outputs; the Linux target rebuilds from Cargo.lock instead.
    # Verification reports and work logs remain in the source repository. They
    # may record this bundle's checksum, so including them would create a
    # self-referential package hash on every release rebuild.
    $robocopyArguments = @($source, $destination, '/E', '/XD', 'bin', 'obj', 'target', '.nuget', 'artifacts', 'vendor')
    if ($directory -eq 'docs') {
        $robocopyArguments += @('verification', '/XF', 'WORKLOG.md', 'WORKLOG-*.md')
    }
    robocopy.exe @robocopyArguments | Out-Null
    if ($LASTEXITCODE -gt 7) { throw "robocopy failed for $directory with exit code $LASTEXITCODE" }
}
Copy-Item -LiteralPath $asset -Destination $bundle
New-Item -ItemType Directory -Force -Path (Join-Path $bundle 'licenses') | Out-Null
Copy-Item -LiteralPath (Join-Path $licenseDirectory 'LICENSE') -Destination (Join-Path $bundle 'licenses\EasyTier-LGPL-3.0.txt')
Copy-Item -LiteralPath (Join-Path $licenseDirectory 'GPL-3.0.txt') -Destination (Join-Path $bundle 'licenses\GPL-3.0.txt')
& (Join-Path $root 'scripts\generate-sbom.ps1') -OutputPath (Join-Path $bundle 'SBOM.spdx.json') -PackageName 'WhaleLink-v3-linux-x64-deployment'
if ($LASTEXITCODE -ne 0) { throw 'SBOM generation failed.' }

$archive = Join-Path $outputPath 'WhaleLink-v3-linux-x64-deployment.zip'
Compress-Archive -Path (Join-Path $bundle '*') -DestinationPath $archive -CompressionLevel Optimal
Get-ChildItem -File -Recurse $outputPath | Get-FileHash -Algorithm SHA256 | ForEach-Object { "{0}  {1}" -f $_.Hash.ToLowerInvariant(), $_.Path } | Set-Content (Join-Path $outputPath 'SHA256SUMS.txt')
Write-Output "PASS: Linux deployment source bundle written to $outputPath"
