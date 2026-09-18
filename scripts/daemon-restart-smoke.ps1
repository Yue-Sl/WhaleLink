$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$batch = Join-Path $env:TEMP ("whalelink-restart-" + [guid]::NewGuid().ToString('N') + '.cmd')
$config = Join-Path $env:TEMP ("whalelink-restart-" + [guid]::NewGuid().ToString('N') + '.toml')
$credentialRoot = Join-Path $env:TEMP ("whalelink-restart-credentials-" + [guid]::NewGuid().ToString('N'))
$roomId = 'restart-smoke-room-' + [guid]::NewGuid().ToString('N').Substring(0, 8)
$previousLocalAppData = $env:LOCALAPPDATA
$env:LOCALAPPDATA = $credentialRoot

# This test-only child ignores its arguments, runs for two seconds without
# creating a TUN interface, then fails. The following health IPC request must
# observe the exit and start a replacement process.
"@echo off`r`ntimeout /t 2 /nobreak >nul`r`nexit /b 1`r`n" | Set-Content -LiteralPath $batch -NoNewline
$escapedBatch = $batch.Replace('\', '/')
@"
schema_version = 1
[easytier]
executable = "$escapedBatch"
arguments = []
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

function Wait-For-IpcHealth {
    for ($attempt = 0; $attempt -lt 30; $attempt++) {
        try {
            $health = Send-Ipc 'health' @{}
            if ($health.ok) { return $health }
        }
        catch { Start-Sleep -Milliseconds 500 }
    }
    throw 'Daemon Pipe did not become available before the startup deadline.'
}

try {
    $initial = Wait-For-IpcHealth
    if (-not $initial.ok -or $initial.data.status -ne 'stopped') { throw 'Daemon did not start in a stopped state.' }
    $enrollment = @{ network_name = 'restart-smoke-network'; network_secret = ('r' * 32); peers = @(); dhcp = $false; disable_upnp = $true }
    $imported = Send-Ipc 'enrollment.import' @{ room_id = $roomId; enrollment = $enrollment }
    if (-not $imported.ok) { throw 'Could not import restart-smoke enrollment.' }
    $started = Send-Ipc 'tunnel.start' @{ room_id = $roomId }
    if (-not $started.ok) { throw 'Could not start restart-smoke child.' }
    Start-Sleep -Milliseconds 2250
    $recovered = Send-Ipc 'health' @{}
    if (-not $recovered.ok -or $recovered.data.status -ne 'running') { throw 'Daemon did not restart the exited child.' }
    $stopped = Send-Ipc 'tunnel.stop' @{}
    if (-not $stopped.ok) { throw 'Daemon could not stop the restarted child.' }
    Write-Output 'PASS: daemon observed a child exit, restarted the selected room, and stopped the replacement without TUN.'
}
finally {
    if ($null -ne $process -and -not $process.HasExited) { Stop-Process -Id $process.Id -Force }
    Remove-Item -LiteralPath $batch -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $config -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $credentialRoot -Recurse -Force -ErrorAction SilentlyContinue
    $env:LOCALAPPDATA = $previousLocalAppData
}
