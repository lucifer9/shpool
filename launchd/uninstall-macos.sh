#!/bin/bash

# Shpool macOS LaunchAgent Uninstallation Script
# This script removes shpool launchd service on macOS

set -euo pipefail

PLIST_NAME="com.github.lucifer9.shpool.plist"
LAUNCHAGENTS_DIR="${HOME}/Library/LaunchAgents"
PLIST_DEST="${LAUNCHAGENTS_DIR}/${PLIST_NAME}"

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

stop_service() {
    log_info "Stopping shpool service..."
    
    # Check if service is loaded
    if launchctl list | grep -q "${PLIST_NAME%.*}"; then
        log_info "Service is running, stopping it..."
        launchctl unload "${PLIST_DEST}" 2>/dev/null || true
        log_success "Service stopped"
    else
        log_info "Service is not currently running"
    fi
}

remove_plist() {
    log_info "Removing LaunchAgent plist..."
    
    if [[ -f "${PLIST_DEST}" ]]; then
        rm "${PLIST_DEST}"
        log_success "Removed: ${PLIST_DEST}"
    else
        log_warning "Plist file not found: ${PLIST_DEST}"
    fi
}

cleanup_processes() {
    log_info "Cleaning up any remaining shpool processes..."
    
    # Kill any remaining shpool daemon processes
    pkill -f "shpool.*daemon" 2>/dev/null || true
    
    log_success "Process cleanup completed"
}

show_cleanup_info() {
    echo
    log_info "Uninstallation completed successfully!"
    echo
    echo "Note: The following items were NOT removed and may be cleaned up manually if desired:"
    echo "  - Shpool binary (wherever it was installed)"
    echo "  - Configuration files in ~/.config/shpool/ (if any)"
    echo "  - Runtime directory ~/.local/run/shpool/ (contains logs and session data)"
    echo "  - Session data and logs"
    echo
    echo "To completely remove all shpool data:"
    echo "  rm -rf ~/.local/run/shpool"
    echo "  rm -rf ~/.config/shpool"
    echo
}

main() {
    log_info "Starting shpool macOS uninstallation..."
    echo
    
    # Check if running on macOS
    if [[ "$OSTYPE" != "darwin"* ]]; then
        log_error "This script is only for macOS systems."
        exit 1
    fi
    
    stop_service
    remove_plist
    cleanup_processes
    
    show_cleanup_info
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi