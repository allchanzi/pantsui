# 🐍 pantsui

A terminal UI for exploring and interacting with **[Pants](https://www.pantsbuild.org/)**-managed Python repositories — inspired by [lazygit](https://github.com/jesseduffield/lazygit) and [lazydocker](https://github.com/jesseduffield/lazydocker).

> ⚠️ **Status: early / work in progress.** A stateful TUI with process integration and a clean, layered architecture.

---

## What it does

Run it inside a Pants repository to:

- Detect the Pants repo automatically (or fail gracefully if it isn't one)
- Browse a list of test targets
- Discover available Pants **goals** and run them from the UI
- View target metadata and dependencies in a context-sensitive detail panel
- See execution history and test status

---

## Install & run

Requires a recent stable [Rust toolchain](https://rustup.rs/) and the [`pants`](https://www.pantsbuild.org/) CLI on your `PATH`.

```bash
cargo run
```

Run from inside a Pants repository for full functionality.

---

## Goals as actions

Pants exposes ~30 goals. pantsui classifies each into one of four kinds that drive how the UI runs it:

| Kind | Examples | Behavior |
|---|---|---|
| **Data** | `list`, `peek`, `dependencies`, `roots` | populates panels (not a user action) |
| **Action** | `test`, `lint`, `check`, `package` | runs and streams output |
| **Mutating** | `fmt`, `fix`, `tailor`, `publish` | confirms first, then runs |
| **Interactive** | `run`, `repl` | hands the terminal to the child process |

---

## Concepts

Two principles guide how pantsui is built:

- **Declarative first.** Code describes *what* the UI and behavior should be, not *how* to mutate it step by step. State is the source of truth; the UI is rendered as a function of that state, and goals are described as data (their kind drives behavior) rather than hand-wired control flow. Prefer expressing intent declaratively over imperative mutation.
- **Vertical slice architecture.** The codebase is organized by feature, not by technical layer. Each slice owns the full vertical of its concern — model, Pants interaction, and UI panel — instead of being split across horizontal `models/`, `services/`, `views/` folders. This keeps features self-contained and easy to reason about in isolation.

---

## Architecture

**Vertical slice architecture** — code is organized by feature, not by technical layer. Each slice owns its model, its Pants interaction, and its UI panel. A thin shared `shared/pants/` kernel handles the one cross-cutting concern: running the `pants` CLI and parsing its JSON. A thin shell (`state.rs` + `ui.rs`) holds the overall state and composes the feature panels into the layout.

```
src/
  shared/
    pants/        shared kernel: command runner, JSON parsing, models, errors
  features/
    version/      pants version + metadata sub-tabs   (model + data + panel)
    user_input/   keyboard input handling
    ...           targets / goals / runs slices        (planned)
  terminal/       raw mode / alt screen / interactive handoff
  error.rs        app-level error & result types
  state.rs        shell state: focus, mode, aggregated slice state
  ui.rs           shell composition: lays out the frame, draws each feature's panel
  main.rs         bootstrap + event loop
```

Each slice renders its own panel (and its internal layout when active); the shell `ui.rs` arranges those panels into the overall layout and routes focus.

---

## Tech stack

`ratatui` · `crossterm` · `tokio` · `serde` / `serde_json`

---

## License

TBD.
