param([string]$PreviousInstaller)
$ErrorActionPreference = 'Stop'
$installer = Get-ChildItem app/target/release/bundle/nsis/*-setup.exe | Select-Object -First 1
if (!$installer) { throw 'Installer missing' }
$installDir = Join-Path $env:RUNNER_TEMP 'VeekPanel Install'
$exe = Join-Path $installDir 'veekpanel.exe'
$config = Join-Path $env:APPDATA 'org.veekpanel.desktop/config.json'
$version = (Get-Content app/tauri.conf.json -Raw | ConvertFrom-Json).version
$runKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$startupName = 'org.veekpanel.desktop'
$ownedStartup = '"' + $exe + '" --autostart'
$otherStartupName = 'VeekPanel-smoke-unrelated'
$otherStartup = '"C:\Unrelated Test\other.exe" --stay'

function Read-StartupValue([string]$name = $startupName) {
    if (!(Test-Path $runKey)) { return $null }
    return Get-ItemPropertyValue -Path $runKey -Name $name -ErrorAction SilentlyContinue
}

function Set-TestStartup([string]$command) {
    if (!(Test-Path $runKey)) { $null = New-Item -Path $runKey }
    $null = New-ItemProperty -Path $runKey -Name $startupName -Value $command -PropertyType String -Force
}
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

function Open-AndCloseApp([int]$schema, [bool]$Lifecycle = $true) {
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
        # Native second processes must hand off without rewriting config or exiting
        # the responsive owner. Registration must also remain untouched on launch.
        $configHash = (Get-FileHash $config).Hash
        $registration = Read-StartupValue
        $launches = if ($Lifecycle) { @('', '--autostart') } else { @() }
        foreach ($arguments in $launches) {
            if ($arguments) { $second = Start-Process $exe -ArgumentList $arguments -PassThru }
            else { $second = Start-Process $exe -PassThru }
            try {
                if (!$second.WaitForExit(10000)) { throw 'Second launch did not hand off within 10 seconds' }
                if ($second.ExitCode -ne 0) { throw "Second launch failed: $($second.ExitCode)" }
                $app.Refresh()
                if ($app.HasExited -or !$app.Responding) { throw 'Original owner stopped responding after handoff' }
                if ((Get-FileHash $config).Hash -ne $configHash) { throw 'Second launch rewrote configuration' }
                if ((Read-StartupValue) -cne $registration) { throw 'Launching changed startup registration' }
            } finally {
                if (!$second.HasExited) { Stop-Process -Id $second.Id -Force }
            }
        }
    } finally {
        if (!$app.HasExited) {
            $null = $app.CloseMainWindow()
            if (!$app.WaitForExit(5000)) {
                Stop-Process -Id $app.Id -Force
                $null = $app.WaitForExit(5000)
                if ($Lifecycle) { throw 'Window close did not shut down the configuration owner cleanly' }
            }
        }
    }
    # Shutdown must release the config lock; a forced kill cannot count as success.
    $lockPath = [IO.Path]::ChangeExtension($config, 'lock')
    $lock = [IO.File]::Open($lockPath, [IO.FileMode]::Open, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
    $lock.Dispose()
}

function Assert-ReinstallAndUninstallPreserveSettings {
    $before = (Get-FileHash $config).Hash
    Install-Package $installer.FullName
    if ((Get-FileHash $config).Hash -ne $before) { throw 'Reinstall modified configuration' }
    if ((Read-StartupValue) -cne $ownedStartup) { throw 'Reinstall changed the existing startup opt-in' }
    $null = New-ItemProperty -Path $runKey -Name $otherStartupName -Value $otherStartup -PropertyType String -Force
    $p = Start-Process (Join-Path $installDir 'uninstall.exe') -ArgumentList '/S' -Wait -PassThru
    if ($p.ExitCode -ne 0) { throw 'Uninstall failed' }
    for ($i=0; $i -lt 30 -and ((Test-Path $exe) -or $null -ne (Read-StartupValue)); $i++) { Start-Sleep -Milliseconds 500 }
    if (Test-Path $exe) { throw 'Application remained after uninstall' }
    if ((Get-FileHash $config).Hash -ne $before) { throw 'Uninstall changed saved configuration' }
    if ($null -ne (Read-StartupValue)) { throw 'Uninstall retained its own startup entry' }
    if ((Read-StartupValue $otherStartupName) -cne $otherStartup) { throw 'Uninstall changed an unrelated startup entry' }
    Remove-ItemProperty -Path $runKey -Name $otherStartupName
}

# Disposable CI only: never erase a developer's existing configuration.
if (Test-Path $config) { throw 'Smoke test requires a fresh Windows user configuration' }
if ($null -ne (Read-StartupValue) -or $null -ne (Read-StartupValue $otherStartupName)) { throw 'Smoke test refuses to replace existing startup entries' }
Install-Package $installer.FullName
if ($null -ne (Read-StartupValue)) { throw 'Install unexpectedly enabled startup' }
Open-AndCloseApp 2
Set-TestStartup $ownedStartup
Assert-ReinstallAndUninstallPreserveSettings
Write-Output 'PASS: install path with spaces, native handoff and clean shutdown, no implicit startup, reinstall preserving opt-in, uninstall removing only the owned entry and preserving settings.'

# Tauri's /UPDATE uninstall mode must preserve the opt-in for replacement.
Install-Package $installer.FullName
Set-TestStartup $ownedStartup
$beforeUpdate = (Get-FileHash $config).Hash
$p = Start-Process (Join-Path $installDir 'uninstall.exe') -ArgumentList '/S /UPDATE' -Wait -PassThru
if ($p.ExitCode -ne 0) { throw "Update-mode uninstall failed: $($p.ExitCode)" }
for ($i=0; $i -lt 30 -and (Test-Path $exe); $i++) { Start-Sleep -Milliseconds 500 }
if (Test-Path $exe) { throw 'Update-mode removal left the old binary' }
if ((Read-StartupValue) -cne $ownedStartup) { throw 'Update-mode removal deleted startup opt-in' }
Install-Package $installer.FullName
if ((Read-StartupValue) -cne $ownedStartup -or (Get-FileHash $config).Hash -ne $beforeUpdate) { throw 'Replacement changed startup opt-in or settings' }
Assert-ReinstallAndUninstallPreserveSettings
Write-Output 'PASS: update-mode removal/replacement preserves startup opt-in and settings.'

# An entry for another installation under the same value name must survive.
Install-Package $installer.FullName
$foreignStartup = '"C:\Another VeekPanel Install\veekpanel.exe" --autostart'
Set-TestStartup $foreignStartup
$p = Start-Process (Join-Path $installDir 'uninstall.exe') -ArgumentList '/S' -Wait -PassThru
if ($p.ExitCode -ne 0) { throw "Foreign-entry uninstall failed: $($p.ExitCode)" }
for ($i=0; $i -lt 30 -and (Test-Path $exe); $i++) { Start-Sleep -Milliseconds 500 }
if (Test-Path $exe) { throw 'Foreign-entry uninstall retained the binary' }
if ((Read-StartupValue) -cne $foreignStartup) { throw 'Uninstall deleted another installation startup entry' }
Remove-ItemProperty -Path $runKey -Name $startupName
Write-Output 'PASS: uninstall leaves a startup entry belonging to another install untouched.'

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
    # Published 0.1.0 predates single-instance handling; only check its baseline.
    Open-AndCloseApp 1 $false
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
    Set-TestStartup $ownedStartup
    Assert-ReinstallAndUninstallPreserveSettings
    Write-Output 'PASS: published 0.1.0 -> current installer upgrade, schema-1 migration into both profiles, original backup, relaunch and uninstall preservation.'
}
Write-Output 'No physical PCPanel or Windows audio-write acceptance is implied.'
