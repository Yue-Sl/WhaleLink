$ErrorActionPreference = 'Stop'
$serverPort = 18787
$statePath = Join-Path $env:TEMP ("whalelink-smoke-" + [guid]::NewGuid().ToString() + ".json")
$configPath = Join-Path $env:TEMP ("whalelink-smoke-" + [guid]::NewGuid().ToString() + ".toml")
$previousToken = $env:WHALELINK_ADMIN_TOKEN
$previousBind = $env:WHALELINK_BIND
$previousState = $env:WHALELINK_STATE_PATH
$previousConfig = $env:WHALELINK_CONFIG
$previousRoomSecret = $env:WHALELINK_SMOKE_ROOM_SECRET
$env:WHALELINK_ADMIN_TOKEN = 'smoke-test-token-not-for-production'
$env:WHALELINK_BIND = "127.0.0.1:$serverPort"
$env:WHALELINK_STATE_PATH = $statePath
$env:WHALELINK_CONFIG = $configPath
$env:WHALELINK_SMOKE_ROOM_SECRET = 'smoke-test-secret-0123456789abcdef0123456789abcdef'
@'
[[rooms]]
id = "default"
display_name = "Default room"
[rooms.data_plane]
network_name = "whalelink-smoke"
network_secret_env = "WHALELINK_SMOKE_ROOM_SECRET"
peers = ["tcp://127.0.0.1:19000"]
'@ | Set-Content -LiteralPath $configPath -NoNewline
$cargoPath = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
function Start-Server {
    Start-Process -FilePath $cargoPath -ArgumentList 'run --quiet --package whalelink-server' -WorkingDirectory $PSScriptRoot\.. -WindowStyle Hidden -PassThru
}

function Wait-ForHealth {
    for ($attempt = 0; $attempt -lt 20; $attempt++) {
        Start-Sleep -Milliseconds 250
        try {
            $health = Invoke-RestMethod "http://127.0.0.1:$serverPort/api/v1/health"
            if ($health.ok) { return }
        } catch { }
    }
    throw 'Server health endpoint did not become available.'
}

$process = Start-Server
try {
    Wait-ForHealth
    $headers = @{ 'X-WhaleLink-Token' = $env:WHALELINK_ADMIN_TOKEN }
    $rooms = Invoke-RestMethod "http://127.0.0.1:$serverPort/api/v1/rooms" -Headers $headers
    if (-not $rooms.ok -or $rooms.data.Count -lt 1) { throw 'Authenticated room listing failed.' }
    $expiresAt = [DateTime]::UtcNow.AddMinutes(5).ToString('o')
    $invite = Invoke-RestMethod "http://127.0.0.1:$serverPort/api/v1/rooms/default/invites" -Method Post -Headers $headers -ContentType 'application/json' -Body (@{ expires_at = $expiresAt } | ConvertTo-Json)
    if (-not $invite.ok) { throw 'Invite creation failed.' }
    $revokedInvite = Invoke-RestMethod "http://127.0.0.1:$serverPort/api/v1/rooms/default/invites" -Method Post -Headers $headers -ContentType 'application/json' -Body (@{ expires_at = $expiresAt } | ConvertTo-Json)
    if (-not $revokedInvite.ok) { throw 'Second invite creation failed.' }
    $revoked = Invoke-RestMethod "http://127.0.0.1:$serverPort/api/v1/invites/$($revokedInvite.data.id)" -Method Delete -Headers $headers
    if (-not $revoked.ok) { throw 'Invite revocation failed.' }
    try {
        Invoke-RestMethod "http://127.0.0.1:$serverPort/api/v1/invites/$($revokedInvite.data.code)/redeem" -Method Post | Out-Null
        throw 'Revoked invite unexpectedly redeemed.'
    }
    catch {
        $statusCode = $_.Exception.Response.StatusCode.value__
        if ($statusCode -ne 410) { throw }
    }
    Stop-Process -Id $process.Id -Force
    $process = Start-Server
    Wait-ForHealth
    $redeemed = Invoke-RestMethod "http://127.0.0.1:$serverPort/api/v1/invites/$($invite.data.code)/redeem" -Method Post
    if (-not $redeemed.ok -or $redeemed.data.room_id -ne 'default' -or $redeemed.data.enrollment.data_plane.network_name -ne 'whalelink-smoke') { throw 'Persisted invite redemption failed.' }
    $members = Invoke-RestMethod "http://127.0.0.1:$serverPort/api/v1/rooms/default/members" -Headers $headers
    if (-not $members.ok -or $members.data.Count -ne 1) { throw 'Member state was not recorded.' }
    $rotated = Invoke-RestMethod "http://127.0.0.1:$serverPort/api/v1/rooms/default/credentials/rotate" -Method Post -Headers $headers
    if (-not $rotated.ok -or $rotated.data.credential_epoch -ne 2) { throw 'Credential epoch rotation failed.' }
    $roomsAfterRotation = Invoke-RestMethod "http://127.0.0.1:$serverPort/api/v1/rooms" -Headers $headers
    if (-not $roomsAfterRotation.ok -or $roomsAfterRotation.data[0].credential_epoch -ne 2) { throw 'Rotated credential epoch was not persisted.' }
    Write-Output 'PASS: health, authentication, invite persistence/revocation/redemption, member state, and credential epoch rotation succeeded.'
}
finally {
    if ($null -ne $process -and -not $process.HasExited) { Stop-Process -Id $process.Id -Force }
    Remove-Item -LiteralPath $statePath -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $configPath -Force -ErrorAction SilentlyContinue
    $env:WHALELINK_ADMIN_TOKEN = $previousToken
    $env:WHALELINK_BIND = $previousBind
    $env:WHALELINK_STATE_PATH = $previousState
    $env:WHALELINK_CONFIG = $previousConfig
    $env:WHALELINK_SMOKE_ROOM_SECRET = $previousRoomSecret
}
