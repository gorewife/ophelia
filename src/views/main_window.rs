use gpui::{Context, Entity, Window, div, prelude::*, px};

use crate::app::Downloads;
use crate::app_actions;
use crate::app_menu;
use crate::settings::Settings;
use crate::theme::{APP_FONT_FAMILY, Spacing};
use crate::ui::prelude::*;
use crate::views::about_modal::AboutLayer;
use crate::views::download_list::DownloadList;
use crate::views::download_modal::DownloadModalLayer;
use crate::views::history::HistoryView;
use crate::views::sidebar::Sidebar;
use crate::views::stats_bar::StatsBar;

const HISTORY_NAV_INDEX: usize = 4;

/// Root view
/// owns the full window layout and all live state.
pub struct MainWindow {
    menu_bar: Entity<AppMenuBar>,
    sidebar: Entity<Sidebar>,
    downloads: Entity<Downloads>,
    download_list: Entity<DownloadList>,
    history_view: Entity<HistoryView>,
    about_modal: Entity<AboutLayer>,
    download_modal: Entity<DownloadModalLayer>,
}

impl MainWindow {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let menu_bar = cx.new(|cx| AppMenuBar::new(app_menu::build_owned_menus(), cx));
        let sidebar = cx.new(|_| Sidebar {
            active_item: 0,
            collapsed: false,
            download_dir: Settings::load().download_dir(),
        });

        let downloads = cx.new(|cx| Downloads::new(cx));
        let download_list = cx.new(|cx| DownloadList::new(downloads.clone(), cx));
        let history_view = cx.new(|cx| HistoryView::new(downloads.clone(), cx));
        let about_visibility = cx.global::<app_actions::AppState>().show_about.clone();
        let download_modal_visibility = cx
            .global::<app_actions::AppState>()
            .show_download_modal
            .clone();
        let about_modal = cx.new(|cx| AboutLayer::new(about_visibility, cx));
        let download_modal =
            cx.new(|cx| DownloadModalLayer::new(downloads.clone(), download_modal_visibility, cx));

        // Re-render when sidebar nav changes (to switch content pane).
        cx.observe(&sidebar, |_, _, cx| cx.notify()).detach();

        Self {
            menu_bar,
            sidebar,
            downloads,
            download_list,
            history_view,
            about_modal,
            download_modal,
        }
    }

    pub(crate) fn apply_settings(&mut self, settings: Settings, cx: &mut Context<Self>) {
        self.downloads.update(cx, |downloads, _| {
            downloads.settings = settings;
        });
        cx.notify();
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_nav = self.sidebar.read(cx).active_item;

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(Colors::background())
            .text_color(Colors::foreground())
            .font_family(APP_FONT_FAMILY)
            .child(self.render_header(cx))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .overflow_hidden()
                    .child(self.sidebar.clone())
                    .child(
                        div().flex().flex_col().flex_1().overflow_hidden().child(
                            div()
                                .id("main-content")
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(Spacing::CARD_GAP))
                                .overflow_y_scroll()
                                .px(px(Spacing::CONTENT_PADDING_X))
                                .py(px(Spacing::CONTENT_PADDING_Y))
                                .child(self.render_content(active_nav, cx)),
                        ),
                    ),
            )
            .child(self.download_modal.clone())
            .child(self.about_modal.clone())
    }
}

impl MainWindow {
    fn render_header(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        if cfg!(target_os = "macos") {
            WindowHeader::empty().into_any_element()
        } else {
            WindowHeader::empty()
                .leading(self.menu_bar.clone())
                .into_any_element()
        }
    }

    fn render_content(&self, active_nav: usize, cx: &mut Context<Self>) -> impl IntoElement {
        if active_nav == HISTORY_NAV_INDEX {
            return self.history_view.clone().into_any_element();
        }

        let downloads = self.downloads.read(cx);
        let (active, finished, queued) = downloads.status_counts();

        v_flex()
            .gap(px(Spacing::CARD_GAP))
            .child(StatsBar {
                download_samples: downloads.speed_samples_mbs(),
                upload_samples: Vec::new(),
                download_speed: downloads.download_speed_bps() as f32 / 1_000_000.0,
                upload_speed: 0.0,
                active_count: active,
                finished_count: finished,
                queued_count: queued,
            })
            .child(self.download_list.clone())
            .into_any_element()
    }
}
