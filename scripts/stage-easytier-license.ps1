$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$destination = Join-Path $root 'vendor\easytier\licenses\v2.6.4'
New-Item -ItemType Directory -Force -Path $destination | Out-Null

$assets = @(
    @{ File = 'LICENSE'; Url = 'https://raw.githubusercontent.com/EasyTier/EasyTier/v2.6.4/LICENSE'; Sha256 = 'e3a994d82e644b03a792a930f574002658412f62407f5fee083f2555c5f23118' },
    @{ File = 'GPL-3.0.txt'; Url = 'https://www.gnu.org/licenses/gpl-3.0.txt'; Sha256 = '3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986' }
)
foreach ($asset in $assets) {
    $path = Join-Path $destination $asset.File
    curl.exe --fail --location --silent --show-error --output $path $asset.Url
    if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -ne $asset.Sha256) {
        Remove-Item -LiteralPath $path -Force -ErrorAction SilentlyContinue
        throw "License hash mismatch: $($asset.File)"
    }
}
Write-Output 'PASS: locked EasyTier LGPL and GPL license texts staged.'
