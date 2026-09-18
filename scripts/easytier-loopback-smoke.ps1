$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$bin = Join-Path $root 'vendor\easytier\v2.6.4\easytier-windows-x86_64'
$core = Join-Path $bin 'easytier-core.exe'
$cli = Join-Path $bin 'easytier-cli.exe'
$network = 'wl-smoke-' + [guid]::NewGuid().ToString('N').Substring(0, 12)
$secret = [guid]::NewGuid().ToString('N')
$common = @('--network-name', $network, '--network-secret', $secret, '--no-tun', '--disable-upnp', '--console-log-level', 'error')
$nodeA = Start-Process -FilePath $core -ArgumentList ($common + @('--listeners', 'tcp:19000', '--rpc-portal', '127.0.0.1:19100')) -WindowStyle Hidden -PassThru
$nodeB = $null
try {
    Start-Sleep -Milliseconds 500
    $nodeB = Start-Process -FilePath $core -ArgumentList ($common + @('--no-listener', '--peers', 'tcp://127.0.0.1:19000', '--rpc-portal', '127.0.0.1:19101')) -WindowStyle Hidden -PassThru
    $peers = $null
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        Start-Sleep -Milliseconds 250
        try {
            $raw = & $cli --rpc-portal 127.0.0.1:19101 --output json peer 2>$null
            if ($LASTEXITCODE -eq 0 -and $raw) { $peers = $raw | ConvertFrom-Json; break }
        } catch { }
    }
    if ($null -eq $peers) { throw 'EasyTier peer RPC did not become available.' }
    if (($peers | ConvertTo-Json -Compress).Length -lt 3) { throw 'EasyTier peer result was empty.' }
    Write-Output 'PASS: two EasyTier no-TUN loopback nodes established and peer RPC responded.'
}
finally {
    if ($null -ne $nodeB -and -not $nodeB.HasExited) { Stop-Process -Id $nodeB.Id -Force }
    if ($null -ne $nodeA -and -not $nodeA.HasExited) { Stop-Process -Id $nodeA.Id -Force }
}
