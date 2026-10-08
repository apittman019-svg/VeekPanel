# Test-only CDP connection to the actual installed WebView2, no new dependencies
# or production IPC hooks. Loaded by smoke.ps1 on a disposable hosted runner.
# https://learn.microsoft.com/en-us/microsoft-edge/webview2/how-to/debug-visual-studio-code
Add-Type -TypeDefinition @'
using System;
using System.IO;
using System.Net.WebSockets;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
public static class VeekSmokeWindow {
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    private delegate bool EnumProc(IntPtr window, IntPtr data);
    [DllImport("user32.dll")] private static extern bool EnumWindows(EnumProc proc, IntPtr data);
    [DllImport("user32.dll")] private static extern uint GetWindowThreadProcessId(IntPtr window, out uint pid);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] private static extern int GetWindowText(IntPtr window, StringBuilder text, int length);
    public static IntPtr Find(int pid) {
        IntPtr result = IntPtr.Zero;
        EnumWindows((window, data) => {
            uint owner; GetWindowThreadProcessId(window, out owner);
            var title = new StringBuilder(256); GetWindowText(window, title, title.Capacity);
            if (owner == pid && title.ToString() == "VeekPanel") { result = window; return false; }
            return true;
        }, IntPtr.Zero);
        return result;
    }
}
public sealed class VeekSmokeCdp : IDisposable {
    private readonly ClientWebSocket socket = new ClientWebSocket();
    public VeekSmokeCdp(string uri) {
        var endpoint = new Uri(uri);
        if (!endpoint.IsLoopback || endpoint.Scheme != "ws")
            throw new ArgumentException("Only loopback WebView2 debugging is allowed");
        using (var timeout = new CancellationTokenSource(15000))
            socket.ConnectAsync(endpoint, timeout.Token).GetAwaiter().GetResult();
    }
    public string Exchange(string request) {
        using (var timeout = new CancellationTokenSource(15000)) {
            byte[] bytes = Encoding.UTF8.GetBytes(request);
            socket.SendAsync(new ArraySegment<byte>(bytes), WebSocketMessageType.Text, true,
                timeout.Token).GetAwaiter().GetResult();
            using (var message = new MemoryStream()) {
                var buffer = new byte[16384];
                WebSocketReceiveResult received;
                do {
                    received = socket.ReceiveAsync(new ArraySegment<byte>(buffer),
                        timeout.Token).GetAwaiter().GetResult();
                    if (received.MessageType != WebSocketMessageType.Text)
                        throw new IOException("WebView2 CDP closed or returned a non-text message");
                    message.Write(buffer, 0, received.Count);
                    if (message.Length > 4194304) throw new IOException("CDP response too large");
                } while (!received.EndOfMessage);
                return Encoding.UTF8.GetString(message.ToArray());
            }
        }
    }
    public void Dispose() { socket.Dispose(); }
}
'@

function Connect-InstalledWebView([int]$port) {
    for ($i = 0; $i -lt 60; $i++) {
        try {
            $targets = Invoke-RestMethod "http://127.0.0.1:$port/json/list" -TimeoutSec 2
            $target = @($targets | Where-Object { $_.type -eq 'page' -and $_.url -match '^(https?://tauri\.localhost|tauri://localhost)(/|$)' })
            if ($target.Count -eq 1 -and $target[0].webSocketDebuggerUrl) {
                return [VeekSmokeCdp]::new($target[0].webSocketDebuggerUrl)
            }
        } catch { $lastError = $_.Exception.Message }
        Start-Sleep -Milliseconds 500
    }
    throw "Installed WebView2 CDP page was not available: $lastError"
}

function Invoke-InstalledScript($cdp, [string]$expression) {
    # Runtime/Page events are not enabled, so one command has one response.
    $request = @{id=1;method='Runtime.evaluate';params=@{expression=$expression;awaitPromise=$true;returnByValue=$true}}
    $response = $cdp.Exchange(($request | ConvertTo-Json -Depth 20 -Compress)) | ConvertFrom-Json
    if ($response.id -ne 1 -or $response.error -or $response.result.exceptionDetails) {
        throw "Installed UI script failed: $($response | ConvertTo-Json -Depth 20 -Compress)"
    }
    return $response.result.result.value
}

function Assert-InstalledFrontend($cdp) {
    for ($i = 0; $i -lt 60; $i++) {
        if (Invoke-InstalledScript $cdp "document.readyState === 'complete' && document.querySelector('h1')?.textContent === 'Dashboard' && !!document.querySelector('header select') && !document.querySelector('header select').disabled") { break }
        if ($i -eq 59) { throw 'Bundled frontend did not initialize with native state' }
        Start-Sleep -Milliseconds 500
    }
    $result = Invoke-InstalledScript $cdp "(async()=>{const s=await window.__TAURI_INTERNALS__.invoke('snapshot',{knownVersion:null});return {schema:s.state.config.schema_version,hardware:s.state.config.hardware.mode,profile:s.state.config.active_profile,startup:s.desktop.startup_registered,supported:s.desktop.startup_supported,backend:s.state.audio?.backend??null,ua:navigator.userAgent};})()"
    if ($result.schema -ne 2 -or !$result.supported -or $result.startup -ne $false) { throw 'Current installed native snapshot/initial startup state is invalid' }
    if ($result.hardware -ne 'disabled') { throw 'Hosted smoke must never open physical hardware' }
    Write-Output "PASS: real installed WebView2 rendered Dashboard and returned native schema-2 state; $($result.ua)"
}

function Assert-StartupCommands($cdp) {
    $hash = (Get-FileHash $config).Hash
    foreach ($enabled in @($false, $true, $true, $false, $false)) {
        $value = if ($enabled) { 'true' } else { 'false' }
        $observed = Invoke-InstalledScript $cdp "(async()=>{await window.__TAURI_INTERNALS__.invoke('set_login_startup',{enabled:$value});const s=await window.__TAURI_INTERNALS__.invoke('snapshot',{knownVersion:null});return {registered:s.desktop.startup_registered,error:s.desktop.startup_error};})()"
        if ($observed.registered -ne $enabled -or $observed.error) { throw 'Application startup readback failed' }
        $expected = if ($enabled) { $ownedStartup } else { $null }
        if ((Read-StartupValue) -cne $expected) { throw 'Actual HKCU startup value differs from application readback' }
        if ((Get-FileHash $config).Hash -ne $hash) { throw 'Machine-local startup command rewrote exported configuration' }
    }
    Write-Output 'PASS: real native startup command enables/detects/removes HKCU opt-in idempotently without rewriting configuration.'
}

function Set-InstalledBackground($cdp, [bool]$minimized, [bool]$closeToTray) {
    $m = if ($minimized) { 'true' } else { 'false' }
    $c = if ($closeToTray) { 'true' } else { 'false' }
    $null = Invoke-InstalledScript $cdp "(async()=>{const i=window.__TAURI_INTERNALS__.invoke;const s=await i('snapshot',{knownVersion:null});s.state.config.settings.start_minimized=$m;s.state.config.settings.close_to_tray=$c;await i('action',{request:{type:'save',revision:s.state.revision,config:s.state.config}});return true;})()"
    $saved = Get-Content $config -Raw | ConvertFrom-Json
    if ($saved.settings.start_minimized -ne $minimized -or $saved.settings.close_to_tray -ne $closeToTray) { throw 'Native background settings did not persist' }
}
