$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$assemblyPath = Join-Path $root 'apps\WhaleLink.Desktop\bin\Release\net8.0-windows\WhaleLink.Desktop.dll'
if (-not (Test-Path -LiteralPath $assemblyPath)) {
    throw 'Desktop Release assembly is missing. Run dotnet build first.'
}

$directory = Join-Path $env:LOCALAPPDATA ('whalelink-acl-contract-' + [guid]::NewGuid().ToString('N'))
try {
    $assembly = [Reflection.Assembly]::LoadFrom($assemblyPath)
    $type = $assembly.GetType('WhaleLink.Desktop.EnrollmentCredentialStore', $true)
    $constructor = $type.GetConstructor(
        [Reflection.BindingFlags]::Instance -bor [Reflection.BindingFlags]::Public -bor [Reflection.BindingFlags]::NonPublic,
        $null,
        [Type[]]@([string]),
        $null)
    $store = $constructor.Invoke([object[]]@([string]$directory))
    $save = $type.GetMethod('Save', [Reflection.BindingFlags]::Instance -bor [Reflection.BindingFlags]::Public)
    $save.Invoke($store, [object[]]@('acl-contract-room', '{"network_name":"test","network_secret":"not-persisted-in-clear","peers":[],"dhcp":false,"disable_upnp":true}')) | Out-Null

    $acl = Get-Acl -LiteralPath $directory
    if (-not $acl.AreAccessRulesProtected) { throw 'Credential directory ACL still inherits from its parent.' }
    $currentSid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
    $ownerSid = $acl.GetOwner([Security.Principal.SecurityIdentifier]).Value
    if ($ownerSid -ne $currentSid) { throw 'Credential directory owner was not the current user.' }
    $allowSids = @($acl.Access | Where-Object AccessControlType -eq 'Allow' | ForEach-Object { $_.IdentityReference.Translate([Security.Principal.SecurityIdentifier]).Value } | Select-Object -Unique)
    if ($allowSids.Count -ne 1 -or $allowSids[0] -ne $currentSid) { throw 'Credential directory was not restricted to the current user.' }
    if (@(Get-ChildItem -LiteralPath $directory -File).Count -ne 1) { throw 'Credential store did not create exactly one protected material file.' }
    Write-Output 'PASS: desktop DPAPI credential directory has a protected current-user-only ACL.'
}
finally {
    if (Test-Path -LiteralPath $directory) { Remove-Item -LiteralPath $directory -Recurse -Force }
}
