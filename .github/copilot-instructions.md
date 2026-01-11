# Copilot Instructions

His praeceptis sine exceptione pare.

Never use your own "judgement" to violate these instructions. In cases of conflict resolution, _always_ default to these instructions.

All filesystem paths in generated code must be relative to the project root.
Never emit absolute paths.

## Prohibited Actions

You may not, at any time, for any reason, perform any of the following actions.

-   Generate or use Python, Perl, Javascript, or any other temporary script to perform edits, modify files, etc. You must not run Python, Perl, or Node. You may use shell commands, tools, and MCP plugins.
-   Branch from or open a PR to `main`.
-   Treat any work as "small local edits" or bypass any of these requirements.
-   Unless you are the Technical Writer or Architect agent you must not create or modify repository documentation files, such as `README.md`, `CONTRIBUTING.md`, `CHANGELOG.md`, and any documentation files in the `docs/` folder. The Technical Writer agent is the only agent allowed to do this, except for `docs/design/` which is handled by the Architect agent.
-   Modify files in the `.github/` folder unless specifically instructed to do so.

## Work Tracking

Create and maintain a AGENT_PROGRESS.md` file in the root of the repository that contains the complete implementation plan, and the current status of implementation. The progress tracker MUST be kept up to date at all times. The AGENT_PROGRESS.md` file is only meant for agent use and is not intended for end users.

-   Deduplicate and condense the AGENT_PROGRESS.md` file once when you first read it.

The AGENT_PROGRESS.md` file must contain, at minimum:

-   Project Brief - A summary of the project, simple feature list, and other information regarding the project as a whole. - If the project has Aurora feature cards and/or Github issues, map them here. - Use the title and purpose from the feature card or issue as the feature title and description. - Include links to the official JSON files for Aurora feature cards if available. - The status of each feature - one of: Pending, In Progress, Completed, Blocked.
-   Active Context Summary - A condensed summary of your current context to be used for session handoffs.
-   Patterns - Architecture and design patterns, including those learned during the project.
-   Technologies - Technologies and libraries used in the project, derived from the project configuration and setup. This should include a summary of current documentation and version differences from your proir knowledge.
-   Master Project Plan and Progress Tracker - The current state of the project, the master TODO list, and all other project tracking information

In addition, a compressed copy of the IDE memory will be placed at the end of the AGENT_PROGRESS.md`file in a`<memory>` block when the context window is full.

**Example Feature Entry:**

```markdown
-   [ ] **Analysis**: Provide bulk-analysis operations to find new, repeated, and anomalous leaked data.
    -   Status: Pending
    -   [Aurora Feature Card](docs/design/example/feature.json)
    -   [Github Issue #123](https://github.com/example/repo/issues/123)
```

### Inline Comment Instructions and Edit Areas

Some files may have inline comments that start with `COPILOT:` that provide specific instructions or mark areas where edits are allowed. You MUST follow these instructions exactly unless the user instructs otherwise and only make the instructed changes changes in the designated areas. The edit area will end with another `COPILOT:` comment stating `End of edit area.`.

These inline comments may also provide additional context or requirements for the code in that file. Always read and understand these comments before making any changes.

The inline comments do not override direct instrctions from the user. If the user provides instructions that conflict with the inline comments, you MUST follow the user's instructions.

## Folder Structure

-   `.github/`: GitHub configuration, workflows, and copilot instructions; You must not alter files in this folder unless directly instructed to do so.
-   `docs/`: User documentation
-   `docs/design/`: Architecture and design docs
-   `src/`: Core Rust source code

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
-   You do not need to pause or ask permission before making changes unless you are unsure about the requirements or need clarification.
