# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Nushell plugin (`nu_plugin_secret`) providing 8 custom value types for secure secret handling. Secrets are always redacted in display/debug output but preserve their actual data through serialization for pipeline operations (the "dual-layer security model").

**Nushell compatibility**: nu-plugin/nu-protocol 0.110.0 · **MSRV**: 1.88.0 · **License**: BSD-3-Clause

## Build & Development Commands

```bash
cargo fmt                                                    # Format code
cargo clippy --all-targets --all-features -- -D warnings    # Lint (treat warnings as errors)
cargo test                                                   # Run all tests
cargo test <test_name>                                       # Run a single test
cargo test --test unified_wrap_tests                         # Run a specific test file
cargo +nightly miri test                                     # Run under Miri (MIRIFLAGS=-Zmiri-disable-isolation)
cargo bench --bench secret_performance                       # Run specific benchmark
./scripts/run_nu_tests.sh                                    # Run Nushell integration tests
./scripts/install-plugin.sh                                  # Build and register plugin with Nushell
```

**Pre-commit hook** runs `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings` automatically. Always run both before completing tasks.

## Architecture

### Plugin Entry Point

[main.rs](src/main.rs) serves the plugin via `nu-plugin`'s MsgPackSerializer. The `SecretPlugin` struct ([lib.rs](src/lib.rs)) implements the `Plugin` trait and registers 15 commands. Configuration is dependency-injected via `Arc<RwLock<ConfigManager>>`.

### Secret Types (`src/secret_types/`)

Eight types wrapping Nushell value types: `SecretString`, `SecretInt`, `SecretBool`, `SecretFloat`, `SecretDate`, `SecretBinary`, `SecretList`, `SecretRecord`.

Each type follows the same pattern:
- Struct with `inner: T` and `redaction_template: Option<String>`
- `Display`/`Debug` always return redacted output (never the real value)
- `Serialize`/`Deserialize` carry the actual data for pipeline operations
- `Drop` zeros memory via `zeroize` crate
- `CustomValue` impl integrates with Nushell's type system via `#[typetag::serde]`
- `reveal()` and `into_inner()` provide controlled access to the underlying value

### Commands (`src/commands/`)

- **`secret wrap`** / **`secret wrap-with`** — wrap values into secret types (with optional custom redaction template)
- **`secret unwrap`** — extract the underlying value
- **`secret contains`**, **`secret hash`**, **`secret length`**, **`secret validate`**, **`secret type-of`** — operate on secrets without exposing them
- **`secret info`** — plugin information
- **`secret configure`**, **`secret config {show,reset,validate,export,import}`** — configuration management

### Redaction System (`src/redaction.rs`, `src/tera_functions.rs`)

Uses the Tera template engine. Default template: `<redacted:{{secret_type}}>`. Custom functions available in templates: `replicate`, `reverse`, `take`, `strlen`, `mask_partial`, `secret_string`.

### Configuration (`src/config.rs`)

TOML-based config loaded from `~/.local/share/nushell/plugins/secret/config.toml` (platform-dependent via `dirs` crate). Supports environment variable overrides (e.g., `SHOW_UNREDACTED=1`). Three security levels: minimal, standard, paranoid.

## Code Quality Rules

See [docs/STYLE_GUIDE.md](docs/STYLE_GUIDE.md) for detailed, numbered coding conventions.

- **NEVER** use hardcoded mathematical constants (`3.14`, `2.718`). Use `std::f64::consts::PI`, `E`, `TAU` to avoid clippy `approx_constant` warnings.
- Write Miri-compatible code where possible (avoid system time in tests under Miri; config loading is `#[cfg(not(miri))]`-gated).
- This is security-focused code — never introduce changes that could leak sensitive information in display, debug, or log output.

## ADRs

Architecture Decision Records live in [docs/adrs/](docs/adrs/). Use four-digit zero-padded numbering (e.g., `adr-0001.md`). Run the `update-adr-inventory` skill after adding or changing an ADR.
