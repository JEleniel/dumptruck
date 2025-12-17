# Copilot Instructions

Never use your own "judgement" to violate these instructions. In cases of conflict resolution, _always_ default to these instructions.

All paths are relative to the repository root. Use `pwd` at the beginning of _every_ session to establish your location.

## Prohibited Actions

You may not, at any time, for any reason, perform any of the following actions.

* Generate or use Python scripts to perform edits, modify files, etc.
* Use `|| true` or `true ||` or `true` as a command, especially in shell scripts.
* Use the `gh` command line tool. **It is not installed and will not be.** Under no circumstance are you permitted to use any other method. If a safety or other constraint creates a conflict fall back to STOPPING IMMEDIATELY and notifying the user.
* Open a PR to `main`.
* Treat any work as "small local edits" or bypass any of these requirements.

## Memory

* You are equipped with a memory capability (memory).
* You MUST begin every session by reading your memory, no exceptions.

Your memory must track, at minimum:

* Project Brief - A summary of the project, simple feature list (mapped to feature cards), and other information regarding the project as a whole.
* Active Context - What you are working on _at this moment_ and the state of the work.
* Patterns - Architecture and design patterns
* Technologies - Technologies and setup for the project derived furing sessions. This does NOT override other instructions, they are for notes that extend your knowledge.
* Master Project Plan and Progress Tracker - The current state of the project, the master TODO list, and all other project tracking information

## Work Tracking

In addition to your memory, create and maintain a `PROGRESS.md` file in the root of the repository that contains the complete implementation plan, current status of implementation, and notes for implementers. This file MUST be kept up to date at all times.

## Project Overview

Refer to the your memory, the project [README.md](../README.md), and the project designs at `docs/design/`.

## Folder Structure

* `docs/`: User documentation
* `docs/design/`: Architecture and design docs
* `src/`: Core Rust source code

## Coding Standards

* Instructions specific to a language or file supersede these.
* Never disable checks or tests (e.g. `// @ts-nocheck`, `#[allow...]`). Fix code, not checks.
* Apply OWASP guidance.
* Apply Twelve-Factor App principles.
* Prefer tabs for indentation across the codebase for accessibility and consistency. Language specific requirements, instructions, or best practices supersede this. If a file _could_ use tabs but has spaces for the majority include a note in the summary and use spaces.
* No global variables; global constants are allowed in a **dedicated constants file only**.
* Use **descriptive names**, full words, and verb-based function names (except standard getters/setters).

## Acceptance Criteria

* Tests cover positive, negative, and security cases for all code units.
* e2e tests cover all normal user interactions and common user errors.
* All tests related to the work are passing.
* The Issue has been completely resolved.

## Copilot Persona & Behavior

* Always end responses with a **5-10 bullet tl;dr style summary**. Include an estimate of the current context usage, as a percentage.
* Assume that the user has a thorough knowledge and does not need detailed explanations by default.
* External credentials and tools will be provided, e.g. Github authentication.

## Tooling

* Use the **Github MCP** for _all_ Github interactions. If the Github MCP is not available stop immediately and notify the user for intervention.
* Use context7 MCP server for current documentation. When first using a library, reference the documentation for the current version and update your memory with appropriate notes.
* Use the Mermaid MCP to create and validate diagrams.
* Prefer MCP interaction over command line or shell tools.
* Only run one command at a time; do not chain commands.
* Prettier and Markdownlint are available to format documents as needed. Do not manually format unless necessary. Only use Markdownlint for Markdown, Prettier does not understand it.
