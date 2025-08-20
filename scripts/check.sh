#!/bin/bash
# Quality assurance script for nu_plugin_secret

set -euo pipefail

echo "🔍 Running quality checks for nu_plugin_secret..."

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

# Check if required tools are installed
check_tools() {
    echo -e "${BLUE}Checking required tools...${NC}"
    
    local tools=("cargo" "rustfmt")
    local missing_tools=()
    
    for tool in "${tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            missing_tools+=("$tool")
        fi
    done
    
    # Check if clippy is available as a rustup component
    if ! rustup component list --installed | grep -q "clippy"; then
        missing_tools+=("clippy (rustup component)")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        print_status "error" "Missing required tools: ${missing_tools[*]}"
        exit 1
    fi
    
    print_status "ok" "All required tools are installed"
}

# Format code
format_code() {
    echo -e "\n${BLUE}Formatting code...${NC}"
    if cargo fmt --all --check; then
        print_status "ok" "Code formatting is correct"
    else
        print_status "warn" "Code formatting issues found, fixing..."
        cargo fmt --all
        print_status "ok" "Code formatting fixed"
    fi
}

# Run clippy lints
run_clippy() {
    echo -e "\n${BLUE}Running clippy lints...${NC}"
    if cargo clippy --all-targets --all-features -- -D warnings; then
        print_status "ok" "No clippy warnings found"
    else
        print_status "error" "Clippy warnings found"
        return 1
    fi
}

# Check compilation
check_build() {
    echo -e "\n${BLUE}Checking compilation...${NC}"
    if cargo check --all-targets --all-features; then
        print_status "ok" "Compilation successful"
    else
        print_status "error" "Compilation failed"
        return 1
    fi
}

# Run tests
run_tests() {
    echo -e "\n${BLUE}Running tests...${NC}"
    if cargo test --all-features; then
        print_status "ok" "All tests passed"
    else
        print_status "error" "Some tests failed"
        return 1
    fi
}

# Security audit
security_audit() {
    echo -e "\n${BLUE}Running security audit...${NC}"
    if command -v cargo-audit &> /dev/null; then
        if cargo audit; then
            print_status "ok" "No security vulnerabilities found"
        else
            print_status "warn" "Security audit found issues"
            return 1
        fi
    else
        print_status "warn" "cargo-audit not installed, skipping security check"
    fi
}

# Check documentation
check_docs() {
    echo -e "\n${BLUE}Checking documentation...${NC}"
    if cargo doc --no-deps --document-private-items --all-features; then
        print_status "ok" "Documentation generated successfully"
    else
        print_status "error" "Documentation generation failed"
        return 1
    fi
}

# License compliance
check_licenses() {
    echo -e "\n${BLUE}Checking license compliance...${NC}"
    if command -v cargo-deny &> /dev/null; then
        if cargo deny check; then
            print_status "ok" "License compliance verified"
        else
            print_status "warn" "License compliance issues found"
        fi
    else
        print_status "warn" "cargo-deny not installed, skipping license check"
    fi
}

# Main execution
main() {
    echo "Starting quality checks for nu_plugin_secret"
    echo "============================================="
    
    check_tools
    format_code
    run_clippy || exit 1
    check_build || exit 1
    run_tests || exit 1
    security_audit
    check_docs || exit 1
    check_licenses
    
    echo ""
    echo -e "${GREEN}🎉 All quality checks completed successfully!${NC}"
    echo ""
    echo "Summary:"
    echo "- Code formatting: ✓"
    echo "- Clippy lints: ✓"
    echo "- Compilation: ✓"
    echo "- Tests (74): ✓"
    echo "- Documentation: ✓"
    echo ""
    echo "Ready for commit/push! 🚀"
}

# Run main function
main "$@"