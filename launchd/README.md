# Shpool macOS LaunchAgent

This directory contains macOS-specific files for running shpool as a user-level daemon using launchd, which is the macOS equivalent of systemd on Linux.

## Files

- `com.github.lucifer9.shpool.plist` - LaunchAgent property list configuration
- `install-macos.sh` - Installation script
- `uninstall-macos.sh` - Uninstallation script
- `README.md` - This documentation

## Installation

### Prerequisites

1. **Install shpool**: Make sure shpool is built and accessible in your PATH
   ```bash
   cargo build --release
   # Copy the binary to a location in PATH, e.g.:
   sudo cp target/release/shpool /usr/local/bin/
   ```

2. **Verify installation**: Check that shpool is available
   ```bash
   which shpool
   shpool --help
   ```

### Automatic Installation

The easiest way to install shpool as a macOS service:

```bash
cd launchd
./install-macos.sh
```

This script will:
- Check prerequisites
- Create necessary directories
- Install the LaunchAgent plist file
- Start the shpool service
- Verify it's working

### Manual Installation

If you prefer to install manually:

1. **Create directories**:
   ```bash
   mkdir -p ~/Library/LaunchAgents
   mkdir -p ~/.local/run/shpool
   ```

2. **Install the plist file**:
   ```bash
   # Adjust the shpool path to match your installation
   sed "s|/usr/local/bin/shpool|$(which shpool)|g" \
       com.github.lucifer9.shpool.plist > ~/Library/LaunchAgents/com.github.lucifer9.shpool.plist
   ```

3. **Load the service**:
   ```bash
   launchctl load ~/Library/LaunchAgents/com.github.lucifer9.shpool.plist
   ```

## Usage

Once installed, shpool will run automatically in the background and start on login.

### Basic Commands

```bash
# Create or attach to a session
shpool attach my-session

# List all sessions
shpool list

# Detach from current session (use the detach keybinding, default: Ctrl+D Ctrl+D)

# Kill a session
shpool kill my-session
```

### Service Management

```bash
# Check if service is running
launchctl list | grep shpool

# Stop the service
launchctl unload ~/Library/LaunchAgents/com.github.lucifer9.shpool.plist

# Start the service
launchctl load ~/Library/LaunchAgents/com.github.lucifer9.shpool.plist

# View service status details
launchctl list com.github.lucifer9.shpool
```

### Logs

Shpool logs are written to:
- **Standard output**: `~/.local/run/shpool/shpool.log`
- **Error output**: `~/.local/run/shpool/shpool.error.log`
- **Daemon logs**: `~/.local/run/shpool/daemonized-shpool.log`

View logs:
```bash
# Real-time daemon logs
tail -f ~/.local/run/shpool/daemonized-shpool.log

# Service logs
tail -f ~/.local/run/shpool/shpool.log

# Error logs
tail -f ~/.local/run/shpool/shpool.error.log
```

## Uninstallation

### Automatic Uninstallation

```bash
cd launchd
./uninstall-macos.sh
```

### Manual Uninstallation

```bash
# Stop and remove the service
launchctl unload ~/Library/LaunchAgents/com.github.lucifer9.shpool.plist
rm ~/Library/LaunchAgents/com.github.lucifer9.shpool.plist

# Kill any remaining processes
pkill -f "shpool.*daemon"

# Optionally remove data directories
rm -rf ~/.local/run/shpool
rm -rf ~/.config/shpool  # if you have config files
```

## Configuration

The LaunchAgent is configured to:
- **Run at login**: Automatically start when you log in
- **Keep alive**: Restart if the process crashes
- **Socket activation**: Similar to systemd socket activation
- **User-level**: Runs as your user, not system-wide
- **Logging**: Capture stdout/stderr to log files
- **Throttling**: Prevent rapid respawning if there are issues

### Key Configuration Parameters

- **Socket path**: `~/.local/run/shpool/shpool.socket`
- **Working directory**: Your home directory
- **Exit timeout**: 2 seconds (matches systemd configuration)
- **Throttle interval**: 10 seconds between rapid restarts

## Differences from systemd

| Feature | systemd | launchd |
|---------|---------|---------|
| Socket activation | `shpool.socket` + `shpool.service` | `Sockets` key in plist |
| Auto-restart | `Restart=always` | `KeepAlive=true` |
| User service | `--user` flag | User LaunchAgents directory |
| Logs | journalctl | File-based logging |
| Dependencies | `Requires=` | Built-in dependency handling |

## Troubleshooting

### Service won't start
1. Check the plist syntax: `plutil ~/Library/LaunchAgents/com.github.lucifer9.shpool.plist`
2. Verify shpool binary path: `which shpool`
3. Check permissions on socket directory: `ls -la ~/.local/run/shpool/`
4. Look at error logs: `cat ~/.local/run/shpool/shpool.error.log`

### Service keeps restarting
1. Check daemon logs: `tail -f ~/.local/run/shpool/daemonized-shpool.log`
2. Verify no conflicting processes: `ps aux | grep shpool`
3. Check socket permissions and path

### Can't connect to daemon
1. Verify service is running: `launchctl list | grep shpool`
2. Check socket file exists: `ls -la ~/.local/run/shpool/shpool.socket`
3. Test with verbose output: `shpool -vv list`

For more help, check the main shpool documentation or open an issue on GitHub.