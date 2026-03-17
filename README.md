# tack

`tack` is a small Rust terminal task app built around fast capture. It uses a three-pane TUI: group boxes across the top, a centered composer in the middle, and the active group's task log at the bottom. State is persisted to a single JSON file and saved after each meaningful change.

## Features

- `tack new <project-name>` creates `<project-name>.tack.json` and opens the TUI
- `tack open <path>` loads an existing project file and opens the TUI
- default `Inbox` group for new projects
- group selection and in-app group creation
- task capture from the center pane
- task closing and reopening from the bottom pane
- lightweight terminal animations for focus, inserts, status flashes, and invalid actions

## Run

```bash
cargo run -- new demo
```

To reopen an existing project:

```bash
cargo run -- open demo.tack.json
```

## Controls

- `Up` / `Down`: move between panes, and also move through tasks while the task pane is focused
- `Left` / `Right`:
  - in groups: move across the top group boxes
  - in composer: move the text cursor
  - in tasks: `Left` moves focus back to the composer
- `1` / `2` / `3`: jump directly to groups, capture, or tasks
- `Enter`:
  - in composer: create a task in the active group
  - in groups: activate the selected group
  - in tasks: close the selected task
  - in new-group modal: confirm the new group name
- `n`: start creating a new group while groups pane is focused
- `d`: clear all closed tasks in the active group after confirmation
- `x`: close the selected task while tasks pane is focused
- `o` or `r`: reopen the selected task while tasks pane is focused
- `Esc`: cancel new-group entry
- `q` or `Ctrl-C`: quit

## Design notes

- JSON was chosen for persistence because it is easy to inspect and stable for a small MVP.
- Closed tasks remain visible with a crossed-out style and can be reopened in place. That keeps history visible without adding filtering yet.
- The updated layout adds group capsules, a blinking text cursor, task motion for add/close/reopen actions, and small ambient motion in the capture pane while keeping the screen restrained.
