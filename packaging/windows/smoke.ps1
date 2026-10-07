param([string]$PreviousInstaller)
$ErrorActionPreference = 'Stop'
$installer = Get-ChildItem app/target/release/bundle/nsis/*-setup.exe | Select-Object -First 1
if (!$installer) { throw 'Installer missing' }
$installDir = Join-Path $env:RUNNER_TEMP 'VeekPanelInstall'
$exe = Join-Path $installDir 'veekpanel.exe'
$config = Join-Path $env:APPDATA 'org.veekpanel.desktop/config.json'
$version = (Get-Content app/tauri.conf.json -Raw | ConvertFrom-Json).version
Set-Service AudioEndpointBuilder -StartupType Manual
Set-Service Audiosrv -StartupType Manual
Start-Service AudioEndpointBuilder
Start-Service Audiosrv

function Install-Package([string]$path) {
    $p = Start-Process $path -ArgumentList "/S /D=$installDir" -Wait -PassThru
    if ($p.ExitCode -ne 0) { throw "Install failed: $($p.ExitCode)" }
    if (!(Test-Path $exe)) { throw 'Installed application missing' }
    foreach ($notice in @('rust','frontend')) {
        if (!(Test-Path (Join-Path $installDir "THIRD_PARTY/$notice/README.txt"))) { throw "$notice notices missing" }
    }
}

function Open-AndCloseApp([int]$schema) {
    $app = Start-Process $exe -PassThru
    try {
        $ready = $false
        # Windows exposes a window before WebView2 and Tauri setup finish.
        # Wait for saved defaults/migration and the running configuration owner.
        for ($i=0; $i -lt 60; $i++) {
            Start-Sleep -Milliseconds 500
            $app.Refresh()
            if ($app.HasExited) { throw "Installed app exited early: $($app.ExitCode)" }
            $savedSchema = $null
            if (Test-Path $config) { $savedSchema = (Get-Content $config -Raw | ConvertFrom-Json).schema_version }
            $owned = $false
            $lockPath = [IO.Path]::ChangeExtension($config, 'lock')
            if (Test-Path $lockPath) {
                try {
                    $lock = [IO.File]::Open($lockPath, [IO.FileMode]::Open, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
                    $lock.Dispose()
                } catch [IO.IOException] { $owned = $true }
            }
            if ($app.MainWindowHandle -ne 0 -and $app.Responding -and $savedSchema -eq $schema -and $owned) { $ready=$true; break }
        }
        if (!$ready) { throw "Installed app did not open with schema $schema and an active configuration owner within 30 seconds" }
    } finally {
        if (!$app.HasExited) {
            $null = $app.CloseMainWindow()
            if (!$app.WaitForExit(5000)) { Stop-Process -Id $app.Id -Force }
        }
    }
}

function Assert-ReinstallAndUninstallPreserveSettings {
    $before = (Get-FileHash $config).Hash
    Install-Package $installer.FullName
    if ((Get-FileHash $config).Hash -ne $before) { throw 'Reinstall modified configuration' }
    $p = Start-Process (Join-Path $installDir 'uninstall.exe') -ArgumentList '/S' -Wait -PassThru
    if ($p.ExitCode -ne 0) { throw 'Uninstall failed' }
    for ($i=0; $i -lt 30 -and (Test-Path $exe); $i++) { Start-Sleep -Milliseconds 500 }
    if (Test-Path $exe) { throw 'Application remained after uninstall' }
    if ((Get-FileHash $config).Hash -ne $before) { throw 'Uninstall changed saved configuration' }
}

# Disposable CI only: never erase a developer's existing configuration.
if (Test-Path $config) { throw 'Smoke test requires a fresh Windows user configuration' }
Install-Package $installer.FullName
Open-AndCloseApp 2
Assert-ReinstallAndUninstallPreserveSettings
Write-Output 'PASS: clean install, bundled notices, responsive native window, schema-2 config creation, reinstall and uninstall preserving settings.'

if ($PreviousInstaller) {
    # A representative schema-1 document is loaded by the published old app
    # before upgrading. Hardware stays disabled; the target is synthetic.
    $legacy = @{
        schema_version = 1; active_profile = 'gaming'
        profiles = @(
            @{ id='gaming'; name='Gaming'; mappings=@(
                @{ control=@{device='primary';kind='analog';index=0}; action=@{type='volume';target=@{type='group';id='mix'}} }
            ) },
            @{ id='work'; name='Work'; mappings=@() }
        )
        groups = @(@{id='mix';name='Saved mix';members=@(@{type='preferred_output'});relative=$true})
        preferences = @{ output=@{type='match';kind='output';identities=@{'endpoint.id'='veek-upgrade-synthetic-output'}}; input=$null }
        hardware = @{mode='disabled';model='mini';address=''}
        settings = @{theme='dark';close_to_tray=$false}
    }
    $legacy | ConvertTo-Json -Depth 20 | Set-Content -Encoding utf8NoBOM $config
    $originalHash = (Get-FileHash $config).Hash
    Install-Package $PreviousInstaller
    Open-AndCloseApp 1
    if ((Get-FileHash $config).Hash -ne $originalHash) { throw 'Baseline application changed the legacy fixture' }
    Install-Package $installer.FullName
    if ((Get-Item $exe).VersionInfo.ProductVersion -ne $version) { throw 'Upgrade did not replace the installed binary' }
    if ((Get-FileHash $config).Hash -ne $originalHash) { throw 'Installer changed config before app migration' }
    Open-AndCloseApp 2
    if ((Get-FileHash "$config.bak").Hash -ne $originalHash) { throw 'Migration did not preserve exact legacy backup' }
    $migrated = Get-Content $config -Raw | ConvertFrom-Json
    if ($migrated.active_profile -ne 'gaming' -or $migrated.profiles.Count -ne 2) { throw 'Migration changed profiles' }
    if ($migrated.PSObject.Properties.Name -contains 'groups' -or $migrated.PSObject.Properties.Name -contains 'preferences') { throw 'Shared schema-1 fields remain' }
    foreach ($profile in $migrated.profiles) {
        if ($profile.groups.Count -ne 1 -or $profile.groups[0].id -ne 'mix' -or !$profile.groups[0].relative) { throw 'Migration lost a group' }
        if ($profile.preferences.output.identities.'endpoint.id' -ne 'veek-upgrade-synthetic-output') { throw 'Migration lost a device preference' }
    }
    if ($migrated.profiles[0].mappings[0].action.target.id -ne 'mix' -or $migrated.settings.theme -ne 'dark' -or $migrated.hardware.mode -ne 'disabled') { throw 'Migration changed saved intent' }
    $migratedHash = (Get-FileHash $config).Hash
    Open-AndCloseApp 2
    if ((Get-FileHash $config).Hash -ne $migratedHash) { throw 'Relaunch changed migrated configuration' }
    Assert-ReinstallAndUninstallPreserveSettings
    Write-Output 'PASS: published 0.1.0 -> current installer upgrade, schema-1 migration into both profiles, original backup, relaunch and uninstall preservation.'
}
Write-Output 'No physical PCPanel or Windows audio-write acceptance is implied.'
