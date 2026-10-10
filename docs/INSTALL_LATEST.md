# Install the newest VeekPanel preview

Quit any running copy of VeekPanel before installing or opening this version.

## Nobara 44 (this computer)

Open `VeekPanel-0.1.1-0.preview.1791605701.g7b8ecbfe.nobara44.x86_64.rpm`
with **DNF App Center**, click **Install**, and enter your administrator password
when requested. Then open **VeekPanel** from the application menu. Replug the
PCPanel after installation. Run the app normally, never as root.

## Windows 10/11 (your friend)

Run `VeekPanel_0.1.1_x64-setup.exe` and follow the installer. It installs for your
user account and downloads WebView2 if needed. Existing settings are preserved.
This is an unsigned preview, so Windows may show an unknown-publisher warning.
Do not disable Windows security globally.

Both installers include quick audio assignments and the optional online Doom tab.
Source: a4d0ab8 application; RPM packaging 7b8ecbfe. Windows native installer checks
and all four core/desktop CI jobs passed. Nobara package/private GUI checks passed.
Consumer Windows/audio, physical Mini lifecycle and long-term reliability still
need acceptance. The full original feature list is not complete.

The older `VeekPanel-Nobara-x86_64.AppImage` is archived on GitHub with its mandatory
source companion; it does NOT include the newest quick assignments or Doom.
Use the RPM for the newest Nobara app.

Before uninstalling the Linux package, disable login startup in Settings and quit
VeekPanel. Package removal preserves your settings. No startup entry is enabled
by installation itself.

## Downloads

[GitHub draft with both newest installers and installation files](https://github.com/apittman019-svg/VeekPanel/releases/tag/untagged-1423dee8b4c29eb74fe0) (owner access; not publicly published).

[Passing Windows installer CI and download artifact](https://github.com/apittman019-svg/VeekPanel/actions/runs/38001562190) (GitHub sign-in required to download the artifact).

Local installers and this guide are together in `/home/austinp/Downloads/gaem/VeekPanel-Latest`.
