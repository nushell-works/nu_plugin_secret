# Contributing to nu_plugin_secret

Thank you for your interest in contributing to nu_plugin_secret! This document provides guidelines and information for contributors.

## 🔐 Security First

This project handles sensitive data and security is our top priority. All contributions must maintain the security-first approach:

- **No Data Exposure**: Secret types must never expose actual content
- **Memory Safety**: All new code must use secure memory practices
- **Constant-Time Operations**: Avoid timing side-channel vulnerabilities
- **Comprehensive Testing**: Security-focused test coverage required

## 🚀 Getting Started

### Prerequisites

- **Rust 1.88.0+** (MSRV - Minimum Supported Rust Version)
- **Nushell 0.109.1+** for testing
- **Git** for version control

### Development Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/nushell-works/nu_plugin_secret.git
   cd nu_plugin_secret
   ```

2. **Install development tools:**
   ```bash
   # Required for quality checks
   rustup component add rustfmt clippy
   cargo install cargo-audit cargo-deny cargo-llvm-cov
   
   # Optional but recommended
   pip install pre-commit
   pre-commit install
   ```

3. **Run initial checks:**
   ```bash
   ./scripts/check.sh
   ```

4. **Install plugin for testing:**
   ```bash
   ./scripts/install-plugin.sh
   ```

## 🛠️ Development Workflow

### Quality Assurance

Before submitting any changes:

```bash
# Run comprehensive quality checks
./scripts/check.sh

# This includes:
# - Code formatting (rustfmt)
# - Linting (clippy)
# - Compilation check
# - All tests (74 tests)
# - Documentation generation
# - Security audit
# - License compliance
```

### Testing

```bash
# Run all tests
cargo test --all-features

# Run specific test category
cargo test secret_string
cargo test commands
cargo test integration

# Test with coverage
cargo llvm-cov --all-features --html --output-dir target/coverage
```

### Local Plugin Testing

```bash
# Build and install plugin
cargo build --release
nu -c "plugin add target/release/nu_plugin_secret"

# Test functionality
nu -c 'echo "test" | secret wrap'
nu -c 'secret info'
```

## 📝 Contribution Guidelines

### Code Style

- **Rust Style**: Follow official Rust formatting (`rustfmt`)
- **Documentation**: All public APIs must have comprehensive docs
- **Comments**: Security-sensitive code requires detailed comments
- **Naming**: Clear, descriptive names that reflect security intent

### Security Requirements

#### For New Secret Types
- Must implement `CustomValue` trait properly
- Must use `ZeroizeOnDrop` for memory safety
- Must implement constant-time equality comparison
- Must display as `<redacted:type>` in all contexts
- Must include comprehensive tests

#### For New Commands
- Must validate input types thoroughly
- Must provide clear error messages (without exposing secrets)
- Must include security warnings for sensitive operations
- Must follow existing command patterns

### Testing Standards

- **Unit Tests**: Every new function/method
- **Integration Tests**: Command functionality with Nushell
- **Security Tests**: Memory safety and no data exposure
- **Edge Cases**: Handle all possible input scenarios
- **Performance**: No significant regression

### Documentation Requirements

- **API Documentation**: All public items documented
- **Security Notes**: Document security implications
- **Examples**: Real-world usage examples
- **Migration Guides**: For breaking changes

## 🐛 Bug Reports

### Security Vulnerabilities

**⚠️ SECURITY ISSUES**: Please report security vulnerabilities privately via GitHub Security Advisories, not public issues.

### Bug Report Template

```markdown
## Bug Description
[Clear description of the issue]

## Steps to Reproduce
1. [First step]
2. [Second step]
3. [Additional steps...]

## Expected Behavior
[What should happen]

## Actual Behavior
[What actually happens]

## Environment
- OS: [e.g., macOS 13.5, Ubuntu 22.04]
- Rust Version: [e.g., 1.75.0]
- Nushell Version: [e.g., 0.109.1]
- Plugin Version: [e.g., 0.1.0]

## Additional Context
[Any other relevant information]
```

## ✨ Feature Requests

### Feature Request Template

```markdown
## Feature Description
[Clear description of the proposed feature]

## Use Case
[Why is this feature needed?]

## Proposed Solution
[How should this be implemented?]

## Security Considerations
[How does this maintain security-first principles?]

## Alternatives Considered
[Other approaches considered]
```

## 🔄 Pull Request Process

### Before Submitting

1. **Run Quality Checks**: `./scripts/check.sh` must pass
2. **Update Documentation**: Include relevant documentation updates
3. **Add Tests**: Comprehensive test coverage for new features
4. **Security Review**: Consider security implications
5. **Performance**: Ensure no significant performance regression

### Pull Request Template

```markdown
## Description
[Brief description of changes]

## Type of Change
- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that causes existing functionality to change)
- [ ] Documentation update

## Security Impact
- [ ] No security implications
- [ ] Maintains existing security properties
- [ ] Enhances security
- [ ] Requires security review

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] All tests pass
- [ ] Manual testing completed

## Documentation
- [ ] Code documentation updated
- [ ] README updated (if applicable)
- [ ] API documentation updated

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Quality checks pass (`./scripts/check.sh`)
- [ ] Security considerations documented
```

### Review Process

1. **Automated Checks**: CI/CD pipeline must pass
2. **Security Review**: All security-related changes reviewed
3. **Code Review**: At least one maintainer approval
4. **Testing**: Comprehensive testing in multiple environments
5. **Documentation**: Documentation completeness verified

## 🏗️ Architecture Guidelines

### Secret Type Implementation

When adding new secret types:

```rust
// Example structure for new secret type
#[derive(Clone, Serialize, Deserialize, ZeroizeOnDrop)]
pub struct SecretNewType {
    #[zeroize(skip)]
    inner: NewType,
}

impl SecretNewType {
    pub fn new(value: NewType) -> Self {
        Self { inner: value }
    }
    
    pub fn reveal(&self) -> &NewType {
        &self.inner
    }
    
    pub fn into_inner(self) -> NewType {
        self.inner.clone()
    }
}

// Required trait implementations
impl CustomValue for SecretNewType { /* ... */ }
impl Display for SecretNewType { /* Always "<redacted:newtype>" */ }
impl Debug for SecretNewType { /* Never expose content */ }
impl PartialEq for SecretNewType { /* Constant-time comparison */ }
```

### Command Implementation

```rust
// Example command structure
#[derive(Clone)]
pub struct SecretWrapNewTypeCommand;

impl PluginCommand for SecretWrapNewTypeCommand {
    fn name(&self) -> &str {
        "secret wrap-newtype"
    }
    
    fn signature(&self) -> Signature {
        // Define input/output types
    }
    
    fn run(&self, /* ... */) -> Result<PipelineData, LabeledError> {
        // Implementation with proper error handling
    }
}
```

## 🎯 Current Priorities

### High Priority
- CI/CD pipeline completion and testing
- Performance benchmarking and optimization
- Security audit preparation
- Documentation improvements

### Medium Priority
- Additional utility commands (`secret select`, `secret get`)
- Advanced security features (audit logging)
- Integration with credential stores

### Future Considerations
- Support for additional Nushell types
- Plugin-to-plugin secret sharing
- Advanced security policies

## 📞 Getting Help

- **Questions**: Open a GitHub Discussion
- **Bugs**: Create a GitHub Issue
- **Security**: Use GitHub Security Advisories
- **Development**: Check existing issues and discussions

## 📄 License

By contributing to nu_plugin_secret, you agree that your contributions will be licensed under the BSD 3-Clause License.

## 🙏 Recognition

All contributors are recognized in our:
- Release notes
- CONTRIBUTORS.md file
- Git history

Thank you for helping make nu_plugin_secret more secure and useful! 🚀