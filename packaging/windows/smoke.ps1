$ErrorActionPreference = 'Stop'
$installer = Get-ChildItem app/target/release/bundle/nsis/*-setup.exe | Select-Object -First 1
if (!$installer) { throw 'Installer missing' }
$installDir = Join-Path $env:RUNNER_TEMP 'VeekPanelInstall'
$p = Start-Process $installer.FullName -ArgumentList "/S /D=$installDir" -Wait -PassThru
if ($p.ExitCode -ne 0) { throw "Install failed: $($p.ExitCode)" }
$exe = Join-Path $installDir 'veekpanel.exe'
if (!(Test-Path $exe)) { throw 'Installed application missing' }
if (!(Test-Path (Join-Path $installDir 'THIRD_PARTY/rust/README.txt'))) { throw 'Rust notices missing' }
if (!(Test-Path (Join-Path $installDir 'THIRD_PARTY/frontend/README.txt'))) { throw 'Frontend notices missing' }
Set-Service AudioEndpointBuilder -StartupType Manual
Set-Service Audiosrv -StartupType Manual
Start-Service AudioEndpointBuilder
Start-Service Audiosrv
$app = Start-Process $exe -PassThru
$config = Join-Path $env:APPDATA 'org.veekpanel.desktop/config.json'
try {
    $visible = $false
    # Windows can expose Tauri's window before WebView2 initialization and the
    # setup callback have finished. Wait for both the window and saved defaults.
    for ($i=0; $i -lt 60; $i++) {
        Start-Sleep -Milliseconds 500
        $app.Refresh()
        if ($app.HasExited) { throw "Installed app exited early: $($app.ExitCode)" }
        if ($app.MainWindowHandle -ne 0 -and $app.Responding) { $visible=$true }
        if ($visible -and (Test-Path $config)) { break }
    }
    if (!$visible) { throw 'Installed app did not open a responsive window' }
    if (!(Test-Path $config)) {
        Get-ChildItem (Split-Path $config) -ErrorAction SilentlyContinue | Select-Object Name,Length
        throw "Installed app did not initialize configuration within 30 seconds: $config"
    }
    $before = Get-FileHash $config
} finally {
    if (!$app.HasExited) {
        $null = $app.CloseMainWindow()
        if (!$app.WaitForExit(5000)) { Stop-Process -Id $app.Id -Force }
    }
}
# Same-version reinstall verifies replacement and configuration preservation.
$p = Start-Process $installer.FullName -ArgumentList "/S /D=$installDir" -Wait -PassThru
if ($p.ExitCode -ne 0) { throw 'Reinstall failed' }
if ((Get-FileHash $config).Hash -ne $before.Hash) { throw 'Reinstall modified configuration' }
$uninstaller = Join-Path $installDir 'uninstall.exe'
$p = Start-Process $uninstaller -ArgumentList '/S' -Wait -PassThru
if ($p.ExitCode -ne 0) { throw 'Uninstall failed' }
for ($i=0; $i -lt 30 -and (Test-Path $exe); $i++) { Start-Sleep -Milliseconds 500 }
if (Test-Path $exe) { throw 'Application remained after uninstall' }
if ((Get-FileHash $config).Hash -ne $before.Hash) { throw 'Uninstall changed saved configuration' }
Write-Output 'PASS: install, bundled notices, responsive native window, config creation, reinstall and uninstall with config preserved. No physical PCPanel/audio-write acceptance.'
