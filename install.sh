#!/usr/bin/env bash
# EvalEds Installation Script
# 
# This script installs EvalEds, the AI evaluation platform with PromptEds integration.
# It can install from GitHub releases or build from source.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
REPO_URL="https://github.com/prequired/evaleds"
BINARY_NAME="evaleds"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
CONFIG_DIR="${CONFIG_DIR:-$HOME/.config/evaleds}"

# Platform detection
detect_platform() {
    local platform=""
    local arch=""
    
    case "$(uname -s)" in
        Linux*)   platform="linux" ;;
        Darwin*)  platform="macos" ;;
        CYGWIN*|MINGW*|MSYS*) platform="windows" ;;
        *)        platform="unknown" ;;
    esac
    
    case "$(uname -m)" in
        x86_64|amd64) arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
        armv7l) arch="armv7" ;;
        *) arch="unknown" ;;
    esac
    
    echo "${platform}-${arch}"
}

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

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check for curl or wget
    if ! command_exists curl && ! command_exists wget; then
        print_error "curl or wget is required for installation"
        exit 1
    fi
    
    # Check for tar
    if ! command_exists tar; then
        print_error "tar is required for installation"
        exit 1
    fi
    
    print_success "Prerequisites check passed"
}

# Create installation directory
create_install_dir() {
    if [[ ! -d "$INSTALL_DIR" ]]; then
        print_status "Creating installation directory: $INSTALL_DIR"
        mkdir -p "$INSTALL_DIR"
    fi
}

# Create configuration directory
create_config_dir() {
    if [[ ! -d "$CONFIG_DIR" ]]; then
        print_status "Creating configuration directory: $CONFIG_DIR"
        mkdir -p "$CONFIG_DIR"
        
        # Create default configuration
        cat > "$CONFIG_DIR/config.toml" << 'EOF'
# EvalEds Configuration File
# See https://github.com/prequired/evaleds for documentation

[defaults]
temperature = 0.7
max_tokens = 1000
timeout_seconds = 120
max_concurrent = 5
retry_attempts = 3

[analysis]
enable_similarity_analysis = true
enable_content_analysis = true
enable_quality_assessment = true
similarity_threshold = 0.7
max_keywords = 10

# Provider configurations will be loaded from separate files
# or environment variables (OPENAI_API_KEY, ANTHROPIC_API_KEY, etc.)
EOF
        print_success "Created default configuration at $CONFIG_DIR/config.toml"
    fi
}

# Get latest release version
get_latest_version() {
    local version=""
    
    if command_exists curl; then
        version=$(curl -sSL "https://api.github.com/repos/prequired/evaleds/releases/latest" | grep '"tag_name"' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')
    elif command_exists wget; then
        version=$(wget -qO- "https://api.github.com/repos/prequired/evaleds/releases/latest" | grep '"tag_name"' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')
    fi
    
    echo "$version"
}

# Download and install binary
install_binary() {
    local platform=$(detect_platform)
    local version=$(get_latest_version)
    
    if [[ -z "$version" ]]; then
        print_warning "Could not determine latest version, building from source..."
        build_from_source
        return
    fi
    
    print_status "Installing EvalEds $version for platform $platform"
    
    local download_url="${REPO_URL}/releases/download/${version}/evaleds-${version}-${platform}.tar.gz"
    local temp_dir=$(mktemp -d)
    local temp_file="${temp_dir}/evaleds.tar.gz"
    
    # Download binary
    print_status "Downloading from $download_url"
    if command_exists curl; then
        if ! curl -sSL "$download_url" -o "$temp_file"; then
            print_warning "Binary download failed, building from source..."
            rm -rf "$temp_dir"
            build_from_source
            return
        fi
    elif command_exists wget; then
        if ! wget -q "$download_url" -O "$temp_file"; then
            print_warning "Binary download failed, building from source..."
            rm -rf "$temp_dir"
            build_from_source
            return
        fi
    fi
    
    # Extract and install
    print_status "Extracting and installing binary..."
    cd "$temp_dir"
    tar -xzf "$temp_file"
    
    if [[ -f "$BINARY_NAME" ]]; then
        chmod +x "$BINARY_NAME"
        mv "$BINARY_NAME" "$INSTALL_DIR/"
        print_success "Binary installed to $INSTALL_DIR/$BINARY_NAME"
    else
        print_error "Binary not found in archive"
        rm -rf "$temp_dir"
        exit 1
    fi
    
    # Cleanup
    rm -rf "$temp_dir"
}

# Build from source using Rust/Cargo
build_from_source() {
    print_status "Building EvalEds from source..."
    
    # Check for Rust/Cargo
    if ! command_exists cargo; then
        print_error "Cargo (Rust) is required to build from source"
        print_info "Install Rust from: https://rustup.rs/"
        exit 1
    fi
    
    local temp_dir=$(mktemp -d)
    cd "$temp_dir"
    
    # Clone repository
    print_status "Cloning repository..."
    if command_exists git; then
        git clone "$REPO_URL.git" evaleds
    else
        print_error "Git is required to clone the repository"
        exit 1
    fi
    
    cd evaleds
    
    # Build release binary
    print_status "Building release binary... (this may take a few minutes)"
    cargo build --release
    
    # Install binary
    if [[ -f "target/release/$BINARY_NAME" ]]; then
        cp "target/release/$BINARY_NAME" "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
        print_success "Binary built and installed to $INSTALL_DIR/$BINARY_NAME"
    else
        print_error "Build failed - binary not found"
        exit 1
    fi
    
    # Cleanup
    cd /
    rm -rf "$temp_dir"
}

# Add to PATH if needed
setup_path() {
    # Check if install directory is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        print_warning "$INSTALL_DIR is not in your PATH"
        
        # Determine shell config file
        local shell_config=""
        case "$SHELL" in
            */bash) shell_config="$HOME/.bashrc" ;;
            */zsh)  shell_config="$HOME/.zshrc" ;;
            */fish) shell_config="$HOME/.config/fish/config.fish" ;;
            *)      shell_config="$HOME/.profile" ;;
        esac
        
        print_info "To add $INSTALL_DIR to your PATH, run:"
        echo ""
        echo "  echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> $shell_config"
        echo "  source $shell_config"
        echo ""
        print_info "Or restart your terminal"
    fi
}

# Verify installation
verify_installation() {
    if [[ -x "$INSTALL_DIR/$BINARY_NAME" ]]; then
        print_success "EvalEds installed successfully!"
        
        # Test the binary
        if "$INSTALL_DIR/$BINARY_NAME" --version >/dev/null 2>&1; then
            local version=$("$INSTALL_DIR/$BINARY_NAME" --version 2>/dev/null || echo "unknown")
            print_success "Version: $version"
        fi
        
        print_info "Configuration directory: $CONFIG_DIR"
        print_info "Binary location: $INSTALL_DIR/$BINARY_NAME"
        
        echo ""
        print_info "Get started with:"
        echo "  evaleds create my-first-evaluation --interactive"
        echo "  evaleds --help"
        echo ""
        print_info "Documentation: $REPO_URL#readme"
    else
        print_error "Installation verification failed"
        exit 1
    fi
}

# Main installation function
main() {
    echo ""
    echo -e "${CYAN}🎯 EvalEds Installation Script${NC}"
    echo -e "${CYAN}   AI evaluation platform with PromptEds integration${NC}"
    echo ""
    
    # Parse command line arguments
    local force_build=false
    local install_dir_override=""
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --build-from-source)
                force_build=true
                shift
                ;;
            --install-dir)
                install_dir_override="$2"
                shift 2
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --build-from-source    Force build from source instead of downloading binary"
                echo "  --install-dir DIR      Custom installation directory (default: ~/.local/bin)"
                echo "  --help, -h             Show this help message"
                echo ""
                echo "Environment Variables:"
                echo "  INSTALL_DIR           Installation directory"
                echo "  CONFIG_DIR            Configuration directory"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                print_info "Use --help for usage information"
                exit 1
                ;;
        esac
    done
    
    # Override install directory if specified
    if [[ -n "$install_dir_override" ]]; then
        INSTALL_DIR="$install_dir_override"
    fi
    
    print_status "Installing to: $INSTALL_DIR"
    print_status "Configuration: $CONFIG_DIR"
    
    # Run installation steps
    check_prerequisites
    create_install_dir
    create_config_dir
    
    if [[ "$force_build" == true ]]; then
        build_from_source
    else
        install_binary
    fi
    
    setup_path
    verify_installation
    
    print_success "🚀 EvalEds installation complete!"
}

# Run main function with all arguments
main "$@"