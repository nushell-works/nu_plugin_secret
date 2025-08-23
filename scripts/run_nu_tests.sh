#!/bin/bash
# Cross-platform Nushell test runner for nu_plugin_secret
# Supports Linux, macOS, and Windows (via WSL/Git Bash)

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEST_DIR="$PROJECT_ROOT/tests/nushell"
PLUGIN_BINARY="$PROJECT_ROOT/target/release/nu_plugin_secret"

# Default values
SUITE="all"
VERBOSE=""
PARALLEL=""
TIMEOUT=""
FORMAT="detailed"
OUTPUT=""
SETUP_ONLY=""
CLEANUP_ONLY=""
NU_CMD="nu"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_color() {
    color=$1
    shift
    echo -e "${color}$*${NC}"
}

print_error() {
    print_color $RED "❌ ERROR: $*" >&2
}

print_warning() {
    print_color $YELLOW "⚠️  WARNING: $*"
}

print_info() {
    print_color $BLUE "ℹ️  $*"
}

print_success() {
    print_color $GREEN "✅ $*"
}

# Show usage information
show_usage() {
    cat << EOF
nu_plugin_secret Nushell Test Runner

Usage: $0 [OPTIONS]

OPTIONS:
    -s, --suite SUITE       Test suite to run (all, commands, integration, security, performance)
    -v, --verbose          Enable verbose output
    -p, --parallel JOBS    Number of parallel jobs (default: 1)
    -t, --timeout DURATION Test timeout (default: 30sec)
    -f, --format FORMAT    Report format (summary, detailed, json)
    -o, --output FILE      Output file for results
    --setup-only           Only run setup, don't run tests
    --cleanup-only         Only run cleanup
    --nu-path PATH         Path to nu executable (default: nu)
    -h, --help             Show this help message

EXAMPLES:
    $0                                    # Run all tests
    $0 -s commands -v                     # Run command tests with verbose output
    $0 -s integration -p 4                # Run integration tests with 4 parallel jobs
    $0 --setup-only                       # Only setup the plugin
    $0 --cleanup-only                     # Only cleanup test environment
    
ENVIRONMENT VARIABLES:
    NU_PLUGIN_SECRET_TEST_TIMEOUT    Default test timeout
    NU_PLUGIN_SECRET_TEST_PARALLEL   Default parallel jobs
    CI                              Set to 'true' for CI environment
EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -s|--suite)
                SUITE="$2"
                shift 2
                ;;
            -v|--verbose)
                VERBOSE="--verbose"
                shift
                ;;
            -p|--parallel)
                PARALLEL="--parallel $2"
                shift 2
                ;;
            -t|--timeout)
                TIMEOUT="--timeout $2"
                shift 2
                ;;
            -f|--format)
                FORMAT="--format $2"
                shift 2
                ;;
            -o|--output)
                OUTPUT="--output $2"
                shift 2
                ;;
            --setup-only)
                SETUP_ONLY="--setup-only"
                shift
                ;;
            --cleanup-only)
                CLEANUP_ONLY="--cleanup-only"
                shift
                ;;
            --nu-path)
                NU_CMD="$2"
                shift 2
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Detect operating system
detect_os() {
    case "$(uname -s)" in
        Linux*)     echo "linux";;
        Darwin*)    echo "macos";;
        CYGWIN*|MINGW*|MSYS*) echo "windows";;
        *)          echo "unknown";;
    esac
}

# Install Nushell if not present
install_nushell() {
    local os=$(detect_os)
    
    if command_exists $NU_CMD; then
        print_success "Nushell found at $(command -v $NU_CMD)"
        return 0
    fi
    
    print_info "Nushell not found. Attempting to install..."
    
    case $os in
        linux)
            if command_exists curl; then
                print_info "Installing Nushell for Linux..."
                curl -L https://github.com/nushell/nushell/releases/latest/download/nu-linux-x86_64.tar.gz | tar xz
                sudo cp nu*/nu /usr/local/bin/
                rm -rf nu*
            else
                print_error "curl not found. Please install Nushell manually."
                return 1
            fi
            ;;
        macos)
            if command_exists brew; then
                print_info "Installing Nushell via Homebrew..."
                brew install nushell
            elif command_exists curl; then
                print_info "Installing Nushell for macOS..."
                curl -L https://github.com/nushell/nushell/releases/latest/download/nu-darwin-x86_64.tar.gz | tar xz
                sudo cp nu*/nu /usr/local/bin/
                rm -rf nu*
            else
                print_error "Neither brew nor curl found. Please install Nushell manually."
                return 1
            fi
            ;;
        windows)
            print_error "Please install Nushell manually on Windows."
            print_info "Download from: https://github.com/nushell/nushell/releases"
            return 1
            ;;
        *)
            print_error "Unsupported operating system: $os"
            return 1
            ;;
    esac
    
    if command_exists $NU_CMD; then
        print_success "Nushell installed successfully"
        return 0
    else
        print_error "Failed to install Nushell"
        return 1
    fi
}

# Check prerequisites
check_prerequisites() {
    print_info "Checking prerequisites..."
    
    # Check if we're in the correct directory
    if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        print_error "Not in a Rust project directory. Expected to find Cargo.toml"
        exit 1
    fi
    
    # Check for cargo
    if ! command_exists cargo; then
        print_error "Cargo not found. Please install Rust and Cargo."
        exit 1
    fi
    
    # Check/install Nushell
    if ! install_nushell; then
        exit 1
    fi
    
    # Verify Nushell version
    local nu_version=$($NU_CMD --version 2>/dev/null || echo "unknown")
    print_info "Using Nushell: $nu_version"
    
    # Check test directory structure
    if [[ ! -d "$TEST_DIR" ]]; then
        print_error "Test directory not found: $TEST_DIR"
        exit 1
    fi
    
    print_success "Prerequisites check passed"
}

# Build the plugin
build_plugin() {
    print_info "Building nu_plugin_secret..."
    
    cd "$PROJECT_ROOT"
    
    if [[ -n "$CI" ]]; then
        # CI environment
        cargo build --release --quiet
    else
        # Local development
        if ! cargo build --release; then
            print_error "Failed to build plugin"
            exit 1
        fi
    fi
    
    if [[ ! -f "$PLUGIN_BINARY" ]]; then
        print_error "Plugin binary not found after build: $PLUGIN_BINARY"
        exit 1
    fi
    
    print_success "Plugin built successfully"
}

# Run the tests
run_tests() {
    print_info "Starting Nushell tests..."
    
    cd "$TEST_DIR"
    
    # Use simple test - no arguments needed
    if [[ -n "$SETUP_ONLY" ]]; then
        print_info "Running setup only..."
        local nu_args="-c 'use setup.nu; setup setup_plugin'"
    elif [[ -n "$CLEANUP_ONLY" ]]; then
        print_info "Running cleanup only..."
        local nu_args="-c 'use setup.nu; setup cleanup_plugin'"
    else
        print_info "Running simple test suite..."
        local nu_args="simple_test.nu"
    fi
    
    print_info "Running: $NU_CMD $nu_args"
    
    # Run the tests
    if $NU_CMD $nu_args; then
        print_success "Tests completed successfully"
        return 0
    else
        print_error "Tests failed"
        return 1
    fi
}

# Main execution
main() {
    parse_args "$@"
    
    print_color $BLUE "🧪 nu_plugin_secret Nushell Test Runner"
    print_color $BLUE "======================================="
    
    # Set environment variables
    export NU_PLUGIN_SECRET_TEST_TIMEOUT=${NU_PLUGIN_SECRET_TEST_TIMEOUT:-30sec}
    export NU_PLUGIN_SECRET_TEST_PARALLEL=${NU_PLUGIN_SECRET_TEST_PARALLEL:-1}
    
    # Skip building if only cleanup
    if [[ -z "$CLEANUP_ONLY" ]]; then
        check_prerequisites
        build_plugin
    fi
    
    # Run tests
    if run_tests; then
        print_success "All operations completed successfully"
        exit 0
    else
        print_error "Operations failed"
        exit 1
    fi
}

# Run main function
main "$@"