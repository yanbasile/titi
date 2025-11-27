# Titi Terminal Emulator

A GPU-accelerated terminal emulator written in Rust with hierarchical tab/pane management, designed to be compatible with Claude Code.

## 🚀 Quick Start

**New to Titi?** Check out the [Getting Started Guide](GETTING_STARTED.md) for detailed installation instructions including how to install Rust, build dependencies, and run your first terminal session.

## Features

- **GPU-Accelerated Rendering**: Uses `wgpu` for high-performance text rendering
- **Hierarchical Panes**: VS Code-style pane management with splits
- **Full ANSI/VT100 Support**: Complete escape sequence support for tools like Claude Code
- **Configurable**: TOML-based configuration with sensible defaults
- **Cross-Platform**: Works on Linux, macOS, and Windows
- **🆕 Comprehensive Testing**: 90+ unit tests, 27+ stress tests with TDD approach
- **🆕 Real-time Metrics**: FPS, memory, per-terminal stats, performance profiling
- **🆕 Memory Leak Detection**: Automatic detection and warnings for memory issues
- **🆕 Production-Ready Monitoring**: Structured logging and metrics collection
- **🆕 Terminal Automation**: Redis-like server (redititi) for command injection and screen capture
- **🆕 Copy/Paste Support**: Clipboard integration with platform-specific handling
- **🆕 Mouse Support**: Click to focus panes and interact with terminal applications
- **🆕 Dirty Rectangle Tracking**: Performance optimization for selective rendering

## Architecture

### Components

- **Terminal Backend** (`src/terminal/`):
  - `pty.rs`: Pseudo-terminal (PTY) management using `portable-pty`
  - `grid.rs`: Terminal grid with cells and cursor management
  - `parser.rs`: VTE-based ANSI/VT100 escape sequence parser

- **GPU Renderer** (`src/renderer/`):
  - `gpu_state.rs`: wgpu setup and surface management
  - `text_renderer.rs`: Font rendering using `cosmic-text`

- **UI System** (`src/ui/`):
  - `pane.rs`: Individual terminal pane wrapper
  - `layout.rs`: Hierarchical split layout management

- **Configuration** (`src/config.rs`):
  - TOML-based configuration
  - Color schemes, fonts, window settings

- **🆕 Metrics & Monitoring** (`src/metrics.rs`):
  - Real-time performance tracking
  - Memory leak detection
  - Per-terminal statistics
  - Comprehensive logging

- **🆕 Testing Infrastructure** (`tests/`):
  - Unit tests: 90+ test cases
  - Stress tests: 27+ scenarios
  - Performance benchmarks
  - Concurrency tests

- **🆕 Automation Server** (`src/server/`, `src/bin/redititi.rs`):
  - Redis-like TCP server for terminal automation
  - Token-based authentication
  - Session/pane registry with pub/sub channels
  - Command injection and screen capture APIs
  - Standalone binary for external control

## Installation

**📖 For detailed installation instructions, see [GETTING_STARTED.md](GETTING_STARTED.md)**

### Quick Install (for experienced Rust users)

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/yourusername/titi.git
cd titi
cargo build --release

# Run
./target/release/titi
```

### What You'll Build

- **`titi`**: GPU-accelerated terminal emulator
- **`redititi`**: Redis-like automation server for terminal control

Both binaries will be available in `target/release/`.

## Usage

Run the terminal emulator:

```bash
cargo run --release
```

### Keyboard Shortcuts

- `Ctrl+T` or `Ctrl+Enter`: Create new terminal pane
- `Ctrl+Shift+C`: Copy selected text
- `Ctrl+Shift+V`: Paste from clipboard
- Mouse click: Focus pane
- Standard terminal key bindings (arrows, home, end, etc.)
- `Ctrl+[a-z]`: Control character combinations

### Running the Automation Server

Start the redititi server for terminal automation:

```bash
# Run with default settings (port 6379)
cargo run --bin redititi --release

# Or use the binary directly
./target/release/redititi

# Custom port
./target/release/redititi --port 6380
```

The server will display your authentication token on startup. Save it for connecting clients!

**Example commands** (using netcat):

```bash
nc localhost 6379
AUTH your_token_here
LIST SESSIONS
CREATE SESSION my-session
SUBSCRIBE session-{id}/pane-{id}/output
```

See [GETTING_STARTED.md](GETTING_STARTED.md) for more details on server automation.

### Configuration

Configuration file is located at:
- Linux: `~/.config/titi/config.toml`
- macOS: `~/Library/Application Support/titi/config.toml`
- Windows: `%APPDATA%\titi\config.toml`

Example configuration:

```toml
[font]
family = "monospace"
size = 14.0

[window]
width = 1280
height = 720
title = "Titi Terminal"

[shell]
program = "/bin/bash"  # Optional: defaults to $SHELL
args = []

[colors]
background = [0.0, 0.169, 0.212, 1.0]
foreground = [0.514, 0.580, 0.588, 1.0]
# ... more colors (Solarized Dark by default)
```

## Claude Code Compatibility

Titi is designed to work seamlessly with Claude Code by providing:

1. **Complete ANSI Support**: All standard escape sequences including:
   - Cursor movement (CSI sequences)
   - Text styling (SGR - bold, italic, colors)
   - Screen clearing and scrolling
   - 256-color and RGB color support

2. **Proper PTY Handling**: Full pseudo-terminal support for interactive sessions

3. **Responsive Input**: Fast key event processing with proper Ctrl+key combinations

## Development

### Project Structure

```
titi/
├── src/
│   ├── main.rs           # Application entry point
│   ├── lib.rs            # Library root
│   ├── config.rs         # Configuration management
│   ├── terminal/         # Terminal backend
│   │   ├── mod.rs
│   │   ├── pty.rs
│   │   ├── grid.rs
│   │   └── parser.rs
│   ├── renderer/         # GPU rendering
│   │   ├── mod.rs
│   │   ├── gpu_state.rs
│   │   └── text_renderer.rs
│   └── ui/               # Pane management
│       ├── mod.rs
│       ├── pane.rs
│       └── layout.rs
├── Cargo.toml
└── README.md
```

### Key Dependencies

- `wgpu`: GPU graphics API
- `winit`: Window management
- `vte`: Terminal escape sequence parsing
- `portable-pty`: Cross-platform PTY
- `cosmic-text`: Font rendering

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run with logging
RUST_LOG=debug cargo run
```

## Roadmap

### ✅ Completed

- [x] Complete text rendering with glyph atlas and caching
- [x] Multiple pane rendering and switching
- [x] Copy/paste support with clipboard integration
- [x] Mouse support for pane focus
- [x] Dirty rectangle tracking for performance
- [x] Comprehensive testing infrastructure (90+ tests)
- [x] Real-time metrics and monitoring
- [x] Memory leak detection
- [x] Redis-like automation server (redititi)

### 🚧 In Progress

- [ ] Terminal integration with redititi server
- [ ] Headless mode for automated testing
- [ ] Python client library (titipy)

### 📋 Planned

- [ ] Pane resize and drag-and-drop
- [ ] Scrollback buffer
- [ ] Configuration hot-reloading
- [ ] Custom key bindings
- [ ] Tabs in addition to panes
- [ ] Search functionality
- [ ] URL detection and opening
- [ ] Image protocol support (Sixel, iTerm2)
- [ ] Ligature support for coding fonts
- [ ] Multi-monitor support

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

MIT License - see LICENSE file for details

## Credits

Built with:
- [wgpu](https://github.com/gfx-rs/wgpu) - Modern GPU API
- [winit](https://github.com/rust-windowing/winit) - Window handling
- [vte](https://github.com/alacritty/vte) - VT parser
- [portable-pty](https://github.com/wez/wezterm/tree/main/pty) - PTY implementation
- [cosmic-text](https://github.com/pop-os/cosmic-text) - Text rendering

Inspired by:
- [Alacritty](https://github.com/alacritty/alacritty)
- [WezTerm](https://github.com/wez/wezterm)
- [Kitty](https://github.com/kovidgoyal/kitty)
