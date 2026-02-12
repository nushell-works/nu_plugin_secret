# Pull Request

## Description
<!-- Provide a brief description of the changes in this PR -->

## Type of Change
<!-- Mark the relevant option with an "x" -->
- [ ] 🐛 Bug fix (non-breaking change that fixes an issue)
- [ ] ✨ New feature (non-breaking change that adds functionality)
- [ ] 💥 Breaking change (fix or feature that causes existing functionality to change)
- [ ] 📚 Documentation update
- [ ] 🔧 Maintenance (refactoring, dependencies, CI/CD)
- [ ] 🔒 Security enhancement

## Security Impact
<!-- Mark the relevant option with an "x" -->
- [ ] ✅ No security implications
- [ ] 🛡️ Maintains existing security properties
- [ ] 🔒 Enhances security
- [ ] ⚠️ Requires security review

### Security Considerations
<!-- If this PR has security implications, describe them here -->

## Testing
<!-- Mark the relevant options with an "x" -->
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Security tests added/updated
- [ ] All tests pass (`cargo test --all-features`)
- [ ] Manual testing completed
- [ ] Performance impact assessed

### Test Coverage
<!-- Describe what you tested and how -->

## Documentation
<!-- Mark the relevant options with an "x" -->
- [ ] Code documentation updated (rustdoc comments)
- [ ] README updated (if applicable)
- [ ] API documentation updated
- [ ] Migration guide updated (if breaking change)
- [ ] Examples added/updated

## Code Quality
<!-- Mark the relevant options with an "x" -->
- [ ] Code follows project [style guidelines](docs/STYLE_GUIDE.md)
- [ ] Self-review completed
- [ ] Quality checks pass (`./scripts/check.sh`)
- [ ] No new clippy warnings
- [ ] rustfmt formatting applied
- [ ] Security checklist reviewed

## Changes Made
<!-- Provide a more detailed description of what was changed -->

### New Features (if applicable)
<!-- List new features added -->

### Bug Fixes (if applicable)
<!-- List bugs fixed -->

### Breaking Changes (if applicable)
<!-- List any breaking changes and migration path -->

## Related Issues
<!-- Link related issues -->
- Closes #<issue_number>
- Relates to #<issue_number>

## Reviewer Notes
<!-- Any specific areas you'd like reviewers to focus on? -->

## Deployment Notes
<!-- Any special considerations for deployment? -->

---

## Pre-submission Checklist
<!-- Mark ALL items with an "x" before submitting -->
- [ ] I have read the [Contributing Guidelines](CONTRIBUTING.md)
- [ ] I have performed a self-review of my code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] Any dependent changes have been merged and published
- [ ] I have checked that this change maintains the security-first principles of the plugin

## Additional Notes
<!-- Add any additional notes for reviewers -->

---

**Security Reminder**: This plugin handles sensitive data. All changes must maintain or enhance security properties. If you're unsure about security implications, please highlight them for review.