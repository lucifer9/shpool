#!/bin/bash

# Shpool macOS LaunchAgent Installation Script
# This script installs shpool as a user-level launchd service on macOS

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLIST_NAME="com.github.lucifer9.shpool.plist"
PLIST_SOURCE="${SCRIPT_DIR}/${PLIST_NAME}"
LAUNCHAGENTS_DIR="${HOME}/Library/LaunchAgents"
PLIST_DEST="${LAUNCHAGENTS_DIR}/${PLIST_NAME}"
RUN_DIR="${HOME}/.local/run/shpool"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_requirements() {
    log_info "Checking requirements..."
    
    # Check if shpool binary exists
    if ! command -v shpool >/dev/null 2>&1; then
        log_error "shpool binary not found in PATH. Please install shpool first."
        log_info "You can install it with: cargo install --path ."
        exit 1
    fi
    
    # Get shpool binary path
    SHPOOL_PATH=$(command -v shpool)
    log_info "Found shpool at: ${SHPOOL_PATH}"
    
    # Check if running on macOS
    if [[ "$OSTYPE" != "darwin"* ]]; then
        log_error "This script is only for macOS systems."
        exit 1
    fi
    
    log_success "Requirements check passed"
}

create_directories() {
    log_info "Creating necessary directories..."
    
    # Create LaunchAgents directory if it doesn't exist
    mkdir -p "${LAUNCHAGENTS_DIR}"
    log_info "Created: ${LAUNCHAGENTS_DIR}"
    
    # Create shpool run directory
    mkdir -p "${RUN_DIR}"
    log_info "Created: ${RUN_DIR}"
    
    log_success "Directories created successfully"
}

install_plist() {
    log_info "Installing LaunchAgent plist..."
    
    # Get actual shpool path
    SHPOOL_PATH=$(command -v shpool)
    
    # Copy and customize the plist file (only replace shpool path)
    sed "s|/usr/local/bin/shpool|${SHPOOL_PATH}|g" \
        "${PLIST_SOURCE}" > "${PLIST_DEST}"
    
    log_info "Installed plist to: ${PLIST_DEST}"
    log_success "LaunchAgent plist installed successfully"
}

start_service() {
    log_info "Starting shpool service..."
    
    # Unload if already loaded (in case of reinstall)
    if launchctl list | grep -q "${PLIST_NAME%.*}"; then
        log_warning "Service already loaded, unloading first..."
        launchctl unload "${PLIST_DEST}" 2>/dev/null || true
    fi
    
    # Load the service
    launchctl load "${PLIST_DEST}"
    
    # Verify it's running
    sleep 2
    if launchctl list | grep -q "${PLIST_NAME%.*}"; then
        log_success "Shpool service started successfully"
        
        # Test the service
        log_info "Testing shpool service..."
        if shpool list >/dev/null 2>&1; then
            log_success "Shpool service is responding correctly"
        else
            log_warning "Shpool service is running but may not be responding correctly"
        fi
    else
        log_error "Failed to start shpool service"
        exit 1
    fi
}

show_usage() {
    log_success "Installation completed successfully!"
    echo
    echo "Shpool is now running as a background service and will start automatically on login."
    echo
    echo "Usage commands:"
    echo "  shpool attach <session-name>  - Create or attach to a session"
    echo "  shpool list                   - List all sessions"
    echo "  shpool detach <session-name>  - Detach from a session"
    echo "  shpool kill <session-name>    - Kill a session"
    echo
    echo "Service management commands:"
    echo "  launchctl list | grep shpool  - Check service status"
    echo "  launchctl unload ~/Library/LaunchAgents/${PLIST_NAME} - Stop service"
    echo "  launchctl load ~/Library/LaunchAgents/${PLIST_NAME}   - Start service"
    echo
    echo "Log files:"
    echo "  Standard output: ~/.local/run/shpool/shpool.log"
    echo "  Error output:    ~/.local/run/shpool/shpool.error.log"
    echo "  Daemon log:      ~/.local/run/shpool/daemonized-shpool.log"
}

main() {
    log_info "Starting shpool macOS installation..."
    echo
    
    check_requirements
    create_directories
    install_plist
    start_service
    
    echo
    show_usage
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi