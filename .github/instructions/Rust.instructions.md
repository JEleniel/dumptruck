---
applyTo: '*.rs'
---

# Rust Style Guide

This document defines formatting and style conventions for all Rust source code These rules are enforced by the project's `rustfmt.toml` configuration.

---

## Formatting Rules

- Use `cargo fmt` to format Rust files.
- Use `cargo clippy` to lint Rust files.
- Use `cargo test` to run tests.
- **Version** Always use the Rust 2004 or later edition.
- **Organization** Organize code into modules logically, and use submodules as needed. Minimize top level `*.rs` files by grouping related functionality into modules. Use the 2024 style of `<name.rs>` and `<name>/` directories for modules. Do NOT use `mod.rs` files.
- **Minimize Lines per File** Aim for a maximum of 200 lines per file. Split large files into smaller, focused modules. Modules should be in files named for the module. Do not mix multiple modules in a single file.
- **Indentation:** Use hard tabs for indentation. Do not use spaces.
- **Line Endings:** Use Unix-style newlines (`\n`).
- **Comment Width:** Limit comments to 100 characters per line.
- **Comment Formatting:** Normalize comments and doc attributes. Wrap comments for readability.
- **Doc Comments:** Format code in documentation comments. Use `//!` for module/crate docs and `///` for item docs.
- **Imports:** Group imports by standard, external, and crate. Use crate-level granularity and reorder implementation items.
- **Hex Literals:** Use uppercase for hex literals.
- **Wildcards:** Condense wildcard suffixes in patterns.
- **Macros:** Format macro matchers for clarity.
- **Strings:** Format string literals for consistency.
- **Field Initialization:** Use field init shorthand where possible.
- **Try Shorthand:** Prefer the `?` operator for error propagation.
- **General:** Normalize all code and documentation attributes.
- Follow the [2024 Rust Style Guide](https://doc.rust-lang.org/stable/style-guide/index.html) for idiomatic code for all rules not covered here.

---

## Best Practices

- `#[allow...]` is not permitted.
- Use `thiserror` and `anyhow` for error handling. All errors MUST be handled, and top level errors should be logged and the app exited cleanly.
- Use `clippy` lints to enforce code quality. Address all warnings.
- Write clear, concise, and well-documented code.
- Include comments for non-obvious logic.
- Do not use unsafe code; all code must be 100% safe Rust.
- Ensure code compiles and passes all tests and lints.

---

## Enforcement

Run rustfmt and clippy as part of the CI pipeline to enforce these rules. All code must pass formatting and linting checks before merging.
