$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$vendor = Join-Path $root 'vendor\easytier\v2.6.4\easytier-windows-x86_64\easytier-core.exe'
if (-not (Test-Path -LiteralPath $vendor)) { throw "Missing verified EasyTier binary: $vendor" }
$config = Join-Path $env:TEMP ("whalelink-daemon-smoke-" + [guid]::NewGuid().ToString('N') + '.toml')
$credentialRoot = Join-Path $env:TEMP ("whalelink-daemon-credentials-" + [guid]::NewGuid().ToString('N'))
$roomId = 'daemon-smoke-room-' + [guid]::NewGuid().ToString('N').Substring(0, 8)
$network = 'daemon-smoke-network-' + [guid]::NewGuid().ToString('N').Substring(0, 8)
$secret = 'daemon-smoke-secret-' + [guid]::NewGuid().ToString('N')
$previousLocalAppData = $env:LOCALAPPDATA
$env:LOCALAPPDATA = $credentialRoot
$escapedVendor = $vendor.Replace('\', '/')
@"
schema_version = 1
[easytier]
executable = "$escapedVendor"
arguments = ["--no-tun"]
[diagnostics]
redact_sensitive_values = true
"@ | Set-Content -LiteralPath $config -NoNewline
$cargo = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
$process = Start-Process -FilePath $cargo -ArgumentList "run --quiet --package whalelinkd -- --config `"$config`" serve" -WorkingDirectory $root -WindowStyle Hidden -PassThru
function Send-Ipc($method, $params) {
    $pipe = [System.IO.Pipes.NamedPipeClientStream]::new('.', 'WhaleLink.v1', [System.IO.Pipes.PipeDirection]::InOut)
    try {
        $pipe.Connect(5000)
        $request = @{ protocol_version = 1; request_id = [guid]::NewGuid(); method = $method; params = $params } | ConvertTo-Json -Compress -Depth 5
        $writer = [System.IO.StreamWriter]::new($pipe)
        $writer.AutoFlush = $true
        $writer.WriteLine($request)
        $reader = [System.IO.StreamReader]::new($pipe)
        return ($reader.ReadLine() | ConvertFrom-Json)
    }
    finally { $pipe.Dispose() }
}
try {
    $response = Send-Ipc 'health' @{}
    if (-not $response.ok -or $response.data.status -ne 'stopped') { throw ("Named Pipe health response was invalid: " + ($response | ConvertTo-Json -Compress)) }
    $cliStatus = & $cargo run --quiet --package whalelink-cli -- status
    if ($LASTEXITCODE -ne 0 -or $cliStatus -notmatch 'stopped') { throw 'CLI status request failed.' }
    $enrollment = @{ network_name = $network; network_secret = $secret; peers = @('tcp://127.0.0.1:19000'); dhcp = $true; disable_upnp = $true }
    $imported = Send-Ipc 'enrollment.import' @{ room_id = $roomId; enrollment = $enrollment }
    if (-not $imported.ok) { throw ("Enrollment import failed: " + $imported.error.code + ' / ' + $imported.error.message) }
    $credentialDirectory = Join-Path $credentialRoot 'WhaleLink\credentials'
    $acl = Get-Acl -LiteralPath $credentialDirectory
    if (-not $acl.AreAccessRulesProtected) { throw 'Daemon credential directory ACL inherited from its parent.' }
    $currentSid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
    $ownerSid = $acl.GetOwner([Security.Principal.SecurityIdentifier]).Value
    if ($ownerSid -ne $currentSid) { throw 'Daemon credential directory owner was not the current user.' }
    $allowSids = @($acl.Access | Where-Object AccessControlType -eq 'Allow' | ForEach-Object { $_.IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value } | Select-Object -Unique)
    if ($allowSids.Count -ne 1 -or ($allowSids[0] -ne $currentSid -and $allowSids[0] -ne 'S-1-3-4')) { throw 'Daemon credential directory was not restricted to the current user.' }
    $cliStart = & $cargo run --quiet --package whalelink-cli -- start --room-id $roomId
    if ($LASTEXITCODE -ne 0 -or $cliStart -notmatch 'starting') { throw 'CLI tunnel start failed.' }
    Start-Sleep -Milliseconds 750
    $running = Send-Ipc 'health' @{}
    if (-not $running.ok -or $running.data.status -ne 'running') { throw 'Tunnel did not report running.' }
    $cliStop = & $cargo run --quiet --package whalelink-cli -- stop
    if ($LASTEXITCODE -ne 0 -or $cliStop -notmatch 'stopped') { throw 'CLI tunnel stop failed.' }
    Write-Output 'PASS: local Pipe enrollment import and CLI-controlled real EasyTier no-TUN start/stop succeeded.'
}
finally {
    if ($null -ne $process -and -not $process.HasExited) { Stop-Process -Id $process.Id -Force }
    Remove-Item -LiteralPath $config -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $credentialRoot -Recurse -Force -ErrorAction SilentlyContinue
    $env:LOCALAPPDATA = $previousLocalAppData
}
