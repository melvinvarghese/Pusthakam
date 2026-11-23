$ErrorActionPreference = 'Stop'
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$sourceLogo = Join-Path $repositoryRoot 'branding/pusthakam-logo.png'
$temporaryIcons = Join-Path ([System.IO.Path]::GetTempPath()) ('pusthakam-brand-icons-' + [guid]::NewGuid().ToString('N'))

Push-Location $repositoryRoot
try {
    & npm run tauri -- icon $sourceLogo --fit contain --output $temporaryIcons
    if ($LASTEXITCODE -ne 0) { throw 'Icon generation failed.' }

    # Keep desktop outputs; mobile artwork stays in the temporary directory.
    $desktopIcons = @(
        '32x32.png', '64x64.png', '128x128.png', '128x128@2x.png',
        'icon.png', 'icon.ico', 'icon.icns', 'StoreLogo.png',
        'Square30x30Logo.png', 'Square44x44Logo.png', 'Square71x71Logo.png',
        'Square89x89Logo.png', 'Square107x107Logo.png', 'Square142x142Logo.png',
        'Square150x150Logo.png', 'Square284x284Logo.png', 'Square310x310Logo.png'
    )
    foreach ($name in $desktopIcons) {
        Copy-Item -LiteralPath (Join-Path $temporaryIcons $name) -Destination (Join-Path $repositoryRoot "src-tauri/icons/$name")
    }

    foreach ($relativeDirectory in @('src/assets', 'public', 'website/assets')) {
        New-Item -ItemType Directory -Force -Path (Join-Path $repositoryRoot $relativeDirectory) | Out-Null
    }
    foreach ($relativePath in @('src/assets/pusthakam-logo.png', 'website/assets/pusthakam-logo.png')) {
        Copy-Item -LiteralPath (Join-Path $temporaryIcons '128x128@2x.png') -Destination (Join-Path $repositoryRoot $relativePath)
    }
    foreach ($relativePath in @('public/favicon.png', 'website/assets/favicon.png')) {
        Copy-Item -LiteralPath (Join-Path $temporaryIcons '32x32.png') -Destination (Join-Path $repositoryRoot $relativePath)
    }
    Copy-Item -LiteralPath (Join-Path $temporaryIcons 'icon.ico') -Destination (Join-Path $repositoryRoot 'website/assets/favicon.ico')
    Write-Output 'Updated desktop, app and website icons from branding/pusthakam-logo.png.'
}
finally {
    Pop-Location
}
