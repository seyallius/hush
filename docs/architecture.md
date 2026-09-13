# Architecture

Hush is organized into a Cargo workspace with strict, one-way dependency rules. This prevents circular dependencies and
ensures that core logic remains completely agnostic of the user interface.

## Crate Dependency Graph

```text
hush_cli  ──┐
            ├──► hush_core ──┬──► hush_crypto
hush_tui  ──┘                └──► hush_envelope
└───────────────┬──────► hush_config ◄──────┘
```

## The Golden Rules

1. **UI is ephemeral, Core is permanent**: `hush_core` contains the business logic. It **never** depends on `hush_cli`
   or `hush_tui`. If the core needs to trigger an event, it returns a state or an error; the UI decides how to display
   it.
2. **Primitives are isolated**: `hush_crypto` and `hush_envelope` handle bytes and math. They **never** depend on
   `hush_core`.
3. **Config is a leaf**: `hush_config` defines the data structures for settings. Every other crate can read the config,
   but the config crate never imports internal logic.

## API Boundaries

- **`pub`**: Used strictly for the Public API of a crate (e.g., `hush_core` exporting the `Vault` struct for the CLI to
  use).
- **`pub(crate)`**: Used for internal helpers. If a function is only used inside `hush_crypto`, it must be `pub(crate)`
  to hide implementation details and prevent tight coupling.
