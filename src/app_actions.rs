use gpui::{App, AppContext, Bounds, Global, PromptLevel, WindowHandle, px, size};

use crate::app_menu;
use crate::platform;
use crate::views::main_window::MainWindow;
use crate::views::settings::{SettingsClosed, SettingsWindow};

#[derive(Default)]
struct AppWindows {
    main_window: Option<WindowHandle<MainWindow>>,
    settings_window: Option<WindowHandle<SettingsWindow>>,
}

impl Global for AppWindows {}

pub fn init(main_window: WindowHandle<MainWindow>, cx: &mut App) {
    cx.set_global(AppWindows {
        main_window: Some(main_window),
        settings_window: None,
    });

    cx.on_action(open_download_modal);
    cx.on_action(open_settings);
    cx.on_action(quit);
    cx.on_action(about);
}

fn open_download_modal(_: &app_menu::OpenDownloadModal, cx: &mut App) {
    let Some(main_window) = main_window(cx) else {
        return;
    };

    let _ = main_window.update(cx, |this, window, cx| {
        window.activate_window();
        this.open_modal(cx);
    });
}

fn open_settings(_: &app_menu::OpenSettings, cx: &mut App) {
    if !cx.has_global::<AppWindows>() {
        return;
    }

    if let Some(settings_window) = cx.global::<AppWindows>().settings_window {
        if settings_window
            .update(cx, |_, window, _| {
                window.activate_window();
            })
            .is_ok()
        {
            return;
        }

        cx.global_mut::<AppWindows>().settings_window = None;
    }

    let Some(main_window) = main_window(cx) else {
        return;
    };

    let bounds = Bounds::centered(None, size(px(960.), px(600.)), cx);
    let Ok(settings_window) = cx.open_window(platform::window_options(bounds), |_, cx| {
        cx.new(|cx| SettingsWindow::new(cx))
    }) else {
        return;
    };

    if let Ok(entity) = settings_window.entity(cx) {
        let subscription = cx.subscribe(&entity, move |_, event: &SettingsClosed, cx| {
            let _ = main_window.update(cx, |this, _, cx| {
                this.apply_settings(event.settings.clone(), cx);
            });

            if cx.has_global::<AppWindows>() {
                cx.global_mut::<AppWindows>().settings_window = None;
            }
        });
        subscription.detach();
    }

    cx.global_mut::<AppWindows>().settings_window = Some(settings_window);
}

fn quit(_: &app_menu::Quit, cx: &mut App) {
    cx.quit();
}

fn about(_: &app_menu::About, cx: &mut App) {
    if let Some(active_window) = cx.active_window() {
        let _ = active_window.update(cx, |_, window, cx| {
            let _ = window.prompt(
                PromptLevel::Info,
                "Ophelia",
                Some("Feature-rich and extensible download manager"),
                &["OK"],
                cx,
            );
        });
        return;
    }

    let Some(main_window) = main_window(cx) else {
        return;
    };

    let _ = main_window.update(cx, |_, window, cx| {
        let _ = window.prompt(
            PromptLevel::Info,
            "Ophelia",
            Some("Feature-rich and extensible download manager"),
            &["OK"],
            cx,
        );
    });
}

fn main_window(cx: &App) -> Option<WindowHandle<MainWindow>> {
    cx.has_global::<AppWindows>()
        .then(|| cx.global::<AppWindows>().main_window)
        .flatten()
}
