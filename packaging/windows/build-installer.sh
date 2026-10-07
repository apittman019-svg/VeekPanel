#!/usr/bin/env bash
# VeekPanel – Windows installer build script (Linux/Nobara)
# This script cross‑compiles the Tauri app for Windows, bundles it with NSIS, and produces a Windows installer.
# It is intended to be run from the repository root on a Linux machine.

set -euo pipefail

# ---------------------------------------------------------------------------
# Helper: locate repository root (directory containing this script's ../..)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

echo "=== VeekPanel Windows Installer Build (Cross‑Compilation) ==="

# ---------------------------------------------------------------------------
# 1. Verify required tools are present
for cmd in node pnpm cargo wine; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Error: Required tool '$cmd' is not installed or not in PATH."
    exit 1
  fi
done

# ---------------------------------------------------------------------------
# 2. Ensure cargo‑xwin is installed (cross‑compilation tool for MSVC)
if ! command -v cargo-xwin >/dev/null 2>&1; then
  echo "--> Installing cargo-xwin..."
  cargo install cargo-xwin --locked
fi

# ---------------------------------------------------------------------------
# 3. Ensure NSIS 3.11 toolset is available (used by Tauri under Wine)
NSIS_DIR="${HOME}/.local/share/nsis"
if [[ ! -f "${NSIS_DIR}/Bin/makensis.exe" ]] || [[ ! -f "${NSIS_DIR}/Include/Win/RestartManager.nsh" ]]; then
  echo "--> Downloading NSIS 3.11..."
  mkdir -p "${NSIS_DIR}"
  NSIS_ZIP="/tmp/nsis-3.11.zip"
  curl -fsSL "https://github.com/tauri-apps/binary-releases/releases/download/nsis-3.11/nsis-3.11.zip" -o "${NSIS_ZIP}"
  TMP_EXTRACT=$(mktemp -d)
  unzip -q "${NSIS_ZIP}" -d "${TMP_EXTRACT}"
  cp -r "${TMP_EXTRACT}/nsis-3.11"/* "${NSIS_DIR}/"
  rm -rf "${TMP_EXTRACT}" "${NSIS_ZIP}"
fi

# ---------------------------------------------------------------------------
# 4. Install the NSIS wrapper script (handles Windows‑style paths for Wine)
PLUGIN_DIR="${NSIS_DIR}/Plugins/x86-unicode/additional"
if [[ ! -f "${PLUGIN_DIR}/nsis_tauri_utils.dll" ]]; then
  echo "--> Fetching nsis_tauri_utils plugin..."
  mkdir -p "${PLUGIN_DIR}"
  curl -fsSL "https://github.com/tauri-apps/nsis-tauri-utils/releases/download/nsis_tauri_utils-v0.5.3/nsis_tauri_utils.dll" -o "${PLUGIN_DIR}/nsis_tauri_utils.dll"
fi
# Mirror to Tauri cache location (optional but kept for compatibility)
mkdir -p "${HOME}/.cache/tauri/NSIS/Plugins/x86-unicode/additional"
cp "${PLUGIN_DIR}/nsis_tauri_utils.dll" "${HOME}/.cache/tauri/NSIS/Plugins/x86-unicode/additional/"

# Create a thin wrapper around makensis that translates POSIX paths to Windows paths for Wine
mkdir -p "${HOME}/.local/bin"
cat << 'WRAPPER_EOF' > "${HOME}/.local/bin/makensis"
#!/usr/bin/env bash
set -e
PROCESSED_ARGS=()
for arg in "$@"; do
  if [[ "$arg" == *.nsi ]] && [[ -f "$arg" ]]; then
    # Rewrite absolute POSIX paths inside the .nsi file to Wine Z: drive paths
    python3 -c "
import sys, re
path = sys.argv[1]
with open(path, 'r', encoding='utf-8') as f:
    c = f.read()
def repl(m):
    p = m.group(1).replace('/', '\\\\\\\\')
    return '\"Z:' + p + '\"'
c = re.sub(r'\"(/[a-zA-Z][^\"]*)\"', repl, c)
with open(path, 'w', encoding='utf-8') as f:
    f.write(c)
" "$arg"
    # Also rewrite any .nsh files referenced alongside the .nsi
    nsi_dir="$(dirname "$arg")"
    for nsh in "$nsi_dir"/*.nsh; do
      [ -f "$nsh" ] || continue
      python3 -c "
import sys, re
path = sys.argv[1]
with open(path, 'r', encoding='utf-8') as f:
    c = f.read()
def repl(m):
    p = m.group(1).replace('/', '\\\\\\\\')
    return '\"Z:' + p + '\"'
c = re.sub(r'\"(/[a-zA-Z][^\"]*)\"', repl, c)
with open(path, 'w', encoding='utf-8') as f:
    f.write(c)
" "$nsh"
    done
    win_path=$(winepath -w "$arg" 2>/dev/null || echo "$arg")
    PROCESSED_ARGS+=("$win_path")
  elif [[ "$arg" == /* ]]; then
    win_path=$(winepath -w "$arg" 2>/dev/null || echo "$arg")
    PROCESSED_ARGS+=("$win_path")
  else
    PROCESSED_ARGS+=("$arg")
  fi
done
exec wine "${HOME}/.local/share/nsis/Bin/makensis.exe" "${PROCESSED_ARGS[@]}"
WRAPPER_EOF
chmod +x "${HOME}/.local/bin/makensis"
export PATH="${HOME}/.local/bin:${PATH}"

# ---------------------------------------------------------------------------
# 5. Build the UI frontend (Vite + Svelte)
echo "---> Building UI frontend..."
cd "${REPO_ROOT}"
# Install UI dependencies (pnpm will read ui/package.json automatically)
pnpm --dir ui install --frozen-lockfile
pnpm --dir ui build

# ---------------------------------------------------------------------------
# 6. Compile the Windows binary with cargo‑xwin
echo "---> Compiling Windows x86_64 MSVC binary..."
cargo xwin build --manifest-path app/Cargo.toml --target x86_64-pc-windows-msvc --release

# ---------------------------------------------------------------------------
# 7. Bundle the application with Tauri (produces NSIS installer)
echo "---> Bundling with Tauri (NSIS installer)..."
# Ensure the Tauri CLI is available – install it once if missing.
if ! command -v cargo-tauri >/dev/null 2>&1; then
  echo "---> Installing Tauri CLI via cargo..."
  cargo install tauri-cli --locked
fi
# Run bundling from the app directory where tauri.conf.json resides.
cd "${REPO_ROOT}/app"
cargo tauri bundle --target x86_64-pc-windows-msvc --bundles nsis

# ---------------------------------------------------------------------------
# 8. Verify installer output and report
INSTALLER_EXE="${REPO_ROOT}/app/target/x86_64-pc-windows-msvc/release/bundle/nsis/VeekPanel_0.1.0_x64-setup.exe"
if [[ -f "${INSTALLER_EXE}" ]]; then
  echo "\n=== Build Succeeded ==="
  echo "Installer created at: ${INSTALLER_EXE}"
  ls -lh "${INSTALLER_EXE}"
  sha256sum "${INSTALLER_EXE}"
else
  echo "Error: Installer not found at expected location: ${INSTALLER_EXE}" >&2
  exit 1
fi
