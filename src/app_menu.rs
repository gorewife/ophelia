use gpui::{Action, App, KeyBinding, Menu, MenuItem, OsAction, OwnedMenu, SharedString, actions};

use crate::ui::text_field;

actions!(ophelia_menu, [OpenDownloadModal, OpenSettings, About, Quit]);

pub fn init(cx: &mut App) {
    cx.bind_keys([
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-q", Quit, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("alt-f4", Quit, None),
        KeyBinding::new("secondary-,", OpenSettings, None),
        KeyBinding::new("secondary-n", OpenDownloadModal, None),
    ]);
}

pub fn build_menus() -> Vec<Menu> {
    if cfg!(target_os = "macos") {
        vec![
            Menu {
                name: "Ophelia".into(),
                items: vec![
                    MenuItem::action("About Ophelia", About),
                    MenuItem::separator(),
                    MenuItem::action("Settings", OpenSettings),
                    MenuItem::separator(),
                    MenuItem::action("Quit", Quit),
                ],
            },
            edit_menu(),
            Menu {
                name: "Window".into(),
                items: vec![MenuItem::action("New Download", OpenDownloadModal)],
            },
            Menu {
                name: "Help".into(),
                items: vec![MenuItem::action("About Ophelia", About)],
            },
        ]
    } else {
        vec![
            Menu {
                name: "File".into(),
                items: vec![
                    MenuItem::action("New Download", OpenDownloadModal),
                    MenuItem::action("Settings", OpenSettings),
                    MenuItem::separator(),
                    MenuItem::action("Quit", Quit),
                ],
            },
            edit_menu(),
            Menu {
                name: "Window".into(),
                items: vec![MenuItem::action("New Download", OpenDownloadModal)],
            },
            Menu {
                name: "Help".into(),
                items: vec![MenuItem::action("About Ophelia", About)],
            },
        ]
    }
}

pub fn build_owned_menus() -> Vec<OwnedMenu> {
    build_menus().into_iter().map(Menu::owned).collect()
}

pub enum OwnedMenuItemLike<'a> {
    Separator,
    Action {
        name: &'a str,
        action: &'a dyn Action,
        checked: bool,
        disabled: bool,
    },
}

pub fn owned_menu_items(menu: &OwnedMenu) -> impl Iterator<Item = OwnedMenuItemLike<'_>> {
    menu.items.iter().filter_map(|item| match item {
        gpui::OwnedMenuItem::Separator => Some(OwnedMenuItemLike::Separator),
        gpui::OwnedMenuItem::Action {
            name,
            action,
            checked,
            ..
        } => Some(OwnedMenuItemLike::Action {
            name,
            action: action.as_ref(),
            checked: *checked,
            disabled: false,
        }),
        _ => None,
    })
}

pub fn menu_label(menu: &OwnedMenu) -> SharedString {
    menu.name.clone()
}

fn edit_menu() -> Menu {
    Menu {
        name: "Edit".into(),
        items: vec![
            MenuItem::os_action("Cut", text_field::Cut, OsAction::Cut),
            MenuItem::os_action("Copy", text_field::Copy, OsAction::Copy),
            MenuItem::os_action("Paste", text_field::Paste, OsAction::Paste),
            MenuItem::separator(),
            MenuItem::os_action("Select All", text_field::SelectAll, OsAction::SelectAll),
        ],
    }
}
