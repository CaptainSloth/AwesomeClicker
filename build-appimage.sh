#!/usr/bin/env bash
set -euo pipefail

# ── colours ──────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

info()    { echo -e "${CYAN}[•]${RESET} $*"; }
success() { echo -e "${GREEN}[✓]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[!]${RESET} $*"; }
error()   { echo -e "${RED}[✗]${RESET} $*" >&2; exit 1; }
header()  { echo -e "\n${BOLD}$*${RESET}"; }

# ── config ────────────────────────────────────────────────────────────────────
BINARY_NAME="awesome-clicker"
APPDIR="build/AwesomeClicker.AppDir"
APPIMAGETOOL_URL="https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
APPIMAGETOOL="build/appimagetool"
ARCH=$(uname -m)
OUTPUT="AwesomeClicker-${ARCH}.AppImage"

# ── preflight ─────────────────────────────────────────────────────────────────
if [[ ! -f "Cargo.toml" ]]; then
    error "Run this script from the AwesomeClicker repo directory."
fi

echo -e "${BOLD}${CYAN}"
echo "  ╔════════════════════════════════════╗"
echo "  ║   AwesomeClicker AppImage Builder  ║"
echo "  ╚════════════════════════════════════╝"
echo -e "${RESET}"

# ── build release binary ──────────────────────────────────────────────────────
header "Building release binary"
[[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env"
command -v cargo &>/dev/null || error "cargo not found — run install.sh first or source \$HOME/.cargo/env"
cargo build --release
success "Binary built → target/release/${BINARY_NAME}"

# ── get appimagetool ──────────────────────────────────────────────────────────
header "Getting appimagetool"
mkdir -p build
if [[ ! -x "$APPIMAGETOOL" ]]; then
    info "Downloading appimagetool…"
    curl -L --progress-bar "$APPIMAGETOOL_URL" -o "$APPIMAGETOOL"
    chmod +x "$APPIMAGETOOL"
    success "Downloaded appimagetool"
else
    success "appimagetool already present"
fi

# ── assemble AppDir ───────────────────────────────────────────────────────────
header "Assembling AppDir"
rm -rf "$APPDIR"
mkdir -p "${APPDIR}/usr/bin"

# Binary
cp "target/release/${BINARY_NAME}" "${APPDIR}/usr/bin/${BINARY_NAME}"

# Icon (top-level required by AppImage spec)
if [[ -f "assets/icon.svg" ]]; then
    cp "assets/icon.svg" "${APPDIR}/${BINARY_NAME}.svg"
else
    warn "No icon found at assets/icon.svg — AppImage will have no icon"
fi

# Desktop file (top-level required by AppImage spec)
cat > "${APPDIR}/${BINARY_NAME}.desktop" <<EOF
[Desktop Entry]
Name=AwesomeClicker
Comment=Cross-platform auto clicker
Exec=${BINARY_NAME}
Icon=${BINARY_NAME}
Terminal=false
Type=Application
Categories=Utility;
Keywords=clicker;auto;mouse;
EOF

# AppRun entry point
cat > "${APPDIR}/AppRun" <<'EOF'
#!/usr/bin/env bash
SELF=$(readlink -f "$0")
HERE="${SELF%/*}"
exec "${HERE}/usr/bin/awesome-clicker" "$@"
EOF
chmod +x "${APPDIR}/AppRun"

success "AppDir assembled at ${APPDIR}"

# ── build AppImage ────────────────────────────────────────────────────────────
header "Building AppImage"

# appimagetool itself is an AppImage — needs FUSE or extraction
if "$APPIMAGETOOL" --version &>/dev/null 2>&1; then
    "$APPIMAGETOOL" "$APPDIR" "$OUTPUT"
else
    # Fallback: extract and run without FUSE (common in containers/CI)
    info "Extracting appimagetool (no FUSE available)…"
    "$APPIMAGETOOL" --appimage-extract &>/dev/null
    ./squashfs-root/AppRun "$APPDIR" "$OUTPUT"
    rm -rf squashfs-root
fi

chmod +x "$OUTPUT"
success "AppImage created → ${OUTPUT}"

echo ""
echo -e "${GREEN}${BOLD}Done!${RESET}"
echo -e "  To run:             ${BOLD}./${OUTPUT}${RESET}"
echo -e "  To install for all: copy it anywhere and double-click"
echo -e "  ${YELLOW}Tip:${RESET} if double-click doesn't work, right-click → Properties → Allow executing as program"
