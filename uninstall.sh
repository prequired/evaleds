#!/usr/bin/env bash
# EvalEds Uninstallation Script
# 
# This script removes EvalEds from your system while providing options
# to preserve or remove configuration and data files.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
BINARY_NAME="evaleds"
POSSIBLE_INSTALL_DIRS=(
    "$HOME/.local/bin"
    "$HOME/bin"
    "/usr/local/bin"
    "/usr/bin"
)
CONFIG_DIRS=(
    "$HOME/.config/evaleds"
    "$HOME/.evaleds"
)
DATA_DIRS=(
    "$HOME/.local/share/evaleds"
    "$HOME/.evaleds"
)

# Command line options
REMOVE_CONFIG=false
REMOVE_DATA=false
FORCE=false
DRY_RUN=false

# Print colored output
print_status() {
    echo -e "${BLUE}==>${NC} ${1}"
}

print_success() {
    echo -e "${GREEN}✅${NC} ${1}"
}

print_warning() {
    echo -e "${YELLOW}⚠️${NC} ${1}"
}

print_error() {
    echo -e "${RED}❌${NC} ${1}" >&2
}

print_info() {
    echo -e "${CYAN}💡${NC} ${1}"
}

print_dry_run() {
    echo -e "${YELLOW}[DRY RUN]${NC} Would ${1}"
}

# Ask for confirmation
confirm() {
    local message="$1"
    local default="${2:-n}"
    
    if [[ "$FORCE" == true ]]; then
        return 0
    fi
    
    local prompt=""
    if [[ "$default" == "y" ]]; then
        prompt="$message [Y/n]: "
    else
        prompt="$message [y/N]: "
    fi
    
    while true; do
        echo -ne "${YELLOW}❓${NC} $prompt"
        read -r response
        
        # Use default if empty response
        if [[ -z "$response" ]]; then
            response="$default"
        fi
        
        case "$response" in
            [Yy]|[Yy][Ee][Ss]) return 0 ;;
            [Nn]|[Nn][Oo]) return 1 ;;
            *) echo "Please answer y or n." ;;
        esac
    done
}

# Find EvalEds binary
find_binary() {
    local found_paths=()
    
    for dir in "${POSSIBLE_INSTALL_DIRS[@]}"; do
        if [[ -f "$dir/$BINARY_NAME" ]]; then
            found_paths+=("$dir/$BINARY_NAME")
        fi
    done
    
    # Also check PATH
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        local path_binary=$(command -v "$BINARY_NAME")
        # Only add if not already in our list
        local already_found=false
        for found in "${found_paths[@]}"; do
            if [[ "$found" == "$path_binary" ]]; then
                already_found=true
                break
            fi
        done
        if [[ "$already_found" == false ]]; then
            found_paths+=("$path_binary")
        fi
    fi
    
    printf '%s\n' "${found_paths[@]}"
}

# Remove binary files
remove_binaries() {
    local binaries=($(find_binary))
    
    if [[ ${#binaries[@]} -eq 0 ]]; then
        print_info "No EvalEds binaries found"
        return 0
    fi
    
    print_status "Found EvalEds binaries:"
    for binary in "${binaries[@]}"; do
        echo "  $binary"
    done
    
    if confirm "Remove EvalEds binary files?" "y"; then
        for binary in "${binaries[@]}"; do
            if [[ "$DRY_RUN" == true ]]; then
                print_dry_run "remove $binary"
            else
                if rm -f "$binary" 2>/dev/null; then
                    print_success "Removed $binary"
                else
                    print_warning "Could not remove $binary (insufficient permissions?)"
                fi
            fi
        done
    else
        print_info "Skipping binary removal"
    fi
}

# Remove configuration files
remove_configuration() {
    local found_configs=()
    
    for dir in "${CONFIG_DIRS[@]}"; do
        if [[ -d "$dir" ]]; then
            found_configs+=("$dir")
        fi
    done
    
    if [[ ${#found_configs[@]} -eq 0 ]]; then
        print_info "No configuration directories found"
        return 0
    fi
    
    if [[ "$REMOVE_CONFIG" == true ]] || confirm "Remove configuration files and directories?" "n"; then
        for config_dir in "${found_configs[@]}"; do
            if [[ "$DRY_RUN" == true ]]; then
                print_dry_run "remove configuration directory $config_dir"
            else
                if rm -rf "$config_dir" 2>/dev/null; then
                    print_success "Removed configuration directory $config_dir"
                else
                    print_warning "Could not remove $config_dir"
                fi
            fi
        done
    else
        print_info "Preserving configuration files"
        echo "Configuration files preserved in:"
        for config_dir in "${found_configs[@]}"; do
            echo "  $config_dir"
        done
    fi
}

# Remove data files (databases, exports, etc.)
remove_data() {
    local found_data=()
    
    for dir in "${DATA_DIRS[@]}"; do
        if [[ -d "$dir" ]]; then
            found_data+=("$dir")
        fi
    done
    
    # Also check for database files in config directories
    for dir in "${CONFIG_DIRS[@]}"; do
        if [[ -d "$dir" ]]; then
            if find "$dir" -name "*.db" -o -name "*.db-*" | grep -q .; then
                found_data+=("$dir/*.db*")
            fi
        fi
    done
    
    if [[ ${#found_data[@]} -eq 0 ]]; then
        print_info "No data directories found"
        return 0
    fi
    
    if [[ "$REMOVE_DATA" == true ]] || confirm "Remove data files (evaluations, databases, exports)?" "n"; then
        for data_path in "${found_data[@]}"; do
            if [[ "$DRY_RUN" == true ]]; then
                print_dry_run "remove data files at $data_path"
            else
                if [[ "$data_path" == *"*.db*" ]]; then
                    # Remove database files specifically
                    local dir_path="${data_path%/*}"
                    find "$dir_path" -name "*.db" -o -name "*.db-*" -exec rm -f {} \; 2>/dev/null && \
                        print_success "Removed database files from $dir_path"
                else
                    # Remove entire directory
                    if rm -rf "$data_path" 2>/dev/null; then
                        print_success "Removed data directory $data_path"
                    else
                        print_warning "Could not remove $data_path"
                    fi
                fi
            fi
        done
    else
        print_info "Preserving data files"
        echo "Data files preserved in:"
        for data_path in "${found_data[@]}"; do
            echo "  $data_path"
        done
    fi
}

# Remove from shell PATH (best effort)
remove_from_path() {
    local shell_configs=(
        "$HOME/.bashrc"
        "$HOME/.zshrc"
        "$HOME/.profile"
        "$HOME/.config/fish/config.fish"
    )
    
    local found_path_entries=()
    
    for config_file in "${shell_configs[@]}"; do
        if [[ -f "$config_file" ]] && grep -q "evaleds\|\.local/bin" "$config_file" 2>/dev/null; then
            found_path_entries+=("$config_file")
        fi
    done
    
    if [[ ${#found_path_entries[@]} -eq 0 ]]; then
        print_info "No PATH entries found to remove"
        return 0
    fi
    
    if confirm "Attempt to remove EvalEds-related PATH entries from shell configs?" "n"; then
        for config_file in "${found_path_entries[@]}"; do
            if [[ "$DRY_RUN" == true ]]; then
                print_dry_run "clean PATH entries in $config_file"
            else
                print_info "Please manually review and clean $config_file if needed"
            fi
        done
    fi
}

# Check for running processes
check_running_processes() {
    if pgrep -f "$BINARY_NAME" >/dev/null 2>&1; then
        print_warning "EvalEds processes are currently running"
        if confirm "Kill running EvalEds processes?" "y"; then
            if [[ "$DRY_RUN" == true ]]; then
                print_dry_run "kill EvalEds processes"
            else
                pkill -f "$BINARY_NAME" 2>/dev/null || true
                print_success "Stopped running EvalEds processes"
            fi
        else
            print_warning "Some processes may still be running after uninstall"
        fi
    fi
}

# Show usage information
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Uninstall EvalEds from your system."
    echo ""
    echo "Options:"
    echo "  --remove-config       Remove configuration files without prompting"
    echo "  --remove-data         Remove data files (evaluations, databases) without prompting"
    echo "  --remove-all          Remove everything without prompting"
    echo "  --force              Skip all confirmation prompts (use with caution)"
    echo "  --dry-run            Show what would be removed without actually removing"
    echo "  --help, -h           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                   # Interactive uninstall (recommended)"
    echo "  $0 --dry-run         # See what would be removed"
    echo "  $0 --remove-all      # Remove everything including data"
    echo "  $0 --force           # Uninstall without confirmations"
}

# Main uninstallation function
main() {
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --remove-config)
                REMOVE_CONFIG=true
                shift
                ;;
            --remove-data)
                REMOVE_DATA=true
                shift
                ;;
            --remove-all)
                REMOVE_CONFIG=true
                REMOVE_DATA=true
                shift
                ;;
            --force)
                FORCE=true
                shift
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --help|-h)
                show_usage
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                print_info "Use --help for usage information"
                exit 1
                ;;
        esac
    done
    
    echo ""
    echo -e "${CYAN}🗑️  EvalEds Uninstallation Script${NC}"
    echo ""
    
    if [[ "$DRY_RUN" == true ]]; then
        print_warning "DRY RUN MODE - Nothing will actually be removed"
        echo ""
    fi
    
    # Check for running processes first
    check_running_processes
    
    print_status "Scanning system for EvalEds components..."
    
    # Show what was found
    local binaries=($(find_binary))
    local config_count=0
    local data_count=0
    
    for dir in "${CONFIG_DIRS[@]}"; do
        if [[ -d "$dir" ]]; then
            ((config_count++))
        fi
    done
    
    for dir in "${DATA_DIRS[@]}"; do
        if [[ -d "$dir" ]]; then
            ((data_count++))
        fi
    done
    
    echo ""
    print_info "Found:"
    echo "  📦 Binaries: ${#binaries[@]}"
    echo "  ⚙️  Configuration directories: $config_count"
    echo "  💾 Data directories: $data_count"
    echo ""
    
    if [[ ${#binaries[@]} -eq 0 && $config_count -eq 0 && $data_count -eq 0 ]]; then
        print_success "EvalEds does not appear to be installed"
        exit 0
    fi
    
    if [[ "$DRY_RUN" != true ]]; then
        if ! confirm "Proceed with uninstallation?" "y"; then
            print_info "Uninstallation cancelled"
            exit 0
        fi
        echo ""
    fi
    
    # Perform uninstallation steps
    remove_binaries
    remove_configuration
    remove_data
    remove_from_path
    
    echo ""
    if [[ "$DRY_RUN" == true ]]; then
        print_info "Dry run complete. Run without --dry-run to actually uninstall"
    else
        print_success "🎯 EvalEds uninstallation complete!"
        print_info "Thank you for using EvalEds!"
        echo ""
        print_info "If you encountered any issues, please report them at:"
        print_info "https://github.com/prequired/evaleds/issues"
    fi
}

# Run main function with all arguments
main "$@"