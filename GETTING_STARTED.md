# Getting Started with Titi Terminal Emulator

This guide will walk you through installing Rust, building Titi, and running your first terminal session.

## Table of Contents

1. [System Requirements](#system-requirements)
2. [Installing Rust](#installing-rust)
3. [Installing System Dependencies](#installing-system-dependencies)
4. [Building Titi](#building-titi)
5. [Running Titi Terminal](#running-titi-terminal)
6. [Running Redititi Server](#running-redititi-server)
7. [Configuration](#configuration)
8. [Troubleshooting](#troubleshooting)

---

## System Requirements

### Minimum Requirements

- **Operating System**: Linux, macOS, or Windows
- **Graphics**: GPU with Vulkan, Metal (macOS), or DirectX 12 support
- **Memory**: 512 MB RAM minimum (1 GB recommended)
- **Disk Space**: 500 MB for Rust toolchain + dependencies

### Supported Platforms

- **Linux**: Any modern distribution (Ubuntu 20.04+, Fedora 35+, Arch, etc.)
- **macOS**: 10.15 (Catalina) or later
- **Windows**: Windows 10/11 with DirectX 12 support

---

## Installing Rust

Titi is written in Rust, so you'll need to install the Rust toolchain first.

### Linux & macOS

1. Open your terminal and run:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Follow the prompts (usually just press Enter to accept defaults)

3. Restart your terminal or run:

```bash
source $HOME/.cargo/env
```

4. Verify installation:

```bash
rustc --version
cargo --version
```

You should see something like:
```
rustc 1.83.0 (90b35a623 2024-11-26)
cargo 1.83.0 (5ffbef321 2024-10-29)
```

### Windows

1. Download and run [rustup-init.exe](https://rustup.rs/)

2. Follow the installer prompts

3. You may need to install [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

4. Restart your terminal and verify:

```powershell
rustc --version
cargo --version
```

### Updating Rust

To update Rust to the latest version:

```bash
rustup update
```

---

## Installing System Dependencies

### Linux

**Ubuntu/Debian:**

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev \
  libfontconfig1-dev libfreetype6-dev libxcb-composite0-dev \
  libx11-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

**Fedora:**

```bash
sudo dnf install -y gcc pkg-config openssl-devel \
  fontconfig-devel freetype-devel libxcb-devel \
  libX11-devel
```

**Arch Linux:**

```bash
sudo pacman -S base-devel pkg-config openssl \
  fontconfig freetype2 libxcb libx11
```

### macOS

Install Xcode Command Line Tools:

```bash
xcode-select --install
```

### Windows

Install [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

During installation, select:
- "Desktop development with C++"
- Windows 10/11 SDK

---

## Building Titi

### 1. Clone the Repository

```bash
git clone https://github.com/yourusername/titi.git
cd titi
```

### 2. Build the Project

**For testing and development (debug build):**

```bash
cargo build
```

**For production use (optimized release build):**

```bash
cargo build --release
```

The release build takes longer but runs much faster and uses less memory.

### 3. Build Output Locations

- **Debug build**: `target/debug/titi` and `target/debug/redititi`
- **Release build**: `target/release/titi` and `target/release/redititi`

### 4. Optional: Install System-Wide

To install the binaries to your PATH:

```bash
cargo install --path .
```

This installs both `titi` and `redititi` to `~/.cargo/bin/` (automatically in your PATH).

---

## Running Titi Terminal

### Basic Usage

**Using cargo (from the project directory):**

```bash
# Run debug build
cargo run

# Run release build
cargo run --release
```

**Using the binary directly:**

```bash
# Debug build
./target/debug/titi

# Release build
./target/release/titi

# If installed system-wide
titi
```

### Keyboard Shortcuts

Once Titi is running:

- **`Ctrl+T`** or **`Ctrl+Enter`**: Create new pane
- **`Ctrl+C`**: Send interrupt signal (SIGINT)
- **`Ctrl+D`**: Send EOF (exit shell)
- **Arrow keys**, **Home**, **End**, **Page Up/Down**: Navigation
- **`Ctrl+Shift+C`**: Copy (when copy/paste is enabled)
- **`Ctrl+Shift+V`**: Paste (when copy/paste is enabled)

### Command-Line Options

```bash
# Show help
titi --help

# Specify custom config file
titi --config /path/to/config.toml

# Run in headless mode (no GUI, for automation)
titi --headless

# Verbose logging
RUST_LOG=debug titi
```

---

## Running Redititi Server

Redititi is a Redis-like server that enables terminal automation through command injection and screen capture.

### Starting the Server

**Basic usage:**

```bash
# Run with cargo
cargo run --bin redititi

# Run binary directly
./target/release/redititi

# If installed system-wide
redititi
```

**Custom port:**

```bash
redititi --port 6380
```

**Custom token file:**

```bash
redititi --token-file /path/to/token
```

### Server Information

On startup, you'll see:

```
═══════════════════════════════════════════════
  Redititi - Terminal Automation Server
═══════════════════════════════════════════════
Port:       6379
Token file: /home/user/.titi/token
Token:      abc123...xyz789
```

**Important:** Save the token! You'll need it to connect clients.

### Connecting to the Server

The server listens on `127.0.0.1:6379` (localhost only for security).

**Using telnet/netcat for testing:**

```bash
# Connect
nc localhost 6379

# Authenticate
AUTH your_token_here

# List sessions
LIST SESSIONS

# Create a session
CREATE SESSION
```

### Server Architecture

- **Token Authentication**: Auto-generated 64-character token stored in `~/.titi/token`
- **Session Management**: Create/manage multiple terminal sessions
- **Pub/Sub Channels**: Real-time communication between terminals
- **Command Injection**: Send commands to terminals programmatically
- **Screen Capture**: Extract terminal output for automation

---

## Configuration

### Configuration File Location

Titi looks for configuration in the following locations:

- **Linux**: `~/.config/titi/config.toml`
- **macOS**: `~/Library/Application Support/titi/config.toml`
- **Windows**: `%APPDATA%\titi\config.toml`

### Creating Your First Config

Create the config directory:

```bash
# Linux
mkdir -p ~/.config/titi

# macOS
mkdir -p ~/Library/Application\ Support/titi

# Windows (PowerShell)
mkdir $env:APPDATA\titi
```

Create `config.toml`:

```toml
# Font Configuration
[font]
family = "monospace"  # Or "JetBrains Mono", "Fira Code", etc.
size = 14.0

# Window Settings
[window]
width = 1280
height = 720
title = "Titi Terminal"

# Shell Configuration
[shell]
program = "/bin/bash"  # macOS/Linux
# program = "powershell.exe"  # Windows
args = []

# Color Scheme (Solarized Dark)
[colors]
background = [0.0, 0.169, 0.212, 1.0]      # #002b36
foreground = [0.514, 0.580, 0.588, 1.0]    # #839496

# ANSI Colors
black = [0.027, 0.212, 0.259, 1.0]         # #073642
red = [0.863, 0.196, 0.184, 1.0]           # #dc322f
green = [0.522, 0.600, 0.0, 1.0]           # #859900
yellow = [0.710, 0.537, 0.0, 1.0]          # #b58900
blue = [0.149, 0.545, 0.824, 1.0]          # #268bd2
magenta = [0.827, 0.212, 0.510, 1.0]       # #d33682
cyan = [0.165, 0.631, 0.596, 1.0]          # #2aa198
white = [0.933, 0.910, 0.835, 1.0]         # #eee8d5

# Bright Colors
bright_black = [0.0, 0.169, 0.212, 1.0]    # #002b36
bright_red = [0.796, 0.294, 0.086, 1.0]    # #cb4b16
bright_green = [0.345, 0.431, 0.459, 1.0]  # #586e75
bright_yellow = [0.396, 0.482, 0.514, 1.0] # #657b83
bright_blue = [0.514, 0.580, 0.588, 1.0]   # #839496
bright_magenta = [0.423, 0.443, 0.769, 1.0] # #6c71c4
bright_cyan = [0.576, 0.631, 0.631, 1.0]   # #93a1a1
bright_white = [0.992, 0.965, 0.890, 1.0]  # #fdf6e3
```

### Popular Color Schemes

**One Dark:**

```toml
[colors]
background = [0.157, 0.165, 0.212, 1.0]
foreground = [0.671, 0.682, 0.745, 1.0]
```

**Dracula:**

```toml
[colors]
background = [0.157, 0.165, 0.212, 1.0]
foreground = [0.945, 0.945, 0.945, 1.0]
```

**Nord:**

```toml
[colors]
background = [0.180, 0.204, 0.251, 1.0]
foreground = [0.847, 0.871, 0.914, 1.0]
```

---

## Troubleshooting

### Build Issues

**Error: "linker 'cc' not found"**

Install build tools:
- **Linux**: `sudo apt install build-essential`
- **macOS**: `xcode-select --install`
- **Windows**: Install Visual Studio C++ Build Tools

**Error: "could not find system library 'fontconfig'"**

Install fontconfig:
- **Ubuntu/Debian**: `sudo apt install libfontconfig1-dev`
- **Fedora**: `sudo dnf install fontconfig-devel`
- **Arch**: `sudo pacman -S fontconfig`

**Error: "network error accessing crates.io"**

- Check your internet connection
- Try: `cargo build --offline` (if dependencies were already downloaded)
- Or wait and retry - this is often a temporary network issue

### Runtime Issues

**Terminal window doesn't appear**

Check graphics drivers:
```bash
# Linux - check Vulkan support
vulkaninfo | grep deviceName

# Test with debug logging
RUST_LOG=debug cargo run
```

**Text not rendering / garbled display**

- Update your graphics drivers
- Try a different font in `config.toml`
- Check terminal size: resize the window

**High CPU/GPU usage**

- Use release build: `cargo run --release`
- Close unused panes
- Check for runaway processes in the terminal

**Token authentication failing (redititi)**

Check token file:
```bash
# Linux/macOS
cat ~/.titi/token

# Windows (PowerShell)
type $env:USERPROFILE\.titi\token
```

If missing, delete `~/.titi/token` and restart redititi to regenerate.

### Performance Tips

1. **Always use release builds** for daily use:
   ```bash
   cargo build --release
   ```

2. **Enable logging only when debugging**:
   ```bash
   # Normal use (no logging)
   ./target/release/titi

   # Debug mode (verbose)
   RUST_LOG=debug ./target/release/titi
   ```

3. **Monitor resource usage**:
   ```bash
   # Linux
   htop

   # macOS
   top

   # Windows
   Task Manager
   ```

### Getting Help

If you encounter issues:

1. **Check logs**: Run with `RUST_LOG=debug` for detailed output
2. **Search issues**: Visit the GitHub issues page
3. **File a bug report**: Include:
   - Operating system and version
   - Rust version (`rustc --version`)
   - Full error message
   - Steps to reproduce

---

## Next Steps

Now that you have Titi running:

1. **Explore the features**: Try creating panes with `Ctrl+T`
2. **Customize your config**: Edit colors, fonts, and shortcuts
3. **Try automation**: Start redititi server and experiment with commands
4. **Read the documentation**: Check README.md and other docs
5. **Contribute**: Found a bug or want a feature? Open an issue!

---

## Quick Reference

### Build Commands

```bash
cargo build              # Debug build
cargo build --release    # Release build (faster)
cargo run                # Build + run debug
cargo run --release      # Build + run release
cargo test               # Run tests
cargo install --path .   # Install to ~/.cargo/bin
```

### Run Commands

```bash
titi                     # Start terminal emulator
titi --help              # Show help
redititi                 # Start automation server
redititi --port 6380     # Custom port
```

### Configuration Locations

- **Linux**: `~/.config/titi/config.toml`
- **macOS**: `~/Library/Application Support/titi/config.toml`
- **Windows**: `%APPDATA%\titi\config.toml`
- **Token**: `~/.titi/token` (redititi authentication)

---

## Additional Resources

- [README.md](README.md) - Project overview and features
- [TESTING.md](TESTING.md) - Running tests and benchmarks
- [TDD_TEST_PLAN.md](TDD_TEST_PLAN.md) - Test-driven development approach
- [Rust Documentation](https://doc.rust-lang.org/book/) - Learn Rust
- [wgpu Tutorial](https://sotrh.github.io/learn-wgpu/) - Understanding GPU rendering

---

Happy terminal emulating! 🚀
