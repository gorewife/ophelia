/***************************************************
** This file is part of Ophelia.
** Copyright © 2026 Viktor Luna <viktor@hystericca.dev>
** Released under the GPL License, version 3 or later.
**
** If you found a weird little bug in here, tell the cat:
** viktor@hystericca.dev
**
**   ⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜⏜
** ( bugs behave plz, we're all trying our best )
**   ⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝⏝
**   ○
**     ○
**       ／l、
**     （ﾟ､ ｡ ７
**       l  ~ヽ
**       じしf_,)ノ
**************************************************/

use gpui::{
    App, Context, Entity, IntoElement, Render, RenderOnce, SharedString, Window, div, prelude::*,
    px,
};
use std::path::Path;

use crate::app::{Downloads, TransferListRow};
use crate::engine::{DownloadId, DownloadStatus};
use crate::settings::{Settings, suggested_destination_rule_icon_name};
use crate::ui::prelude::*;
use crate::views::main::transfer_row::TransferRow;
use crate::views::main::transfer_row::default_transfer_icon_name_for_filename;

use rust_i18n::t;

#[derive(Clone, Copy, PartialEq, Eq)]
enum TransferFilter {
    All,
    Active,
    Finished,
    Paused,
    Failed,
}

impl TransferFilter {
    fn matches(self, status: DownloadStatus) -> bool {
        match self {
            Self::All => true,
            Self::Active => matches!(
                status,
                DownloadStatus::Downloading | DownloadStatus::Pending
            ),
            Self::Finished => status == DownloadStatus::Finished,
            Self::Paused => status == DownloadStatus::Paused,
            Self::Failed => matches!(status, DownloadStatus::Error | DownloadStatus::Cancelled),
        }
    }
}

pub struct TransferList {
    downloads: Entity<Downloads>,
    filter: TransferFilter,
    selected_id: Option<DownloadId>,
}

pub struct TransferListSelectionChanged {
    pub id: Option<DownloadId>,
}

impl gpui::EventEmitter<TransferListSelectionChanged> for TransferList {}

impl TransferList {
    pub fn new(downloads: Entity<Downloads>, cx: &mut Context<Self>) -> Self {
        cx.observe(&downloads, |_, _, cx| cx.notify()).detach();
        Self {
            downloads,
            filter: TransferFilter::All,
            selected_id: None,
        }
    }

    fn view_model(
        &self,
        rows: Vec<TransferListRow>,
        selected_id: Option<DownloadId>,
        settings: &Settings,
    ) -> TransferListViewModel {
        let downloads = self.downloads.clone();
        let filters = vec![
            TransferFilterChipModel::new(
                0,
                TransferFilter::All,
                t!("transfers.filter_all").to_string(),
                self.filter == TransferFilter::All,
            ),
            TransferFilterChipModel::new(
                1,
                TransferFilter::Active,
                t!("transfers.filter_active").to_string(),
                self.filter == TransferFilter::Active,
            ),
            TransferFilterChipModel::new(
                2,
                TransferFilter::Finished,
                t!("transfers.filter_finished").to_string(),
                self.filter == TransferFilter::Finished,
            ),
            TransferFilterChipModel::new(
                3,
                TransferFilter::Paused,
                t!("transfers.filter_paused").to_string(),
                self.filter == TransferFilter::Paused,
            ),
            TransferFilterChipModel::new(
                4,
                TransferFilter::Failed,
                t!("transfers.filter_failed").to_string(),
                self.filter == TransferFilter::Failed,
            ),
        ];

        let rows = rows
            .into_iter()
            .map(|row| {
                let id = row.id;
                let selected = selected_id == Some(id);
                let icon_name = resolved_transfer_icon_name(&row, settings);
                let on_pause_resume: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>> =
                    if row.available_actions.pause {
                        let downloads = downloads.clone();
                        Some(Box::new(move |_window: &mut Window, app: &mut App| {
                            downloads.update(app, |downloads, cx| downloads.pause(id, cx));
                        }))
                    } else if row.available_actions.resume {
                        let downloads = downloads.clone();
                        Some(Box::new(move |_window: &mut Window, app: &mut App| {
                            downloads.update(app, |downloads, cx| downloads.resume(id, cx));
                        }))
                    } else {
                        None
                    };

                let on_remove = if row.available_actions.delete_artifact {
                    let downloads = downloads.clone();
                    Some(Box::new(move |_window: &mut Window, app: &mut App| {
                        downloads.update(app, |downloads, cx| downloads.remove(id, cx));
                    })
                        as Box<dyn Fn(&mut Window, &mut App) + 'static>)
                } else {
                    None
                };

                TransferRow {
                    id,
                    filename: row.filename,
                    destination: row.destination,
                    icon_name: icon_name.into(),
                    downloaded_bytes: row.downloaded_bytes,
                    total_bytes: row.total_bytes,
                    progress: row.progress,
                    state: row.display_state,
                    selected,
                    on_select: None,
                    on_pause_resume,
                    on_remove,
                }
            })
            .collect();

        TransferListViewModel {
            filters,
            rows,
            selected_id,
        }
    }

    pub fn visible_transfer_rows(&self, cx: &App) -> Vec<TransferListRow> {
        self.downloads
            .read(cx)
            .transfer_rows()
            .into_iter()
            .filter(|row| self.filter.matches(row.status))
            .collect()
    }

    fn set_selected_id(&mut self, selected_id: Option<DownloadId>, cx: &mut Context<Self>) {
        if self.selected_id == selected_id {
            return;
        }

        self.selected_id = selected_id;
        cx.emit(TransferListSelectionChanged { id: selected_id });
        cx.notify();
    }
}

impl Render for TransferList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (rows, settings) = {
            let downloads = self.downloads.read(cx);
            (
                downloads
                    .transfer_rows()
                    .into_iter()
                    .filter(|row| self.filter.matches(row.status))
                    .collect::<Vec<_>>(),
                downloads.settings.clone(),
            )
        };
        let selected_id = resolve_selected_transfer_id(&rows, self.selected_id);
        if selected_id != self.selected_id {
            self.selected_id = selected_id;
            cx.emit(TransferListSelectionChanged { id: selected_id });
        }

        let view_model = self.view_model(rows, selected_id, &settings);
        let weak = cx.weak_entity();

        v_flex()
            .pt(px(Spacing::SECTION_GAP))
            .child(
                div()
                    .text_sm()
                    .text_color(Colors::muted_foreground())
                    .font_weight(gpui::FontWeight::EXTRA_BOLD)
                    .mb(px(Spacing::SECTION_LABEL_BOTTOM_MARGIN))
                    .child(t!("transfers.section_label").to_string()),
            )
            .child(
                h_flex()
                    .items_center()
                    .gap(px(Chrome::MENU_BAR_GAP))
                    .mb(px(Spacing::SECTION_GAP))
                    .children(view_model.filters.into_iter().map(|filter_model| {
                        let filter = filter_model.filter;
                        FilterChip::new(
                            ("transfer-filter", filter_model.id),
                            filter_model.label,
                            filter_model.active,
                        )
                        .on_click({
                            let weak = weak.clone();
                            move |_, _, cx| {
                                let _ = weak.update(cx, |this, cx| {
                                    this.filter = filter;
                                    cx.notify();
                                });
                            }
                        })
                        .into_any_element()
                    })),
            )
            .child(if view_model.rows.is_empty() {
                TransferListEmptyState.into_any_element()
            } else {
                v_flex()
                    .gap(px(Spacing::LIST_GAP))
                    .children(view_model.rows.into_iter().map(|mut row| {
                        let id = row.id;
                        row.selected = view_model.selected_id == Some(id);
                        row.on_select = Some(Box::new({
                            let weak = weak.clone();
                            move |_window: &mut Window, app: &mut App| {
                                let _ = weak.update(app, |this, cx| {
                                    this.set_selected_id(Some(id), cx);
                                });
                            }
                        }));
                        row
                    }))
                    .into_any_element()
            })
    }
}

struct TransferListViewModel {
    filters: Vec<TransferFilterChipModel>,
    rows: Vec<TransferRow>,
    selected_id: Option<DownloadId>,
}

#[derive(Clone)]
struct TransferFilterChipModel {
    id: usize,
    filter: TransferFilter,
    label: SharedString,
    active: bool,
}

impl TransferFilterChipModel {
    fn new(
        id: usize,
        filter: TransferFilter,
        label: impl Into<SharedString>,
        active: bool,
    ) -> Self {
        Self {
            id,
            filter,
            label: label.into(),
            active,
        }
    }
}

#[derive(IntoElement)]
struct TransferListEmptyState;

impl RenderOnce for TransferListEmptyState {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .text_sm()
            .text_color(Colors::muted_foreground())
            .child(t!("transfers.empty_state").to_string())
    }
}

fn resolve_selected_transfer_id(
    rows: &[TransferListRow],
    selected_id: Option<DownloadId>,
) -> Option<DownloadId> {
    match selected_id {
        Some(selected_id) if rows.iter().any(|row| row.id == selected_id) => Some(selected_id),
        _ => rows.first().map(|row| row.id),
    }
}

fn resolved_transfer_icon_name(row: &TransferListRow, settings: &Settings) -> String {
    destination_rule_icon_name_for_transfer(row, settings).unwrap_or_else(|| {
        default_transfer_icon_name_for_filename(row.filename.as_ref()).to_string()
    })
}

fn destination_rule_icon_name_for_transfer(
    row: &TransferListRow,
    settings: &Settings,
) -> Option<String> {
    if !settings.destination_rules_enabled {
        return None;
    }

    let destination_dir = Path::new(row.destination.as_ref()).parent()?;
    let filename_extension = normalized_filename_extension(row.filename.as_ref())?;

    settings
        .destination_rules
        .iter()
        .find(|rule| {
            rule.enabled
                && destination_dir == rule.target_dir.as_path()
                && rule
                    .extensions
                    .iter()
                    .filter_map(|extension| normalized_rule_extension(extension))
                    .any(|extension| extension == filename_extension)
        })
        .map(|rule| {
            rule.icon_name
                .as_deref()
                .unwrap_or_else(|| {
                    suggested_destination_rule_icon_name(&rule.label, &rule.extensions)
                })
                .to_string()
        })
}

fn normalized_filename_extension(filename: &str) -> Option<String> {
    Path::new(filename)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| format!(".{}", extension.to_ascii_lowercase()))
}

fn normalized_rule_extension(extension: &str) -> Option<String> {
    let trimmed = extension.trim();
    if trimmed.is_empty() {
        None
    } else if trimmed.starts_with('.') {
        Some(trimmed.to_ascii_lowercase())
    } else {
        Some(format!(".{}", trimmed.to_ascii_lowercase()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{TransferAvailableActions, TransferDisplayState};
    use crate::settings::DestinationRule;
    use std::path::PathBuf;

    #[test]
    fn selection_defaults_to_first_visible_row() {
        let rows = vec![test_row(DownloadId(3)), test_row(DownloadId(7))];

        assert_eq!(
            resolve_selected_transfer_id(&rows, None),
            Some(DownloadId(3))
        );
    }

    #[test]
    fn selection_preserves_current_row_when_still_visible() {
        let rows = vec![test_row(DownloadId(3)), test_row(DownloadId(7))];

        assert_eq!(
            resolve_selected_transfer_id(&rows, Some(DownloadId(7))),
            Some(DownloadId(7))
        );
    }

    #[test]
    fn selection_falls_back_when_row_is_removed_or_filtered_out() {
        let rows = vec![test_row(DownloadId(11)), test_row(DownloadId(14))];

        assert_eq!(
            resolve_selected_transfer_id(&rows, Some(DownloadId(99))),
            Some(DownloadId(11))
        );
    }

    #[test]
    fn selection_clears_when_no_rows_are_visible() {
        assert_eq!(resolve_selected_transfer_id(&[], Some(DownloadId(5))), None);
    }

    #[test]
    fn matched_destination_rule_icon_wins_over_filename_heuristic() {
        let mut settings = Settings::default();
        settings.destination_rules_enabled = true;
        settings.destination_rules = vec![DestinationRule {
            id: "movies".into(),
            label: "Movies".into(),
            enabled: true,
            target_dir: PathBuf::from("/tmp/Movies"),
            extensions: vec![".mkv".into()],
            icon_name: Some("video".into()),
        }];

        let row = TransferListRow {
            destination: "/tmp/Movies/movie.mkv".into(),
            filename: "movie.mkv".into(),
            ..test_row(DownloadId(1))
        };

        assert_eq!(resolved_transfer_icon_name(&row, &settings), "video");
    }

    #[test]
    fn icon_falls_back_to_filename_when_no_destination_rule_matches() {
        let mut settings = Settings::default();
        settings.destination_rules_enabled = true;
        settings.destination_rules = vec![DestinationRule {
            id: "docs".into(),
            label: "Docs".into(),
            enabled: true,
            target_dir: PathBuf::from("/tmp/Documents"),
            extensions: vec![".pdf".into()],
            icon_name: Some("document".into()),
        }];

        let row = TransferListRow {
            destination: "/tmp/Videos/movie.mkv".into(),
            filename: "movie.mkv".into(),
            ..test_row(DownloadId(1))
        };

        assert_eq!(resolved_transfer_icon_name(&row, &settings), "video");
    }

    fn test_row(id: DownloadId) -> TransferListRow {
        TransferListRow {
            id,
            provider_kind: "http".into(),
            source_label: "https://example.com/file.bin".into(),
            filename: "file.bin".into(),
            destination: "/tmp/file.bin".into(),
            status: DownloadStatus::Downloading,
            downloaded_bytes: 512,
            total_bytes: Some(1024),
            progress: 0.5,
            speed_bps: 0,
            display_state: TransferDisplayState::Active,
            available_actions: TransferAvailableActions {
                pause: true,
                resume: false,
                cancel: true,
                delete_artifact: true,
            },
        }
    }
}
