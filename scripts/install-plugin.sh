#!/bin/bash
# Installation script for nu_plugin_secret

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    local status=$1
    local message=$2
    if [ "$status" = "ok" ]; then
        echo -e "${GREEN}✓${NC} $message"
    elif [ "$status" = "warn" ]; then
        echo -e "${YELLOW}⚠${NC} $message"
    else
        echo -e "${RED}✗${NC} $message"
    fi
}

# Check if Nushell is installed
check_nushell() {
    echo -e "${BLUE}Checking Nushell installation...${NC}"
    if command -v nu &> /dev/null; then
        local nu_version
        nu_version=$(nu --version | head -n1 | cut -d' ' -f2)
        print_status "ok" "Nushell $nu_version found"
    else
        print_status "error" "Nushell not found. Please install Nushell first."
        echo "Visit: https://www.nushell.sh/book/installation.html"
        exit 1
    fi
}

# Build the plugin
build_plugin() {
    echo -e "\n${BLUE}Building nu_plugin_secret...${NC}"
    
    local build_mode="release"
    local target_dir="target/release"
    
    if [[ "${1:-}" == "--debug" ]]; then
        build_mode="debug"
        target_dir="target/debug"
        print_status "warn" "Building in debug mode"
    fi
    
    if [ "$build_mode" = "release" ]; then
        if cargo build --release; then
            print_status "ok" "Plugin built successfully in release mode"
        else
            print_status "error" "Failed to build plugin"
            exit 1
        fi
    else
        if cargo build; then
            print_status "ok" "Plugin built successfully in debug mode"
        else
            print_status "error" "Failed to build plugin"
            exit 1
        fi
    fi
    
    echo "BUILD_MODE=$build_mode" > .build_mode
    echo "TARGET_DIR=$target_dir" >> .build_mode
}

# Register plugin with Nushell
register_plugin() {
    echo -e "\n${BLUE}Registering plugin with Nushell...${NC}"
    
    # Read build configuration
    if [ -f .build_mode ]; then
        source .build_mode
    else
        TARGET_DIR="target/release"
    fi
    
    local plugin_path="$PWD/$TARGET_DIR/nu_plugin_secret"
    
    if [ ! -f "$plugin_path" ]; then
        print_status "error" "Plugin binary not found at $plugin_path"
        exit 1
    fi
    
    # Check if plugin is already registered
    if nu -c "plugin list" 2>/dev/null | grep -q "secret"; then
        print_status "warn" "Plugin already registered, updating..."
        nu -c "plugin rm secret" 2>/dev/null || true
    fi
    
    # Register the plugin
    if nu -c "plugin add '$plugin_path'"; then
        print_status "ok" "Plugin registered successfully"
    else
        print_status "error" "Failed to register plugin"
        exit 1
    fi
    
    # Activate the plugin
    if nu -c "plugin use secret"; then
        print_status "ok" "Plugin activated"
    else
        print_status "warn" "Plugin registered but activation may require restart"
    fi
}

# Test plugin functionality
test_plugin() {
    echo -e "\n${BLUE}Testing plugin functionality...${NC}"
    
    local tests=(
        'echo "test" | secret wrap-string'
        '42 | secret wrap-int'
        'true | secret wrap-bool'
        '3.14159 | secret wrap-float'
        'date now | secret wrap-date'
        'secret info'
    )
    
    for test_cmd in "${tests[@]}"; do
        if nu -c "$test_cmd" >/dev/null 2>&1; then
            print_status "ok" "Test passed: $test_cmd"
        else
            print_status "error" "Test failed: $test_cmd"
            return 1
        fi
    done
    
    print_status "ok" "All functionality tests passed"
}

# Show usage examples
show_examples() {
    echo -e "\n${BLUE}Usage Examples:${NC}"
    echo ""
    echo "# Wrap different types of secrets:"
    echo 'echo "my-api-key" | secret wrap-string'
    echo '42 | secret wrap-int'
    echo 'true | secret wrap-bool'
    echo '3.14159 | secret wrap-float'
    echo '0x[deadbeef] | secret wrap-binary'
    echo 'date now | secret wrap-date'
    echo ""
    echo "# Utility commands:"
    echo '$secret | secret unwrap          # Extract value (with warning)'
    echo '$secret | secret validate        # Check if secret type'
    echo '$secret | secret type-of         # Get underlying type'
    echo 'secret info                     # Plugin information'
    echo ""
    echo "# Real-world usage:"
    echo 'let $api_key = ($env.API_KEY | secret wrap-string)'
    echo 'http get "https://api.example.com" -H [Authorization $"Bearer ($api_key | secret unwrap)"]'
    echo ""
}

# Main execution
main() {
    echo "nu_plugin_secret Installation Script"
    echo "===================================="
    echo ""
    
    check_nushell
    build_plugin "$@"
    register_plugin
    test_plugin
    
    echo ""
    echo -e "${GREEN}🎉 Installation completed successfully!${NC}"
    echo ""
    echo "The nu_plugin_secret plugin has been installed and is ready to use."
    echo "You now have access to 8 secret types and 12 commands for secure data handling."
    
    show_examples
    
    echo ""
    echo -e "${BLUE}For more information:${NC}"
    echo "- Documentation: cargo doc --open"
    echo "- Repository: https://github.com/nushell-works/nu_plugin_secret"
    echo "- Plugin info: nu -c 'secret info'"
}

# Show help
if [[ "${1:-}" == "--help" ]] || [[ "${1:-}" == "-h" ]]; then
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --debug    Build in debug mode instead of release"
    echo "  --help     Show this help message"
    echo ""
    echo "This script will:"
    echo "1. Check Nushell installation"
    echo "2. Build the nu_plugin_secret plugin"
    echo "3. Register it with Nushell"
    echo "4. Test basic functionality"
    exit 0
fi

# Run main function
main "$@"