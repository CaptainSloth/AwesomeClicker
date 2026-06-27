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
INSTALL_DIR="/usr/local/bin"
ICON_NAME="awesome-clicker"
ICON_SRC="assets/icon.svg"
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"
DESKTOP_DIR="$HOME/.local/share/applications"
DESKTOP_FILE="$DESKTOP_DIR/awesome-clicker.desktop"

# ── detect package manager ────────────────────────────────────────────────────
detect_pkg_manager() {
    if   command -v apt-get &>/dev/null; then echo "apt"
    elif command -v dnf     &>/dev/null; then echo "dnf"
    elif command -v pacman  &>/dev/null; then echo "pacman"
    elif command -v zypper  &>/dev/null; then echo "zypper"
    else echo "unknown"; fi
}

install_system_deps() {
    local pm; pm=$(detect_pkg_manager)
    header "Installing system dependencies (${pm})"

    case "$pm" in
        apt)
            sudo apt-get update -qq
            sudo apt-get install -y libx11-dev libxtst-dev libxdo-dev pkg-config
            ;;
        dnf)
            sudo dnf install -y libX11-devel libXtst-devel xdotool-devel pkgconf-pkg-config
            ;;
        pacman)
            sudo pacman -Sy --noconfirm libx11 libxtst xdotool pkgconf
            ;;
        zypper)
            sudo zypper install -y libX11-devel libXtst-devel xdotool-devel pkg-config
            ;;
        *)
            warn "Unknown package manager — skipping system deps."
            warn "Make sure these are installed: libx11-dev libxtst-dev libxdo-dev pkg-config"
            ;;
    esac
    success "System dependencies installed"
}

# ── Rust / cargo ──────────────────────────────────────────────────────────────
install_rust() {
    header "Setting up Rust"
    if command -v cargo &>/dev/null; then
        success "cargo already in PATH ($(cargo --version))"
        return
    fi

    # rustup already installed but not sourced?
    if [[ -f "$HOME/.cargo/env" ]]; then
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
        success "Sourced existing Rust install ($(cargo --version))"
        return
    fi

    info "Installing Rust via rustup…"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
    success "Rust installed ($(cargo --version))"

    # Persist in shell config
    for rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
        if [[ -f "$rc" ]] && ! grep -q 'cargo/env' "$rc"; then
            echo 'source "$HOME/.cargo/env"' >> "$rc"
            info "Added cargo to PATH in ${rc}"
        fi
    done
}

# ── build ─────────────────────────────────────────────────────────────────────
build_release() {
    header "Building release binary"
    [[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env"
    cargo build --release
    success "Build complete → target/release/${BINARY_NAME}"
}

# ── install binary ────────────────────────────────────────────────────────────
install_binary() {
    header "Installing binary to ${INSTALL_DIR}"
    sudo install -Dm755 "target/release/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    success "Installed to ${INSTALL_DIR}/${BINARY_NAME}"
}

# ── install icon ──────────────────────────────────────────────────────────────
install_icon() {
    header "Installing icon"
    if [[ ! -f "$ICON_SRC" ]]; then
        warn "Icon not found at ${ICON_SRC} — skipping"
        return
    fi
    mkdir -p "$ICON_DIR"
    cp "$ICON_SRC" "${ICON_DIR}/${ICON_NAME}.svg"
    # Refresh icon cache if gtk-update-icon-cache is available
    command -v gtk-update-icon-cache &>/dev/null \
        && gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
    success "Icon installed to ${ICON_DIR}/${ICON_NAME}.svg"
}

# ── desktop entry ─────────────────────────────────────────────────────────────
install_desktop_entry() {
    header "Creating desktop entry"
    mkdir -p "$DESKTOP_DIR"
    cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Name=AwesomeClicker
Comment=Cross-platform auto clicker
Exec=${INSTALL_DIR}/${BINARY_NAME}
Icon=${ICON_NAME}
Terminal=false
Type=Application
Categories=Utility;
Keywords=clicker;auto;mouse;
StartupWMClass=awesome-clicker
EOF
    command -v update-desktop-database &>/dev/null \
        && update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
    success "Desktop entry created — app will appear in your launcher"
}

# ── main ──────────────────────────────────────────────────────────────────────
echo -e "${BOLD}${CYAN}"
echo "  ╔═══════════════════════════════╗"
echo "  ║     AwesomeClicker Installer  ║"
echo "  ╚═══════════════════════════════╝"
echo -e "${RESET}"

if [[ ! -f "Cargo.toml" ]]; then
    error "Run this script from the AwesomeClicker repo directory."
fi

install_system_deps
install_rust
build_release
install_binary
install_icon
install_desktop_entry

echo ""
echo -e "${GREEN}${BOLD}All done!${RESET}"
echo -e "  • Run from terminal:   ${BOLD}awesome-clicker${RESET}"
echo -e "  • From launcher:       press ${BOLD}Super${RESET}, search \"AwesomeClicker\""
echo -e "  • Pin to dock:         right-click the icon → Add to Favorites"
