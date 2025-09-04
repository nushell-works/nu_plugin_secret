#!/bin/bash

# nu_plugin_secret Installation Script
#
# This script automatically downloads and installs the latest release
# of nu_plugin_secret for your platform.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# GitHub repository
REPO="nushell-works/nu_plugin_secret"
BINARY_NAME="nu_plugin_secret"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect platform
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)

    case $os in
        linux*)
            case $arch in
                x86_64) echo "x86_64-unknown-linux-gnu" ;;
                aarch64|arm64) echo "aarch64-unknown-linux-gnu" ;;
                *)
                    print_error "Unsupported architecture: $arch"
                    exit 1
                    ;;
            esac
            ;;
        darwin*)
            case $arch in
                x86_64) echo "x86_64-apple-darwin" ;;
                arm64) echo "aarch64-apple-darwin" ;;
                *)
                    print_error "Unsupported architecture: $arch"
                    exit 1
                    ;;
            esac
            ;;
        mingw*|msys*|cygwin*)
            case $arch in
                x86_64) echo "x86_64-pc-windows-msvc" ;;
                *)
                    print_error "Unsupported architecture: $arch"
                    exit 1
                    ;;
            esac
            ;;
        *)
            print_error "Unsupported operating system: $os"
            exit 1
            ;;
    esac
}

# Get the latest release version
get_latest_version() {
    print_status "Fetching latest release information..."
    if command -v curl >/dev/null 2>&1; then
        curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
    else
        print_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
}

# Download and extract the binary
download_and_install() {
    local version=$1
    local platform=$2
    local install_dir=${3:-"$HOME/.cargo/bin"}

    # Determine archive format
    local archive_ext="tar.gz"
    if [[ $platform == *"windows"* ]]; then
        archive_ext="zip"
    fi

    local filename="${BINARY_NAME}-${version}-${platform}.${archive_ext}"
    local download_url="https://github.com/$REPO/releases/download/$version/$filename"

    print_status "Downloading $filename..."

    # Create temporary directory
    local temp_dir=$(mktemp -d)
    trap "rm -rf $temp_dir" EXIT

    # Download
    if command -v curl >/dev/null 2>&1; then
        curl -L "$download_url" -o "$temp_dir/$filename"
    elif command -v wget >/dev/null 2>&1; then
        wget "$download_url" -O "$temp_dir/$filename"
    else
        print_error "Neither curl nor wget found."
        exit 1
    fi

    print_status "Extracting binary..."

    # Extract based on format
    cd "$temp_dir"
    if [[ $archive_ext == "tar.gz" ]]; then
        tar -xzf "$filename"
    elif [[ $archive_ext == "zip" ]]; then
        unzip -q "$filename"
    fi

    # Create install directory if it doesn't exist
    mkdir -p "$install_dir"

    # Copy binary
    local binary_name="$BINARY_NAME"
    if [[ $platform == *"windows"* ]]; then
        binary_name="${BINARY_NAME}.exe"
    fi

    cp "$binary_name" "$install_dir/"
    chmod +x "$install_dir/$binary_name"

    print_success "Binary installed to $install_dir/$binary_name"
}

# Register plugin with Nushell
register_plugin() {
    local install_dir=${1:-"$HOME/.cargo/bin"}
    local binary_path="$install_dir/$BINARY_NAME"

    # Check if nushell is available
    if ! command -v nu >/dev/null 2>&1; then
        print_warning "Nushell (nu) not found in PATH. Please install Nushell first."
        print_status "You can install Nushell from: https://nushell.sh/book/installation.html"
        return 1
    fi

    print_status "Registering plugin with Nushell..."

    # Initialize nushell config directory
    mkdir -p ~/.config/nushell

    # Register the plugin
    if nu -c "plugin add $binary_path"; then
        print_success "Plugin registered successfully!"

        # Activate the plugin
        print_status "Activating plugin..."
        if nu -c "plugin use secret"; then
            print_success "Plugin activated successfully!"
        else
            print_warning "Failed to activate plugin. You may need to run 'plugin use secret' manually."
        fi

        # Test basic functionality
        print_status "Testing basic functionality..."
        if nu -c 'echo "test" | secret wrap' >/dev/null 2>&1; then
            print_success "Plugin is working correctly!"
        else
            print_warning "Plugin test failed. Please check your installation."
        fi
    else
        print_error "Failed to register plugin with Nushell."
        print_status "Try running manually: nu -c 'plugin add $binary_path'"
        return 1
    fi
}

# Main installation function
main() {
    echo "🔐 nu_plugin_secret Installation Script"
    echo "======================================"
    echo

    # Parse command line arguments
    local install_dir="$HOME/.cargo/bin"
    local skip_register=false

    while [[ $# -gt 0 ]]; do
        case $1 in
            --install-dir)
                install_dir="$2"
                shift 2
                ;;
            --skip-register)
                skip_register=true
                shift
                ;;
            --help)
                cat << EOF
Usage: $0 [OPTIONS]

OPTIONS:
    --install-dir DIR    Install to specified directory (default: $HOME/.cargo/bin)
    --skip-register      Skip automatic plugin registration with Nushell
    --help              Show this help message

EXAMPLES:
    $0                                    # Install to default location and register
    $0 --install-dir /usr/local/bin       # Install to system-wide location
    $0 --skip-register                    # Install but don't register with Nushell

REQUIREMENTS:
    - curl or wget
    - tar (on Unix) or unzip (on Windows)
    - Nushell (for plugin registration)

EOF
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                print_status "Use --help for usage information"
                exit 1
                ;;
        esac
    done

    # Detect platform
    print_status "Detecting platform..."
    local platform=$(detect_platform)
    print_success "Detected platform: $platform"

    # Get latest version
    local version=$(get_latest_version)
    if [[ -z "$version" ]]; then
        print_error "Failed to get latest version information"
        exit 1
    fi
    print_success "Latest version: $version"

    # Download and install
    download_and_install "$version" "$platform" "$install_dir"

    # Add to PATH if not already there
    if [[ ":$PATH:" != *":$install_dir:"* ]]; then
        print_warning "$install_dir is not in your PATH."
        print_status "Add this to your shell profile: export PATH=\"$install_dir:\$PATH\""
    fi

    # Register with Nushell unless skipped
    if [[ "$skip_register" == "false" ]]; then
        register_plugin "$install_dir"
    else
        print_status "Skipping plugin registration (--skip-register specified)"
        print_status "To register manually, run: nu -c 'plugin add $install_dir/$BINARY_NAME'"
    fi

    echo
    print_success "Installation completed successfully! 🎉"
    echo
    echo "Next steps:"
    echo "1. Restart your terminal or source your shell profile"
    echo "2. Run 'nu' to start Nushell"
    echo "3. Try: echo \"secret\" | secret wrap"
    echo
    echo "For more information, visit: https://github.com/$REPO"
}

# Check if script is being sourced or executed
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
