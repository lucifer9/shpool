# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Shpool is a session persistence tool that enables persistent shell sessions without connection loss. It's designed as a lightweight alternative to tmux/screen that focuses solely on session persistence while preserving native terminal features like scrollback and copy-paste.

## Build Commands

### Development Build
```bash
cargo build
```

### Release Build
```bash
cargo build --release
```

### Running Tests
```bash
cargo test
```

### Running Specific Tests
```bash
cargo test --test <test-suite-name> <test_name> -- --nocapture
```

### Formatting
```bash
cargo +nightly fmt
```

### Running a Single Test with Logs
```bash
SHPOOL_LEAVE_TEST_LOGS=true cargo test --test attach happy_path -- --nocapture
```

## Architecture

### Workspace Structure
- `libshpool/` - Core library containing all business logic
- `shpool/` - Main binary that provides CLI interface
- `shpool-protocol/` - Communication protocol between client and daemon

### Key Components

#### Daemon Architecture (`libshpool/src/daemon/`)
- `server.rs` - Main server that handles client connections and manages sessions
- `shell.rs` - Shell process management and terminal interaction
- `keybindings.rs` - Input handling and keybinding system (Ctrl-Space Ctrl-q to detach)
- `prompt.rs` - Automatic prompt prefix injection for bash/zsh/fish
- `ttl_reaper.rs` - Session cleanup based on time-to-live settings
- `signals.rs` - Signal handling for daemon lifecycle

#### Client Commands (`libshpool/src/`)
- `attach.rs` - Session creation and attachment logic
- `detach.rs` - Session detachment without termination
- `kill.rs` - Session termination with proper cleanup
- `list.rs` - Session listing functionality

#### Session Restore System (`libshpool/src/session_restore/`)
- Maintains in-memory terminal state using `shpool_vt100` crate
- Redraws terminal content on reattach after disconnection
- Shows output generated while disconnected

### Configuration System
- Uses TOML configuration files at `~/.config/shpool/config.toml`
- Dynamic configuration reloading via `config_watcher.rs`
- See `CONFIG.md` for configuration options

### Process Model
- Daemon process manages multiple shell sessions
- Unix domain socket communication (`$XDG_RUNTIME_DIR/shpool/shpool.socket`)
- Systemd socket activation support
- Auto-daemonization when no daemon is running

### Terminal Integration
- Direct byte forwarding (no terminal multiplexing)
- Native scrollback and copy-paste preserved
- Shell detection and automatic prompt prefixing
- TTY management for proper signal handling

## Testing

### Test Structure
- Integration tests in `shpool/tests/`
- Test support utilities in `shpool/tests/support/`
- Test data configurations in `shpool/tests/data/`

### Debug Testing with rr
```bash
cargo test --test <test-suite-name> --no-run
SHPOOL_LEAVE_TEST_LOGS=true rr ./path/to/test/exe <test_name> --nocapture
rr replay --debugger=rust-gdb --onprocess=<PID>
```

## Development Notes

- Minimum Rust version: 1.74.0
- Uses conventional commits for changelog generation
- Release managed by release-plz
- Semver versioning with libshpool/shpool in lockstep
- MSRV targets Debian stable rust version