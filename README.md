# AwesomeClicker

A cross-platform auto clicker built with Rust and egui.

## Quick Install (Linux)

### Option 1 — App Launcher (recommended for Pop!_OS / GNOME desktops)

Run the installer from the repo directory:

```bash
./install.sh
```

This handles everything automatically:
1. Installs system libraries (`libx11-dev`, `libxtst-dev`, `libxdo-dev`, `pkg-config`)
2. Installs Rust via rustup if not already present and adds `cargo` to your shell PATH
3. Builds an optimised release binary
4. Installs it to `/usr/local/bin/awesome-clicker` (requires sudo)
5. Installs the app icon to your icon theme
6. Creates a `.desktop` entry so it appears in your application launcher

After installing:
- Press **Super**, search **AwesomeClicker**, click to launch
- Right-click the icon → **Add to Favorites** to pin it to the dock

Supports **apt** (Ubuntu/Debian/Pop!_OS), **dnf** (Fedora), **pacman** (Arch), and **zypper** (openSUSE).

---

### Option 2 — AppImage (portable double-clickable file)

Build a single self-contained `.AppImage` file — no installation required, works on any Linux distro:

```bash
./build-appimage.sh
```

This produces `AwesomeClicker-x86_64.AppImage` in the repo directory. To run it:

1. Right-click → **Properties** → **Allow executing as program** (or `chmod +x AwesomeClicker-x86_64.AppImage`)
2. Double-click to launch

The AppImage can be copied anywhere — a USB drive, shared with others, or placed in `~/Applications`.

---

## Manual Prerequisites

### Linux

Install Rust (if not already installed):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Then make `cargo` available in every future terminal:

```bash
echo 'source "$HOME/.cargo/env"' >> ~/.bashrc
source ~/.bashrc
```

Install system libraries:

```bash
sudo apt-get install -y libx11-dev libxtst-dev libxdo-dev pkg-config
```

### Windows

Install Rust from https://rustup.rs — no extra system libraries needed.

---

## Build & Run

**Run in development mode:**

```bash
cargo run
```

**Build an optimized release binary:**

```bash
cargo build --release
./target/release/awesome-clicker
```

The release binary is self-contained — copy it anywhere and run it directly.

---

## Usage

### Basic Tab

| Setting | Description |
|---|---|
| Speed | Clicks per second (0.1 – 50). Logarithmic slider. |
| Mouse Button | Left, Right, or Middle. |
| Click Type | Single or Double click. |
| Jitter | Adds a random ±N ms offset to each interval, making timing less robotic. |
| Click Limit | Stop automatically after N clicks, or run forever. |

### Advanced Tab

Lets you define a **sequence of screen coordinates** to click through in order.

- **＋ Add Row** — add a new position to the sequence.
- **⊙ Capture with F6** (or your configured key) — click this button, then hover over any spot on screen and press the capture key to record that coordinate automatically.
- Each row has its own X/Y position, mouse button, and per-click delay.
- **Loop** — repeat the sequence forever or a set number of times.

### Settings Tab

| Setting | Description |
|---|---|
| Toggle Start/Stop | Global hotkey to start or stop clicking (default: F8). |
| Capture Location | Global hotkey to capture mouse position in sequence mode (default: F6). |
| Always on top | Keep the window above all other windows. |

Keys can be set to any F1–F12 key. The two hotkeys cannot be set to the same key.

### Profiles

Type a name in the **Profile** field at the top and click **💾 Save** to save all current settings (including sequences) to `~/.config/awesomeclicker/<name>.json`.

Use the **📂 Load…** dropdown to reload a saved profile.

---

## Notes

### Wayland (Linux)

Global hotkeys (F6/F8) rely on low-level input hooks that are restricted on Wayland compositors. If the app shows a Wayland warning banner, run it under **XWayland**:

```bash
DISPLAY=:0 ./target/release/awesome-clicker
```

Or launch it from an X11 session instead of a Wayland one.

### Windows cross-compile (from Linux)

```bash
rustup target add x86_64-pc-windows-gnu
sudo apt-get install -y mingw-w64
cargo build --release --target x86_64-pc-windows-gnu
```

Output: `target/x86_64-pc-windows-gnu/release/awesome-clicker.exe`
