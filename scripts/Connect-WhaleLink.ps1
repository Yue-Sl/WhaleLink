#Requires -RunAsAdministrator
[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string]$NetworkName,
    [Parameter(Mandatory)] [string]$Relay,
    [string]$IPv4
)

$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
$daemon = Join-Path $root 'bin\whalelinkd.exe'
$core = Join-Path $root 'easytier\easytier-windows-x86_64\easytier-core.exe'
if (-not (Test-Path -LiteralPath $daemon) -or -not (Test-Path -LiteralPath $core)) {
    throw 'WhaleLink runtime files are missing. Run this script from the portable package root.'
}

$secureSecret = Read-Host 'EasyTier network secret' -AsSecureString
$bstr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secureSecret)
try {
    $plainSecret = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($bstr)
    if ([string]::IsNullOrWhiteSpace($plainSecret)) { throw 'Network secret must not be empty.' }
    $env:WHALELINK_NETWORK_SECRET = $plainSecret
    $arguments = @(
        'connect', '--executable', $core, '--network-name', $NetworkName,
        '--relay', $Relay, '--no-listener'
    )
    if (-not [string]::IsNullOrWhiteSpace($IPv4)) { $arguments += @('--ipv4', $IPv4) }
    & $daemon @arguments
}
finally {
    Remove-Item Env:\WHALELINK_NETWORK_SECRET -ErrorAction SilentlyContinue
    if ($bstr -ne [IntPtr]::Zero) { [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr) }
}
