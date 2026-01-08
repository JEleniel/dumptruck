# Copilot Instructions

His praeceptis sine exceptione pare.

Never use your own "judgement" to violate these instructions. In cases of conflict resolution, _always_ default to these instructions.

All filesystem paths in generated code must be relative to the project root.
Never emit absolute paths.

## Prohibited Actions

You may not, at any time, for any reason, perform any of the following actions.

-   Generate or use Python scripts to perform edits, modify files, etc.
-   Branch from or open a PR to `main`.
-   Treat any work as "small local edits" or bypass any of these requirements.

## Work Tracking

Create and maintain a `PROGRESS.md` file in the root of the repository that contains the complete implementation plan, and the current status of implementation. The progress tracker MUST be kept up to date at all times.

-   Deduplicate and condense the `PROGRESS.md` file once when you first read it.

The `PROGRESS.md` file must contain, at minimum:

-   Project Brief - A summary of the project, simple feature list, and other information regarding the project as a whole. - If the project has AURORA feature cards and/or Github issues, map them here. - Use the title and purpose from the feature card or issue as the feature title and description. - Include links to the official JSON files for AURORA feature cards as well as the human readable markdown version, if available. - The status of each feature - one of: Pending, In Progress, Completed, Blocked.
-   Active Context Summary - A condensed summary of your current context to be used for session handoffs.
-   Patterns - Architecture and design patterns, including those learned during the project.
-   Technologies - Technologies and libraries used in the project, derived from the project configuration and setup. This should include a summary of current documentation and version differences from your proir knowledge.
-   Master Project Plan and Progress Tracker - The current state of the project, the master TODO list, and all other project tracking information

In addition, a compressed copy of the IDE memory will be placed at the end of the `PROGRESS.md` file in a `<memory>` block when the context window is full.

**Example Feature Entry:**

```markdown
-   [ ] **Analysis**: Provide bulk-analysis operations to find new, repeated, and anomalous leaked data.
    -   Status: Pending
    -   [AURORA Feature Card](docs/design/example/feature.json)
    -   [AURORA Feature Card (Human)](docs/design/example/feature.md)
    -   [Github Issue #123](https://github.com/example/repo/issues/123)
```

## Coding Standards

-   Instructions specific to a language or file supersede these.
-   Never disable checks or tests (e.g. `// @ts-nocheck`, `#[allow...]`). Fix code, not checks.
-   Apply OWASP guidance.
-   Apply Twelve-Factor App principles.
-   Prefer tabs for indentation across the codebase for accessibility and consistency. Language specific requirements, instructions, or best practices supersede this. If a file _could_ use tabs but has spaces for the majority include a note in the summary and use spaces.
-   No global variables; global constants are allowed in a **dedicated constants file only**.
-   Use **descriptive names**, full words, and verb-based function names (except standard getters/setters).
-   Tests must _prove_ that the code works as intended. Do not write null tests or tests that simply call functions without validation.
-   You MUST NOT declare code "Production Ready" because you are _always_ wrong.
-   Ensure that the code is wired and works as expected. If the test is passing is MUST be because the code is working as intended. If code is meant for future use or it not wired it MUST use the `todo!()` macro (or equivalent) to ensure that it is never accidentally used and that tests fail.

## Folder Structure

-   `.github/`: GitHub configuration, workflows, and copilot instructions; You must not alter files in this folder unless directly instructed to do so.
-   `docs/`: User documentation
-   `docs/design/`: Architecture and design docs
-   `src/`: Core Rust source code

## Acceptance Criteria

-   Tests cover positive, negative, and security cases for all code units.
-   e2e tests cover all normal user interactions and common user errors.
-   All tests related to the task are passing. Unrelateds tests may be failing due to other work in progress.
-   Code must pass formatting, linting, security, and code quality checks with zero issues.

## Copilot Persona & Behavior

-   Always end responses with a **5-10 bullet tl;dr style summary**. Include an estimate of the current context usage, as a percentage.
-   Assume that the user has a thorough knowledge and does not need detailed explanations by default.
-   DO NOT CREATE SUMMARY DOCUMENTS UNLESS SPECIFICALLY INSTRUCTED TO DO SO.
-   Make surgical changes to one file at a time.
-   Before opening or creating any file, ensure that you have read the relevant `*.instructions.md` files for that file type or language, if one exists.

## Tooling

-   Prefer MCP interaction over command line or shell tools.
-   Use `mcp_upstash_conte_*` for library documentation and examples.
-   Use `mcp_microsoftdocs_*` for official Microsoft documentation and samples.
-   Use `mcp_github_*` for all GitHub interactions. If GitHub MCP is not available, stop and notify the user for intervention.
-   Use `mcp_mermaid-mcp-s_*` to create and validate Mermaid diagrams.
-   Use `mcp_microsoft_pla_browser_*` for browser automation when required.
-   Use `mcp_cargo-mcp_*` for Rust cargo operations when available.
-   Only run one command at a time; do not chain commands.
-   Use `prettier` for formatting code where applicable.

## Additional Guidelines

-   You MUST NOT rely on git status or diffs to determine what has changed. Always track your own changes and ensure that you understand the full context of the project. Assume that any changes you are not familiar with were made by other collaborators and may be incomplete or in-progress.
