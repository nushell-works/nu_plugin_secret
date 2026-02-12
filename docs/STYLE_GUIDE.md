# Style Guide

Coding conventions for the nu_plugin_nw_ulid project. Each item has a unique ID for easy reference.

## STYLE-0000: Style guide structure

**Tags:** `meta`

### Situation

A new convention needs to be added to this style guide.

### Guidance

Assign the next sequential ID (currently next is `STYLE-0015`) and include:

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
| `api-design`         | Ownership, must_use, string params                 |
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

- **Static regex** — use `std::sync::LazyLock` so the pattern is compiled once and the
  `unwrap()` is confined to the initialiser. Clippy's `invalid_regex` lint (deny by default)
  validates the literal at compile time, so the `unwrap()` is provably safe.

  ```rust
  use std::sync::LazyLock;
  use regex::Regex;

  static SCOPE_RE: LazyLock<Regex> =
      LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9-]*$").unwrap());
  ```

- **Known-safe constructors** — `FixedOffset::east_opt(0).unwrap()` where the argument is
  a constant that cannot fail.
- **Test code** — tests may use `unwrap()` freely.

**`expect()` is acceptable** for truly catastrophic I/O that should terminate the process:

```rust
io::stdout().flush().expect("Failed to flush stdout");
```

**Never** use `unwrap()` or `expect()` on user-supplied or runtime data in library code.
Use `?` with appropriate error conversion instead.

### Motivation

Panics in library code produce poor diagnostics and cannot be handled by callers. Limiting
panics to provably-safe or catastrophic cases keeps the error surface predictable.
A lazy static avoids recompiling the regex on every call and makes the safety argument
obvious at the declaration site.

---

## STYLE-0002: Naming patterns

**Tags:** `naming`

### Situation

Naming a new type, function, CLI command, or constant.

### Guidance

| Element           | Convention            | Examples                                      |
|-------------------|-----------------------|-----------------------------------------------|
| Structs / Enums   | PascalCase            | `UlidEngine`, `UlidError`, `SecurityRating`   |
| Traits            | PascalCase (adj/verb) | `PluginCommand`, `Serialize`, `Display`       |
| Functions/Methods | snake_case            | `extract_timestamp()`, `validate()`           |
| Type aliases      | PascalCase            | `Result<T>` (for crate-local aliases)         |
| Constants         | UPPER_SNAKE_CASE      | `ULID_STRING_LENGTH`, `DEFAULT_BATCH_SIZE`    |
| CLI commands      | kebab-case            | `ulid generate`, `ulid encode-base32`         |
| Modules / files   | snake_case            | `ulid_engine.rs`, `security.rs`               |

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

| Scope      | Covers                                        |
|------------|-----------------------------------------------|
| `plugin`   | Plugin registration, command dispatch         |
| `engine`   | Core UlidEngine operations                    |
| `commands` | Individual command implementations            |
| `security` | Security warnings, rating system              |
| `error`    | Error types and conversion                    |
| `ci`       | CI/CD workflows and configuration             |
| `deps`     | Dependency updates                            |
| `docs`     | Documentation                                 |
| `release`  | Version bumps, release process                |

**Multi-scope commits** — when a change touches multiple scopes equally, list them
comma-separated: `style(engine,commands): standardise doc comments`.

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
//! Core ULID engine providing all ULID operations for the plugin.
```

**Item-level docs** — every public struct, enum, field, variant, and method gets `///`:

```rust
/// Parsed components of a ULID.
pub struct UlidComponents {
    /// ISO 8601 timestamp extracted from the ULID.
    pub timestamp: String,
    /// Hexadecimal representation of the random component.
    pub randomness_hex: String,
}
```

**Summary line style** — write in **third-person singular present indicative** per
[RFC 505](https://rust-lang.github.io/rfcs/0505-api-comment-conventions.html). Use full
sentences ending with a period:

```rust
/// Generates a new ULID with the current timestamp.
pub fn generate() -> Result<String, UlidError> { ... }

/// Returns `true` if the string is a valid ULID.
pub fn validate(input: &str) -> bool { ... }
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
use std::str::FromStr;

use nu_protocol::{Record, Span, Value};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::security::SecurityWarnings;
use crate::UlidEngine;
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
dependencies (`ulid`, `chrono`, `sha2`, `blake3`). Requiring an ADR for any exception
ensures the decision is reviewed and documented.

---

## STYLE-0008: `#[must_use]` annotation

**Tags:** `api-design`

### Situation

A public function or method returns a computed value without side effects.

### Guidance

Apply `#[must_use]` to public functions whose return value is the entire point of the call.
Discarding the result is almost certainly a bug:

```rust
#[must_use]
pub fn validate(input: &str) -> bool { ... }

#[must_use]
pub fn is_security_sensitive_context(description: &str) -> bool { ... }
```

**Do not apply** `#[must_use]` to:

- Functions that return `Result` — the `#[must_use]` on `Result` itself already covers this.
- Builder methods that return `&mut Self` — the builder pattern implies chaining.
- Functions with meaningful side effects (I/O, mutation) where the return value is
  supplementary.

### Motivation

`#[must_use]` turns silent logic errors (ignoring a return value) into compiler warnings.
Applying it deliberately to pure computations catches bugs at compile time without producing
false positives on side-effectful functions. This aligns with `clippy::must_use_candidate`
from the `pedantic` group.

---

## STYLE-0009: String parameter ownership

**Tags:** `api-design`

### Situation

Deciding whether a function parameter should be `&str`, `String`, or generic.

### Guidance

Use the cheapest type that satisfies the function's needs:

| The function…                          | Accept              | Example                                     |
|----------------------------------------|---------------------|---------------------------------------------|
| Only reads the string                  | `&str`              | `fn validate(input: &str) -> bool`          |
| Stores the string in a struct/`Vec`    | `String`            | `fn set_title(&mut self, title: String)`    |
| Needs flexibility (public API surface) | `impl Into<String>` | `fn new(name: impl Into<String>) -> Self`   |

Prefer `&str` for internal helpers and `impl Into<String>` sparingly — only at public API
boundaries where caller ergonomics justify the generic. Avoid `impl AsRef<str>` unless you
genuinely need to accept both `String` and `&str` without conversion.

For return types, prefer `&str` when returning a reference to owned data, and `String` when
returning a newly constructed value. Avoid `Cow<'_, str>` unless profiling shows the
borrow-or-own flexibility is needed.

```rust
// Good — borrows for read-only access
pub fn timestamp(&self) -> &str {
    &self.timestamp
}

// Good — takes ownership because it stores the value
pub fn with_title(mut self, title: String) -> Self {
    self.title = title;
    self
}

// Good — constructs a new string
pub fn format_summary(&self) -> String {
    format!("{}: {}", self.timestamp, self.randomness_hex)
}
```

### Motivation

Accepting `&str` avoids unnecessary allocations on the caller side. Taking `String` when
ownership is needed makes the transfer explicit and avoids hidden `.to_string()` calls
inside the function. The `impl Into<String>` pattern is convenient for public APIs but adds
monomorphisation cost, so it should be used judiciously.

---

## STYLE-0010: Named constants

**Tags:** `code-style`, `naming`

### Situation

Using a numeric or string literal whose meaning is not obvious from surrounding context.

### Guidance

Extract **magic literals** into named constants or `const` items. A literal is "magic" when its
purpose is not self-evident at the usage site:

```rust
// Bad — what does 26 mean?
if input.len() != 26 {

// Good — the name documents the intent
const ULID_STRING_LENGTH: usize = 26;
if input.len() != ULID_STRING_LENGTH {
```

```rust
// Bad — why 10_000?
if count > 10_000 {
    return Err(...)
}

// Good
const MAX_BULK_GENERATION: usize = 10_000;
if count > MAX_BULK_GENERATION {
    return Err(...)
}
```

Literals that do **not** need extraction:

- **Structural zeros and ones** — `Vec::with_capacity(1)`, `index + 1`, `slice[0]`.
- **Format strings** — `format!("{}: {}", key, value)`.
- **Known-safe constructor arguments** — `FixedOffset::east_opt(0)` (covered by STYLE-0001).
- **Test assertions** — `assert_eq!(result.len(), 3)` where the value is local to the test.

Place constants at the narrowest useful scope: module-level `const` if used across functions in
the same module, crate-level if shared across modules, or function-local `const` if truly local.

### Motivation

Named constants make the code self-documenting and provide a single point of change when a value
needs updating. Searching for `ULID_STRING_LENGTH` finds every usage; searching for `26` returns
hundreds of false positives. The exceptions prevent over-extraction of trivially obvious values.

---

## STYLE-0011: Function length

**Tags:** `code-style`

### Situation

Writing or reviewing a function that is growing long.

### Guidance

Keep functions **under ~50 lines** of logic (excluding doc comments, blank lines, and closing
braces). When a function exceeds this guideline, look for opportunities to extract coherent
sub-operations into well-named helper functions.

Common extraction targets:

- **Setup / teardown** — opening resources, building configuration structs.
- **Distinct phases** — validation, transformation, output formatting.
- **Repeated patterns** — similar blocks that differ only in parameters.
- **Nested closures or callbacks** — especially credential handlers, diff callbacks.

```rust
// Before — 120-line run() mixing validation, parsing, formatting, and output
fn run(&self, ...) -> Result<PipelineData, LabeledError> {
    // ... 120 lines ...
}

// After — orchestrator delegates to focused helpers
fn run(&self, ...) -> Result<PipelineData, LabeledError> {
    let input = self.parse_input(&call)?;
    let components = self.build_components(&input)?;
    let record = self.format_output(components, &call)?;
    Ok(Value::record(record, span).into_pipeline_data())
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

## STYLE-0012: Silent error suppression

**Tags:** `error-handling`

### Situation

Handling a `Result` or `Option` where the error/`None` case is intentionally ignored.

### Guidance

**Never silently discard an error that could indicate a real problem.** Three patterns to watch
for:

1. **`let _ = fallible_call();`** — If the operation can meaningfully fail, at least log the
   error at `debug!` or `warn!` level. If the failure is truly inconsequential (best-effort
   cleanup), add a comment explaining why:

   ```rust
   // Bad — caller has no idea the operation failed
   let _ = fs::remove_file(&temp_path);

   // Good — intent is documented, failure is logged
   // Best-effort cleanup; the file may already have been removed.
   if let Err(e) = fs::remove_file(&temp_path) {
       eprintln!("Cleanup failed: {e}");
   }
   ```

2. **`if let Ok(x) = ... { use(x) }`** with no `else` — returning a silent default on parse
   or I/O failure hides broken data from the user:

   ```rust
   // Bad — silently returns 0 on parse failure
   let timestamp = UlidEngine::extract_timestamp(input).unwrap_or(0);

   // Good — the default is documented, or the error is surfaced
   let timestamp = match UlidEngine::extract_timestamp(input) {
       Ok(ts) => ts,
       Err(e) => {
           eprintln!("Failed to extract timestamp: {e}");
           0
       }
   };
   ```

3. **`.unwrap_or_default()` on non-trivial results** — acceptable for genuinely optional data,
   but not as a blanket substitute for error handling on operations that should succeed.

**Acceptable silent discards:**

- Closing a file or flushing a logger during shutdown.
- Sending on a channel where the receiver may have been dropped.
- Test cleanup in `Drop` implementations.

### Motivation

Silent error suppression is one of the hardest bugs to diagnose because nothing visibly fails —
the program simply produces wrong results or missing data. Logging at `debug!` or `warn!` level
costs nothing on the success path and provides a trail when something goes wrong. The explicit
comment requirement for `let _ =` forces the author to justify the suppression at write time,
which often reveals that the error should not be ignored after all.

---

## STYLE-0013: Single-purpose commits

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
a1b2c3  refactor(engine): extract timestamp formatting helper
d4e5f6  feat(commands): add --json output to inspect command

# Bad — mixed intent, hard to review or revert half of it
git log --oneline
f7g8h9  feat(commands): add --json output and refactor timestamp formatting
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

## STYLE-0014: Module cohesion

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
  state (e.g., `UlidSortCommand` and `UlidInspectCommand` in one file).
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
