# `src/` layout

Ophelia keeps the frontend split into a few clear layers:

- `ui/`: reusable UI building blocks that are not tied to a specific product screen
- `views/`: app-specific compositions such as windows, panels, lists, and overlays
- `theme.rs`: shared design tokens and visual constants
- `app.rs`: application state entities and download-facing UI data
- `app_menu.rs` / `app_actions.rs`: app-level actions, shortcuts, and menu wiring
- `platform/`: platform-specific window/chrome integration
- `engine/`: download engine, persistence, and HTTP logic

## Frontend terms

These names are intentional:

- `primitive`: low-level reusable building block
- `control`: interactive widget with behavior/state
- `chrome`: reusable window/menu/modal shell UI
- `view`: app-specific composition of controls and chrome
- `helper`: non-visual utility function or tiny render helper

## Directory map

### `ui/`

- `primitives/`
  - `icon.rs`: icon rendering helpers and icon names
  - `logo.rs`: Ophelia logo element
- `controls/`
  - `text_field.rs`: custom text input
  - `number_input.rs`: numeric input control
  - `directory_input.rs`: directory-picker input control
- `chrome/`
  - `window_header.rs`: shared titlebar/header chrome
  - `app_menu_bar.rs`: Linux/Windows client-side app menu bar
  - `modal.rs`: reusable modal shell
- `prelude.rs`: shared UI imports for GPUI-heavy files

### `views/`

- `main/`
  - `main_window.rs`: root application window
  - `sidebar.rs`: main navigation/sidebar
  - `download_list.rs`: active download list composition
  - `download_row.rs`: individual download row pieces
  - `history.rs`: history view and filter chips
  - `stats_bar.rs`: throughput and status summary card
- `settings/`
  - `mod.rs`: settings window entity
  - `general.rs`: general settings section
  - `network.rs`: network settings section
- `overlays/`
  - `download_modal.rs`: add-download overlay
  - `about_modal.rs`: about overlay
  - `notification.rs`: transient notification popup

## Placement rules

When adding a new file:

- Put it in `ui/` if it should be reusable outside one screen or window.
- Put it in `views/` if it exists to assemble app-specific state and layout.
- Prefer extending an existing subfolder before creating a new top-level category.
- If a view grows, split presentational pieces first before introducing more folders.
