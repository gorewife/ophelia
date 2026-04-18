# Contributing to Ophelia

> [!NOTE]
> This doc is a work in progress and subject to change

First off, **all PRs are welcome**!

If you have an idea that makes Ophelia better, clearer, nicer to use, or easier to extend, go ahead and open a PR :p

That said, unfortunately for you, I'm a bit of a perfectionist (or add/autistic, diagnosis pending).

Ophelia is being built with a pretty deliberate architecture. The goal is not just to make features work today. The goal is to make future features, providers, and extensions fit into the codebase cleanly without the project turning into a pile of special cases. I won't bash you with insults Linus-style if you submit a bad PR but I WILL look at you like this

![very mad guy](https://100r.co/media/interface/travel.png)

# AI Disclaimer

Please disclaim any use of generative AI in your code, we will not accept PRs with AI-generated images, icons, SVG files, or other critically artistic works are included in the content of the pull request.

## Local updater QA

The custom updater is currently macOS-only and easiest to validate through the local Nightly flow.

Use:

- `scripts/local_nightly_update_qa.sh --minisign-pubkey "<pubkey>"`

That helper writes a reusable env file in `/tmp/ophelia-update-lab/qa-env.sh` and prints the exact remaining steps for:

- building an older Nightly QA app
- building a newer Nightly app
- signing, notarizing, and stapling the newer app
- rebuilding the updater ZIP and manifest
- serving the local update site

## Frontend Structure

The frontend is organized around a few simple layers:

### `src/ui/`

### Reusable UI building blocks

- `primitives/` for low-level reusable pieces
- `controls/` for interactive widgets with behavior/state
- `chrome/` for app shell pieces like headers, popups, and modal surfaces

### `src/views/`

App-specific screens, panels, and overlays.

### `src/app.rs`

This is the frontend-facing bridge to backend state for now, it's pretty long so it's probably due a refactor (yay!)

## Internationalize new user-facing text

If you add user-visible strings, make sure you wire them thru `rust-i18n`.

## Before Opening a PR, run

- `cargo fmt`
- `cargo check`
- `cargo test`
