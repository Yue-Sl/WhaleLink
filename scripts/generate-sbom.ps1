param(
    [Parameter(Mandatory = $true)]
    [string]$OutputPath,
    [string]$PackageName = 'WhaleLink-v3'
)

$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$cargo = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
$metadata = & $cargo metadata --locked --offline --format-version 1 | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) { throw 'Cargo metadata generation failed.' }

$components = [System.Collections.Generic.List[object]]::new()
foreach ($package in $metadata.packages | Sort-Object name, version, source) {
    $components.Add([ordered]@{
        name = $package.name
        versionInfo = $package.version
        downloadLocation = if ($null -eq $package.source) { 'NOASSERTION' } else { $package.source }
        licenseConcluded = if ($package.name -like 'whalelink*') { 'Apache-2.0' } else { 'NOASSERTION' }
    })
}

$dotnetLock = Get-Content -Raw (Join-Path $root 'apps\WhaleLink.Desktop\packages.lock.json') | ConvertFrom-Json
$seenDotnet = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
foreach ($target in $dotnetLock.dependencies.psobject.Properties) {
    foreach ($dependency in $target.Value.psobject.Properties) {
        $value = $dependency.Value
        $key = "$($dependency.Name)|$($value.resolved)"
        if ($seenDotnet.Add($key)) {
            $components.Add([ordered]@{
                name = $dependency.Name
                versionInfo = $value.resolved
                downloadLocation = 'https://api.nuget.org/v3/index.json'
                licenseConcluded = 'NOASSERTION'
            })
        }
    }
}

$components.Add([ordered]@{
    name = 'EasyTier'
    versionInfo = '2.6.4'
    downloadLocation = 'https://github.com/EasyTier/EasyTier/releases/tag/v2.6.4'
    licenseConcluded = 'LGPL-3.0-only'
})

$packages = @()
$index = 1
foreach ($component in $components | Sort-Object name, versionInfo, downloadLocation) {
    $packages += [ordered]@{
        SPDXID = "SPDXRef-Package-$index"
        name = $component.name
        versionInfo = $component.versionInfo
        downloadLocation = $component.downloadLocation
        filesAnalyzed = $false
        licenseConcluded = $component.licenseConcluded
        licenseDeclared = $component.licenseConcluded
    }
    $index++
}

$document = [ordered]@{
    spdxVersion = 'SPDX-2.3'
    dataLicense = 'CC0-1.0'
    SPDXID = 'SPDXRef-DOCUMENT'
    name = $PackageName
    documentNamespace = "https://github.com/WhaleLink/whalelink/sbom/$PackageName/$([guid]::NewGuid())"
    creationInfo = [ordered]@{
        created = [DateTime]::UtcNow.ToString('o')
        creators = @('Tool: WhaleLink scripts/generate-sbom.ps1')
    }
    packages = $packages
}

$parent = Split-Path -Parent $OutputPath
New-Item -ItemType Directory -Force -Path $parent | Out-Null
$document | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $OutputPath -Encoding utf8
Write-Output "PASS: SPDX SBOM with $($packages.Count) packages written to $OutputPath"
