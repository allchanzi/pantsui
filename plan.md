# 🐍 lazypython (working name)

A terminal user interface for exploring and interacting with **Python repositories** — inspired by tools like lazygit and lazydocker.

This project is primarily a **learning project for Rust**, focused on building a real-world, stateful TUI application with process integration, backend abstraction, and flexible support for different Python project types.

---

## 🎯 Goals

- Learn Rust by building a non-trivial application
- Understand ownership, enums, traits, and state management in practice
- Build a responsive TUI using modern Rust tooling
- Detect Python project type automatically
- Support both **Pants** and **Poetry**
- Support test discovery and execution for both **pytest** and **unittest**

---

## 🧠 What This Tool Will Do

When run inside a Python repository, the tool will:

- Detect whether the project uses Pants or Poetry
- Show a tree/list of test targets or test files
- Display test status
- Allow running tests from the UI
- Show execution history
- Display target/file metadata and dependencies where available
- Provide a dynamic detail panel based on focus

---

## 🖥️ UI Layout

The layout is inspired by lazygit/lazydocker:

- Left side = navigation + overview
- Right side = context-sensitive detail
- Focus determines what the right panel displays

---

## 🧱 Tech Stack

- `ratatui` → UI rendering
- `crossterm` → terminal + input handling
- `tokio` → async runtime
- `serde` / `serde_json` → parsing command output
- process execution via Rust standard library or async process APIs

---

## 📚 Learning Roadmap (Rust)

This project is structured to teach Rust progressively.

### Phase 0 — Fundamentals

Before starting:

- structs, enums, impl
- pattern matching (`match`)
- `Option` and `Result`
- ownership and borrowing
- `Vec`, `String`

---

### Phase 1 — Application State

Learn how to model application state:

- enums for modes and focus
- structs for domain objects
- separation of state and rendering

---

### Phase 2 — Ownership in Practice

Focus on:

- owned vs borrowed data
- when to clone vs borrow
- avoiding unnecessary lifetimes

---

### Phase 3 — External Process Integration

Later in the project:

- running external commands
- parsing stdout/stderr
- command failure handling
- repository detection

---

### Phase 4 — Async Programming

Later still:

- async/await
- tokio runtime
- channels (`mpsc`)
- background tasks

---

## 🛠️ Implementation Plan

Follow this order strictly — each step builds on the previous one.

---

### ✅ Milestone 1: TUI Shell

**Goal:** Render basic UI layout

- Setup terminal (alternate screen)
- Render:
  - left column
  - right column
- Handle quit (`q`)

**Learn:**

- basic Rust project structure
- ratatui layout
- terminal lifecycle

---

### ✅ Milestone 2: Static App State

**Goal:** Introduce structured state

Implement:

- `AppState`
- `FocusPane`
- `Mode`

Use hardcoded default values.

**Learn:**

- enums + structs
- central state pattern
- separation of concerns

---

### ✅ Milestone 3: Fake Domain Models

**Goal:** Add typed sample data

Implement:

- `TargetItem`
- `RunItem`
- `TestStatus`

Use hardcoded sample data and keep it in `AppState`.

**Learn:**

- data modeling
- enums for status
- `Vec<T>` and ownership of structured data

---

### ✅ Milestone 4: Terminal Initialization and Cleanup

**Goal:** Make terminal handling safe

- Enable raw mode
- Enter alternate screen
- Restore terminal state on exit

**Learn:**

- `Result`
- `?`
- cleanup logic
- terminal control

---

### ✅ Milestone 5: Render the First Layout

**Goal:** Draw the main layout

- Left column
- Right column
- Split left column into 3 stacked panels

**Learn:**

- layout rendering in `ratatui`
- rectangular UI regions
- separating layout from data

---

### ✅ Milestone 6: Input Handling

**Goal:** Make the UI interactive

- handle key presses
- support quit
- prepare for navigation

**Learn:**

- event loops
- input handling
- `match` on events

---

### ✅ Milestone 7: Navigation and Focus

**Goal:** Move between panels and items

Add:

- `j/k` or arrows → navigation
- `tab` → switch panels
- state updates for focus and selection

**Learn:**

- mutability
- state transitions
- pattern matching on enums

---

### ✅ Milestone 8: Backend Abstraction

**Goal:** Separate UI from data source

Define a backend interface for:

- detecting project type
- listing tests
- loading detail
- running tests

**Learn:**

- traits
- abstraction boundaries
- `Result` in APIs

---

### ✅ Milestone 9: Mock Backend

**Goal:** Wire the UI to a fake backend

- return static sample data
- keep UI independent from inline data construction

**Learn:**

- implementations of traits
- dependency inversion
- testing without external systems

---

### ✅ Milestone 10: Process Execution Basics

**Goal:** Run external commands from Rust

- execute a small command
- capture stdout and stderr
- handle failure cases cleanly

**Learn:**

- `std::process::Command`
- process output handling
- errors and string conversion

---

### ✅ Milestone 11: Repository Detection

**Goal:** Detect the project type automatically

- identify Pants projects
- identify Poetry projects
- fail gracefully if repo is unsupported

**Learn:**

- path handling
- filesystem inspection
- startup validation

---

### ✅ Milestone 12: Pants Backend

**Goal:** Load real data from Pants projects

- run Pants commands
- parse output
- populate target/test lists

**Learn:**

- process execution in practice
- parsing command output
- building typed data from text

---

### ✅ Milestone 13: Poetry Backend

**Goal:** Support Poetry projects

- run Poetry commands
- load test or project data for Poetry-based repos
- map output into the same UI model

**Learn:**

- multiple backend implementations
- shared domain models
- command composition

---

### ✅ Milestone 14: Test Framework Support

**Goal:** Support both pytest and unittest

- detect which test framework is in use
- list and run tests accordingly
- keep UI flow the same regardless of framework

**Learn:**

- enums for variants
- matching on project capabilities
- reusable abstraction design

---

### ✅ Milestone 15: Run Tests from UI

**Goal:** Execute actions from the terminal UI

- add a keybinding to run the selected test
- store run results
- show output/logs

**Learn:**

- process lifecycle
- state updates from actions
- user input → system effect flow

---

### ✅ Milestone 16: Loading and Error States

**Goal:** Make state transitions visible

- loading indicators
- error messages
- loaded / loading / failed states

**Learn:**

- enum-driven state
- `Option` and `Result`
- clean pattern matching

---

### ✅ Milestone 17: Async Architecture

**Goal:** Keep the UI responsive

Introduce:

- `tokio`
- background tasks
- channels (`mpsc`)

**Learn:**

- async/await
- concurrency
- message passing

---

### ✅ Milestone 18: Filters and Modes

**Goal:** Improve usability

- `/` filter
- mode switching
- filtering target/test lists

**Learn:**

- iterators
- `filter` and `collect`
- UI-driven data transforms

---

### ✅ Milestone 19: Detail Panel Improvements

**Goal:** Make the right panel context-aware

- show selected target/file info
- show backend-specific metadata
- show last run info

**Learn:**

- view models
- state-driven rendering
- data-to-UI mapping

---

### ✅ Milestone 20: UX Polish and Stabilization

**Goal:** Clean up and harden the app

- better shortcuts
- better error messages
- small refactors and cleanup

**Learn:**

- refactoring without breaking ownership rules
- reading compiler errors
- module organization

---

## 🚧 Future Ideas

- dependency graph view
- reverse dependencies
- changed files → affected tests
- watch mode
- cache insights
- integration with language-specific tooling
- richer project detection and backend plugins

---

## 🧩 Project Structure

```
src/
    main.rs
    app.rs
    state.rs
    event.rs
    action.rs
    backend/
        mod.rs
        traits.rs
        pants.rs
        poetry.rs
        mock.rs
ui/
    layout.rs
    panels/
        tests.rs
        runs.rs
        context.rs
        detail.rs
domain/
    project.rs
    target.rs
    run.rs
    status.rs
services/
    scheduler.rs
```

---

## 🧠 Key Design Principles

- Single source of truth (`AppState`)
- UI is a pure function of state
- Separate UI from backend logic
- Prefer simple ownership over complex lifetimes
- Introduce async only when needed
- Keep backend support extensible

---

## 🚀 Getting Started

```bash
cargo run
```

Run inside a Python repository for full functionality.

---

## ⏱️ 30-Minute Learning and Implementation Plan

This project is also a Rust learning path, so the work is intentionally split into small 30-minute sessions.

The goal is not to rush to a finished product, but to build the app in a way that teaches Rust fundamentals step by step.

---

### Session 1 — Create the project and run a minimal app

**Task**

- Create a new Rust project with Cargo
- Run `cargo run`
- Verify the development environment works

**You need to know**

- What `cargo new` does
- Basic Rust project structure
- What `main.rs` is
- How to run a Rust binary

**You will learn**

- How Rust projects are organized
- How Cargo works
- The edit-build-run loop

**Done when**

- The project builds successfully
- You can run the binary from the terminal

---

### Session 2 — Add dependencies and set up the project structure

**Task**

- Add initial dependencies to `Cargo.toml`
- Create the base folder/module structure

**Suggested dependencies**

- `ratatui`
- `crossterm`
- `tokio`
- `serde`
- `serde_json`

**You need to know**

- How `Cargo.toml` works
- What dependencies are
- Basic Rust modules and files

**You will learn**

- How Rust crates are added
- How modules map to files/folders
- How to organize a real project

**Done when**

- The project compiles with the new dependencies
- The folder structure exists

---

### Session 3 — Learn structs and enums through app state

**Task**

- Create a basic `AppState`
- Create enums for `FocusPane` and `Mode`

**You need to know**

- `struct`
- `enum`
- `impl`
- `Default`

**You will learn**

- How Rust models state
- Why enums are excellent for UI modes
- How to represent application state cleanly

**Done when**

- You have a compileable `AppState`
- Your state contains focus and mode

---

### Session 4 — Add fake data models

**Task**

- Create `TargetItem`
- Create `RunItem`
- Create `TestStatus`

**You need to know**

- Struct fields
- Enum variants
- Strings and vectors

**You will learn**

- How to model domain data
- How Rust types help avoid invalid states
- How to prepare for rendering UI from structured data

**Done when**

- You can create a vector of fake targets and runs
- The app state holds test/run data

---

### Session 5 — Set up terminal initialization and cleanup

**Task**

- Enable raw mode
- Enter alternate screen
- Restore terminal state on exit

**You need to know**

- Basic `Result` handling
- Function return types
- Error propagation with `?`

**You will learn**

- How terminal apps take control of the screen
- Why cleanup matters
- How Rust handles recoverable errors

**Done when**

- The app switches to an alternate terminal screen
- Exiting the app restores the terminal correctly

---

### Session 6 — Render the first layout

**Task**

- Draw the main layout:
  - left column
  - right column
- Split the left column into 3 stacked panels

**You need to know**

- Basic function calls
- Passing references
- Immutable borrowing

**You will learn**

- How layout rendering works in `ratatui`
- How to think in rectangular UI regions
- How to separate layout from data

**Done when**

- You can see 4 visible panels on screen
- The layout resembles the planned TUI structure

---

### Session 7 — Input loop and quit

**Task**

- Read keyboard input
- Add quit (`q`)
- Structure the main event loop

**You need to know**

- `loop`
- `match`
- Event handling basics

**You will learn**

- How interactive Rust apps process input
- How to structure a terminal event loop

**Done when**

- The app reacts to keyboard input
- You can safely quit the app

---

### Session 8 — Navigation and focus

**Task**

- `j/k` or arrows for navigation
- `tab` for switching panels
- update `focus_pane` in app state

**You need to know**

- Mutability
- `match` on enums
- Updating state from input

**You will learn**

- State transitions
- UI focus management
- How input changes application state

**Done when**

- You can move focus between panels
- The UI reflects the current focus

---

### Session 9 — Backend trait / interface

**Task**

- Define an abstraction for the backend

**Example responsibilities**

- detect project type
- list tests
- load target detail
- run test

**You need to know**

- Traits
- Return types
- `Result`

**You will learn**

- Separation of UI and backend
- Why abstractions make future work easier
- How to avoid tightly coupling everything

**Done when**

- UI code no longer depends directly on hardcoded data creation
- A backend interface exists

---

### Session 10 — Create a mock backend

**Task**

- Implement a fake backend that returns static data

**You need to know**

- Traits and implementations
- Owned data

**You will learn**

- How to develop the UI without real external integration
- How dependency inversion helps testing

**Done when**

- The app loads its data through a mock backend instead of inline fake values

---

### Session 11 — Learn process execution with a small command

**Task**

- Run a small external command from Rust
- Capture stdout and stderr

**You need to know**

- `std::process::Command`
- `Result`
- String conversion basics

**You will learn**

- How Rust runs external programs
- How to collect process output
- How command failures should be handled

**Done when**

- You can run a command and print or store its output

---

### Session 12 — Detect whether you are inside a supported Python repository

**Task**

- Add a repository detection step
- Recognize Pants and Poetry projects
- Fail gracefully if the repo is invalid

**You need to know**

- Path handling basics
- Error handling
- Conditional logic

**You will learn**

- Startup validation
- Defensive programming
- How to communicate missing prerequisites to users

**Done when**

- The app can show a useful error when not launched in a valid project

---

### Session 13 — Load real target data from Pants

**Task**

- Run a real Pants command
- Parse its output into `TargetItem`

**You need to know**

- Process execution
- String parsing
- `Vec` building

**You will learn**

- Real integration with an external tool
- How raw text becomes typed Rust data

**Done when**

- The target list is loaded from a Pants repository

---

### Session 14 — Load real target/data info from Poetry

**Task**

- Run a real Poetry command
- Parse output into the shared domain model

**You need to know**

- Backend-specific command composition
- Parsing command output
- Shared model mapping

**You will learn**

- How to support multiple backend implementations
- How to reuse the same UI with different data sources

**Done when**

- The app can load data from a Poetry project

---

### Session 15 — Support pytest and unittest

**Task**

- Detect which test framework is in use
- List tests and run tests for both frameworks

**You need to know**

- Enums or traits for variants
- Pattern matching
- Shared API design

**You will learn**

- How to model multiple supported behaviors
- How to keep one UI flow for different test systems

**Done when**

- The app can work with both pytest and unittest

---

### Session 16 — Run tests from the UI

**Task**

- Add a keybinding to run the selected test
- Store run results
- Show output/logs

**You need to know**

- State mutation
- Ownership when storing results
- Simple event → action → state flow

**You will learn**

- Process lifecycle
- State updates from user actions
- How UI actions connect to system effects

**Done when**

- A test can be run from the app
- The result is displayed in the UI

---

### Session 17 — Loading and error states

**Task**

- Add loading indicators
- Add error states
- Distinguish loaded / loading / failed

**You need to know**

- Enums for state
- `Option` and `Result`
- Pattern matching

**You will learn**

- How to model transient UI states
- How to avoid `unwrap`-driven design

**Done when**

- The app can show loading and error states clearly

---

### Session 18 — Async architecture

**Task**

- Introduce `tokio`
- Use background tasks
- Connect UI and backend via channels

**You need to know**

- `async/await`
- `tokio::spawn`
- `mpsc`

**You will learn**

- How to keep UI responsive
- Basic concurrency patterns in Rust

**Done when**

- Long-running work no longer blocks the UI

---

### Session 19 — Filters and modes

**Task**

- Add filter text
- Add mode switching
- Filter target/test lists

**You need to know**

- Iterators
- `filter`
- `collect`

**You will learn**

- Basic iterator pipelines
- How to transform state into view data
- Difference between source data and displayed data

**Done when**

- Changing the filter changes what is shown in the lists

---

### Session 20 — UX polish and stabilization

**Task**

- Improve keybindings
- Improve error messages
- Clean up structure and naming

**You need to know**

- Refactoring without breaking ownership rules
- Reading compiler errors
- Module organization

**You will learn**

- How to improve a Rust app safely
- How to stabilize a learning project into something usable

**Done when**

- The app is stable and ready for more features

---

## How to work with this plan

After each session:

- implement just that step
- compare your code against the session goal
- ask for a review before moving on
- keep each session small and focused
