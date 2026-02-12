# Style Guide

Coding conventions for the nu_plugin_secret project. Each item has a unique ID for easy reference.

## STYLE-0000: Style guide structure

**Tags:** `meta`

### Situation

A new convention needs to be added to this style guide.

### Guidance

Assign the next sequential ID (currently next is `STYLE-0013`) and include:

1. A **Tags** line immediately after the heading — a comma-separated list of category labels
   from the tag vocabulary below.
2. Three subheadings:
   - **Situation** — when this rule applies
   - **Guidance** — what to do (with examples where helpful)
   - **Motivation** — why this rule exists

**Tag vocabulary** (extend as needed):

| Tag                  | Covers                                             |
|----------------------|----------------------------------------------------|
| `meta`               | Style guide structure and process                  |
| `error-handling`     | Error types, context messages, panics, suppression |
| `module-organization`| File layout, visibility, cohesion                  |
| `naming`             | Naming conventions for types, functions, files     |
| `commits`            | Commit message format, scope rules, discipline     |
| `documentation`      | Doc comments, examples                             |
| `testing`            | Test structure, fixtures, snapshots                |
| `code-style`         | Imports, clippy, constants, function length        |
| `api-design`         | Ownership, string params                           |
| `unsafe`             | Unsafe code policy                                 |

A rule may have **multiple tags** — e.g., a rule about error messages in tests could be
tagged `error-handling, testing`.

Items are ordered by ID. **Do not** group items under section headings; use tags for
categorisation instead.

### Motivation

Consistent structure makes the guide scannable, and stable IDs allow code review comments
and ADRs to reference specific rules unambiguously. Tags replace section headings so that
items can remain in strict ID order without needing to be shuffled between sections when
categories overlap or new categories are introduced.

---

## STYLE-0001: Panicking operations

**Tags:** `error-handling`

### Situation

Considering `unwrap()`, `expect()`, or other panicking calls.

### Guidance

**`unwrap()` is acceptable** in these cases only:

- **Known-safe constructors** — `FixedOffset::east_opt(0).unwrap()` where the argument is
  a constant that cannot fail.
- **Test code** — tests may use `unwrap()` freely.

**`expect()` is acceptable** for truly catastrophic I/O that should terminate the process:

```rust
io::stdout().flush().expect("Failed to flush stdout");
```

**Prefer `unwrap_or_else` or `unwrap_or`** when a sensible fallback exists. This is the
dominant pattern in the codebase:

```rust
// Good — graceful fallback with context
let config_manager = ConfigManager::load().unwrap_or_else(|_| {
    ConfigManager::new(PluginConfig::default())
});
```

**Never** use `unwrap()` or `expect()` on user-supplied or runtime data in library code.
Use `?` with appropriate error conversion instead.

### Motivation

Panics in library code produce poor diagnostics and cannot be handled by callers. Limiting
panics to provably-safe or catastrophic cases keeps the error surface predictable. Preferring
`unwrap_or_else` with fallbacks keeps the plugin resilient to configuration or I/O failures.

---

## STYLE-0002: Naming patterns

**Tags:** `naming`

### Situation

Naming a new type, function, CLI command, or constant.

### Guidance

| Element           | Convention            | Examples                                         |
|-------------------|-----------------------|--------------------------------------------------|
| Structs / Enums   | PascalCase            | `SecretPlugin`, `ConfigError`, `SecurityLevel`   |
| Traits            | PascalCase (adj/verb) | `PluginCommand`, `Serialize`, `CustomValue`      |
| Functions/Methods | snake_case            | `init_redaction_templating()`, `reveal()`        |
| Type aliases      | PascalCase            | `Result<T>` (for crate-local aliases)            |
| Constants         | UPPER_SNAKE_CASE      | `REDACTION_TEMPLATE`, `TEMPLATE_NAME`            |
| CLI commands      | kebab-case            | `secret wrap`, `secret wrap-with`                |
| Modules / files   | snake_case            | `secret_string.rs`, `config.rs`                  |

### Motivation

Standard Rust naming (`PascalCase` types, `snake_case` functions) is enforced by compiler
warnings and `clippy`. Kebab-case CLI commands follow Nushell conventions and are standard
across Unix tools.

---

## STYLE-0003: Commit message format

**Tags:** `commits`

### Situation

Writing a commit message.

### Guidance

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

**Types:** `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `ci`, `perf`,
`build`.

**Scopes** (use the most specific that applies):

| Scope           | Covers                                       |
|-----------------|----------------------------------------------|
| `core`          | Plugin registration, command dispatch        |
| `commands`      | Individual command implementations           |
| `secret-types`  | Secret type definitions and behaviour        |
| `config`        | Configuration system and validation          |
| `redaction`     | Redaction templating system                  |
| `tera-functions`| Custom Tera template functions               |
| `templates`     | Template engine integration                  |
| `memory`        | Memory optimisation and zeroization          |
| `ci`            | CI/CD workflows and configuration            |
| `deps`          | Dependency updates                           |
| `docs`          | Documentation                                |
| `release`       | Version bumps, release process               |

**Multi-scope commits** — when a change touches multiple scopes equally, list them
comma-separated: `refactor(redaction,templates): extract Tera functions`.

**Subject line rules:**

- Lowercase first word (no capital after the colon)
- No trailing period
- Imperative mood ("add feature" not "added feature")
- Under 72 characters total

### Motivation

Conventional commits produce machine-readable history that enables automated changelogs,
version bumping, and filtering by scope. Consistent subject lines make `git log --oneline`
scannable.

---

## STYLE-0004: Doc comments

**Tags:** `documentation`

### Situation

Adding or updating documentation on a module, type, or function.

### Guidance

**Module-level docs** — every module file starts with a `//!` comment:

```rust
//! Tera-based redaction templating system
```

**Item-level docs** — every public struct, enum, field, variant, and method gets `///`:

```rust
/// Errors that can occur during configuration operations
pub enum ConfigError {
    /// IO error during config file access
    Io(#[from] std::io::Error),
    /// TOML parsing error in config file
    TomlParse(#[from] toml::de::Error),
}
```

**Summary line style** — write in **third-person singular present indicative** per
[RFC 505](https://rust-lang.github.io/rfcs/0505-api-comment-conventions.html). Use full
sentences ending with a period:

```rust
/// Initializes the Tera template engine for redaction.
pub fn init_redaction_templating() -> Result<(), tera::Error> { ... }

/// Returns `true` if the input matches the secret's content.
pub fn contains(&self, needle: &str) -> bool { ... }
```

| Correct (third-person)         | Incorrect (imperative)        |
|--------------------------------|-------------------------------|
| `/// Returns the length.`      | `/// Return the length.`      |
| `/// Creates a new client.`    | `/// Create a new client.`    |
| `/// Parses the input string.` | `/// Parse the input string.` |

### Motivation

The third-person convention matches the Rust standard library and `rustdoc` output, where
doc summaries read as descriptions of what the item *does* (e.g., `Vec::push` — "Appends
an element to the back of a collection."). RFC 505 codifies this as the official Rust API
documentation style.

---

## STYLE-0005: Import ordering

**Tags:** `code-style`

### Situation

Adding `use` statements to a file.

### Guidance

Group imports into three blocks separated by a blank line, in this order:

1. **Standard library** (`std`, `core`, `alloc`)
2. **External crates** (everything from `Cargo.toml` dependencies)
3. **Crate-internal** (`crate::`, `super::`, `self::`)

Within each group, let `cargo fmt` sort alphabetically.

```rust
use std::fmt;
use std::sync::OnceLock;

use nu_protocol::{ShellError, Span, Value};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::config::RedactionContext;
use crate::redaction;
```

**Enforcement note:** The rustfmt option `group_imports = "StdExternalCrate"` that codifies
this convention is still unstable. The three-group ordering is therefore a manual discipline
— `cargo fmt` will sort *within* a group but will not insert or enforce the blank-line
separators between groups. Review for this during code review.

### Motivation

Grouped imports make it easy to see at a glance what a module depends on externally versus
internally. The three-group convention is widely used in the Rust ecosystem. Alphabetical
ordering within groups is enforced by `cargo fmt`.

---

## STYLE-0006: Clippy configuration

**Tags:** `code-style`

### Situation

Configuring or overriding Clippy lints.

### Guidance

Run Clippy with `-D warnings` in CI so that lint violations fail the build.

When suppressing a lint on a specific item, use `#[allow(clippy::...)]` with a justification
comment explaining why the suppression is necessary:

```rust
#[allow(clippy::too_many_arguments)] // Builder pattern requires all fields at construction
fn new(title: &str, description: &str, ...) -> Self { ... }
```

Do not add blanket `#[allow(...)]` at module or crate level to silence warnings. Fix the
warning or suppress it at the narrowest possible scope.

### Motivation

`-D warnings` ensures lint violations are caught in CI. Requiring justification comments on
suppressions ensures each override is a deliberate decision rather than a way to silence
noise. Narrow-scope suppression prevents accidentally disabling a lint for unrelated code.

---

## STYLE-0007: Unsafe policy

**Tags:** `unsafe`, `code-style`

### Situation

Considering the use of `unsafe` code.

### Guidance

This project should not require `unsafe` code. If `unsafe` is ever needed, it must be:

1. Justified in an ADR
2. Isolated in a dedicated module
3. Annotated with a `// SAFETY:` comment per Clippy's `undocumented_unsafe_blocks` lint

### Motivation

This project has no need for `unsafe` — it delegates low-level operations to well-audited
dependencies (`zeroize`, `chrono`, `sha2`, `blake3`, `tera`). Requiring an ADR for any
exception ensures the decision is reviewed and documented.

---

## STYLE-0008: String parameter ownership

**Tags:** `api-design`

### Situation

Deciding whether a function parameter should be `&str`, `String`, or generic.

### Guidance

Use the cheapest type that satisfies the function's needs:

| The function…                          | Accept              | Example                                      |
|----------------------------------------|---------------------|----------------------------------------------|
| Only reads the string                  | `&str`              | `fn contains(&self, needle: &str) -> bool`   |
| Stores the string in a struct/`Vec`    | `String`            | `fn new(inner: String) -> Self`              |
| Needs flexibility (public API surface) | `impl Into<String>` | `fn new(name: impl Into<String>) -> Self`    |

Prefer `&str` for internal helpers and `impl Into<String>` sparingly — only at public API
boundaries where caller ergonomics justify the generic. Avoid `impl AsRef<str>` unless you
genuinely need to accept both `String` and `&str` without conversion.

For return types, prefer `&str` when returning a reference to owned data, and `String` when
returning a newly constructed value. Avoid `Cow<'_, str>` unless profiling shows the
borrow-or-own flexibility is needed.

```rust
// Good — borrows for read-only access
pub fn reveal(&self) -> &str {
    &self.inner
}

// Good — takes ownership because it stores the value
pub fn new(inner: String) -> Self {
    Self { inner, redaction_template: None }
}

// Good — constructs a new string
pub fn redacted_display(&self) -> String {
    format!("<redacted:{}>", self.secret_type())
}
```

### Motivation

Accepting `&str` avoids unnecessary allocations on the caller side. Taking `String` when
ownership is needed makes the transfer explicit and avoids hidden `.to_string()` calls
inside the function. The `impl Into<String>` pattern is convenient for public APIs but adds
monomorphisation cost, so it should be used judiciously.

---

## STYLE-0009: Named constants

**Tags:** `code-style`, `naming`

### Situation

Using a numeric or string literal whose meaning is not obvious from surrounding context.

### Guidance

Extract **magic literals** into named constants or `const` items. A literal is "magic" when its
purpose is not self-evident at the usage site:

```rust
// Bad — what does this string mean?
if template == "<redacted:{{secret_type}}>" {

// Good — the name documents the intent
const REDACTION_TEMPLATE: &str = "<redacted:{{secret_type}}>";
if template == REDACTION_TEMPLATE {
```

Literals that do **not** need extraction:

- **Structural zeros and ones** — `Vec::with_capacity(1)`, `index + 1`, `slice[0]`.
- **Format strings** — `format!("{}: {}", key, value)`.
- **Known-safe constructor arguments** — `FixedOffset::east_opt(0)` (covered by STYLE-0001).
- **Test assertions** — `assert_eq!(result.len(), 3)` where the value is local to the test.
- **Serialization field counts** — `serializer.serialize_struct("SecretString", 2)` where the
  count is immediately obvious from context.

Place constants at the narrowest useful scope: module-level `const` if used across functions in
the same module, crate-level if shared across modules, or function-local `const` if truly local.

### Motivation

Named constants make the code self-documenting and provide a single point of change when a value
needs updating. Searching for `REDACTION_TEMPLATE` finds every usage; searching for the raw
template string returns false positives. The exceptions prevent over-extraction of trivially
obvious values.

---

## STYLE-0010: Function length

**Tags:** `code-style`

### Situation

Writing or reviewing a function that is growing long.

### Guidance

Keep functions **under ~50 lines** of logic (excluding doc comments, blank lines, and closing
braces). When a function exceeds this guideline, look for opportunities to extract coherent
sub-operations into well-named helper functions.

Common extraction targets:

- **Setup / teardown** — initialising the Tera engine, loading configuration.
- **Distinct phases** — validation, transformation, output formatting.
- **Repeated patterns** — similar blocks that differ only in parameters.
- **Match arms** — large `match` blocks over `Value` variants (common in command `run` methods).

```rust
// Before — long run() mixing validation, type dispatch, and wrapping
fn run(&self, ...) -> Result<PipelineData, LabeledError> {
    // ... 100+ lines ...
}

// After — orchestrator delegates to focused helpers
fn run(&self, ...) -> Result<PipelineData, LabeledError> {
    let input = self.extract_input(&call)?;
    let secret = self.wrap_value(input)?;
    Ok(secret.into_pipeline_data())
}
```

This is a **guideline, not a hard limit**. A 60-line function that reads linearly may be clearer
than three 20-line functions with non-obvious data flow. Use judgement — the goal is readability,
not a line count.

### Motivation

Long functions are harder to name, test, and review. Extracting sub-operations gives each piece
a name that serves as documentation and makes the top-level flow scannable. The ~50-line
heuristic is a common industry threshold (Clean Code, Effective Rust) that balances granularity
against fragmentation.

---

## STYLE-0011: Single-purpose commits

**Tags:** `commits`

### Situation

Preparing a set of changes that involves refactoring, new functionality, or bug fixes.

### Guidance

Each commit should do **one kind of work**. Keep refactoring commits separate from
implementation commits, and both separate from bug-fix commits.

If a refactoring would make a subsequent implementation or fix cleaner, land the refactoring
as an **earlier** commit so that:

1. The refactoring can be reviewed on its own terms (no behaviour change expected).
2. The implementation commit starts from a cleaner baseline and is easier to understand.
3. Either commit can be reverted independently if needed.

```
# Good — reviewable, bisectable, revertible
git log --oneline
a1b2c3  refactor(templates): extract Tera functions into dedicated module
d4e5f6  feat(tera-functions): add mask_partial template function

# Bad — mixed intent, hard to review or revert half of it
f7g8h9  feat(tera-functions): add mask_partial and refactor template engine
```

**Acceptable exceptions:**

- Trivial renames or import cleanups that are a natural by-product of the implementation
  (a few lines, not a standalone refactoring effort).
- Prototype or spike branches where commit hygiene is deferred to a squash before merge.

### Motivation

Single-purpose commits make `git bisect` reliable, code review focused, and reverts
surgical. When refactoring is interleaved with behaviour changes, reviewers cannot tell
whether a difference is a deliberate new behaviour or a mechanical restructuring — so they
must verify every line as if it were new logic. Separating the two cuts review effort
roughly in half.

---

## STYLE-0012: Module cohesion

**Tags:** `module-organization`

### Situation

A source file is accumulating types, functions, or `impl` blocks that serve unrelated
purposes.

### Guidance

Each module should have a **single, nameable responsibility**. When you find it hard to
describe what a module does without using "and," it likely contains unrelated code that
would be clearer in separate modules.

**Signals that a module should be split:**

- It contains multiple independent command or handler types that share little or no private
  state (e.g., `SecretConfigExportCommand` and `SecretHashCommand` in one file).
- Unrelated sections require scanning past hundreds of lines to find the piece you need.
- Changes to one logical area routinely cause merge conflicts with work in another area of
  the same file.
- You struggle to name the file — broad names like `commands.rs` or `helpers.rs` suggest
  mixed responsibilities.

**What is *not* a reason to split:**

- Line count alone. A 400-line module with a single cohesive type and its helpers is fine.
- A few shared utility functions that genuinely serve every type in the module.

### Motivation

A module that mixes unrelated responsibilities is hard to navigate, produces noisy diffs,
and invites merge conflicts between independent work streams. Splitting by responsibility
makes each file's purpose obvious from its name, keeps diffs focused on the change at hand,
and lets reviewers evaluate one concern at a time. The emphasis on cohesion rather than a
rigid line limit avoids unnecessary churn on files that are large but focused, while still
flagging files that are large *because* they mix concerns.
