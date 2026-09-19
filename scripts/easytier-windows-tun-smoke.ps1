#Requires -RunAsAdministrator
[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string]$ReleaseDirectory,
    [Parameter(Mandatory)] [string]$ResultPath
)

$ErrorActionPreference = 'Stop'
function Get-FreePort {
    for ($attempt = 0; $attempt -lt 50; $attempt++) {
        $candidate = Get-Random -Minimum 24000 -Maximum 40000
        if ($script:AllocatedPorts -notcontains $candidate -and -not (Get-NetTCPConnection -LocalPort $candidate -ErrorAction SilentlyContinue)) {
            $script:AllocatedPorts += $candidate
            return $candidate
        }
    }
    throw 'No free local test port was found.'
}

$release = [IO.Path]::GetFullPath($ReleaseDirectory)
$daemon = Join-Path $release 'bin\whalelinkd.exe'
$core = Join-Path $release 'easytier\easytier-windows-x86_64\easytier-core.exe'
if (-not (Test-Path -LiteralPath $daemon) -or -not (Test-Path -LiteralPath $core)) {
    throw 'Release payload is incomplete.'
}

$suffix = [Guid]::NewGuid().ToString('N').Substring(0, 10)
$network = "wl-tun-$suffix"
$secret = [Guid]::NewGuid().ToString('N')
$subnet = "10.$(Get-Random -Minimum 100 -Maximum 200).$((Get-Random -Minimum 1 -Maximum 250))"
$addressA = "$subnet.10"
$addressB = "$subnet.20"
$AllocatedPorts = @()
$listenerPort = Get-FreePort
$rpcA = Get-FreePort
$rpcB = Get-FreePort
$a = $null
$b = $null
try {
    $a = Start-Process -WindowStyle Hidden -PassThru -FilePath $core -ArgumentList @(
        '-i', "$addressA/24", '--network-name', $network, '--network-secret', $secret,
        '--listeners', "tcp:127.0.0.1:$listenerPort", '--rpc-portal', "127.0.0.1:$rpcA",
        '--disable-upnp', 'true', '--console-log-level', 'error'
    )
    $env:WHALELINK_NETWORK_SECRET = $secret
    $b = Start-Process -WindowStyle Hidden -PassThru -FilePath $daemon -ArgumentList @(
        'connect', '--executable', $core, '--network-name', $network,
        '--relay', "tcp://127.0.0.1:$listenerPort", '--ipv4', "$addressB/24",
        '--rpc-portal', "127.0.0.1:$rpcB", '--no-listener'
    )
    $reachable = $false
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        Start-Sleep -Milliseconds 500
        if ($b.HasExited) { throw 'Packaged daemon exited before TUN verification.' }
        if (Test-Connection -ComputerName $addressA -Count 1 -Quiet -ErrorAction SilentlyContinue) {
            $reachable = $true
            break
        }
    }
    if (-not $reachable) { throw 'The random local TUN peer did not respond to ICMP.' }
    Set-Content -LiteralPath $ResultPath -Value 'PASS: elevated packaged two-node TUN and ICMP smoke test completed.' -NoNewline
}
catch {
    Set-Content -LiteralPath $ResultPath -Value "FAIL: $($_.Exception.Message)" -NoNewline
    exit 1
}
finally {
    # Do not rely on a stale Process.HasExited snapshot: this cleanup must
    # also run when one of the nodes failed while creating its virtual NIC.
    if ($null -ne $b) {
        Stop-Process -Id $b.Id -Force -ErrorAction SilentlyContinue
        Wait-Process -Id $b.Id -Timeout 3 -ErrorAction SilentlyContinue
    }
    if ($null -ne $a) {
        Stop-Process -Id $a.Id -Force -ErrorAction SilentlyContinue
        Wait-Process -Id $a.Id -Timeout 3 -ErrorAction SilentlyContinue
    }
    Remove-Item Env:\WHALELINK_NETWORK_SECRET -ErrorAction SilentlyContinue
}
