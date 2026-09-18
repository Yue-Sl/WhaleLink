$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$assemblyPath = Join-Path $root 'apps\WhaleLink.Desktop\bin\Release\net8.0-windows\WhaleLink.Desktop.dll'
if (-not (Test-Path -LiteralPath $assemblyPath)) {
    throw 'Desktop Release assembly is missing. Run dotnet build first.'
}

$assembly = [Reflection.Assembly]::LoadFrom($assemblyPath)
$parser = $assembly.GetType('WhaleLink.Desktop.InvitePayloadParser', $true)
$method = $parser.GetMethod('Parse', [Reflection.BindingFlags]::Public -bor [Reflection.BindingFlags]::Static)

$fromQr = $method.Invoke($null, @('whalelink://invite?server=https%3A%2F%2Fcontrol.example&code=wl_test_one_time'))
if ($fromQr.Server.AbsoluteUri.TrimEnd('/') -ne 'https://control.example' -or $fromQr.Code -ne 'wl_test_one_time') {
    throw 'HTTPS QR payload parse contract failed.'
}
$fromCode = $method.Invoke($null, @('wl_direct_one_time'))
if ($null -ne $fromCode.Server -or $fromCode.Code -ne 'wl_direct_one_time') {
    throw 'Raw invite parse contract failed.'
}
foreach ($payload in @(
    'whalelink://invite?server=https%3A%2F%2Fcontrol.example',
    'whalelink://invite?server=http%3A%2F%2Fcontrol.example&code=wl_test_one_time'
)) {
    $rejected = $false
    try { $method.Invoke($null, @($payload)) | Out-Null }
    catch [ArgumentException] { $rejected = $true }
    catch [Reflection.TargetInvocationException] {
        if ($_.Exception.InnerException -is [ArgumentException]) { $rejected = $true }
        else { throw }
    }
    if (-not $rejected) { throw 'Invalid QR payload unexpectedly accepted.' }
}
Write-Output 'PASS: desktop invite payload contract accepted HTTPS/raw input and rejected incomplete or HTTP QR payloads.'
