//! UiTheme からウィジェットスタイルを生成するヘルパー。
//! スタイルクロージャは Color(Copy) をキャプチャして返す。

use crate::model::theme::UiTheme;
use iced::widget::{button, container, text_input};
use iced::{Background, Border, Color, Theme};

fn with_alpha(c: Color, a: f32) -> Color {
    Color { a, ..c }
}

/// ウィンドウ全体の背景（角丸+枠）
pub fn window_root(ui: &UiTheme) -> impl Fn(&Theme) -> container::Style {
    let bg = with_alpha(ui.bg_primary, ui.window_opacity);
    let border = ui.border_color;
    let radius = ui.border_radius;
    move |_| container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            color: border,
            width: 1.0,
            radius: radius.into(),
        },
        ..Default::default()
    }
}

/// タイトルバー/タブバーなどの帯
pub fn bar(ui: &UiTheme) -> impl Fn(&Theme) -> container::Style {
    let bg = with_alpha(ui.bg_secondary, ui.window_opacity);
    let radius = ui.border_radius;
    move |_| container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            radius: iced::border::Radius {
                top_left: radius,
                top_right: radius,
                bottom_right: 0.0,
                bottom_left: 0.0,
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

/// 帯（角丸なし・タブバー用）
pub fn bar_plain(ui: &UiTheme) -> impl Fn(&Theme) -> container::Style {
    let bg = with_alpha(ui.bg_secondary, ui.window_opacity);
    move |_| container::Style {
        background: Some(Background::Color(bg)),
        ..Default::default()
    }
}

/// 半透明パネル（ドラッグゴースト用）
pub fn panel_translucent(ui: &UiTheme) -> impl Fn(&Theme) -> container::Style {
    let bg = with_alpha(ui.bg_secondary, 0.75);
    let border = ui.accent;
    let radius = ui.border_radius_sm;
    move |_| container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            color: border,
            width: 1.0,
            radius: radius.into(),
        },
        ..Default::default()
    }
}

/// ステータスバー（下端）
pub fn statusbar(ui: &UiTheme) -> impl Fn(&Theme) -> container::Style {
    let bg = with_alpha(ui.bg_secondary, ui.window_opacity);
    let radius = ui.border_radius;
    move |_| container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            radius: iced::border::Radius {
                top_left: 0.0,
                top_right: 0.0,
                bottom_right: radius,
                bottom_left: radius,
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

/// セルの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Empty,
    Normal,
    Hovered,
    Pressed,
    DragSource,
    DropTarget,
    Focused,
}

pub fn cell(ui: &UiTheme, state: CellState) -> impl Fn(&Theme) -> container::Style {
    let radius = ui.border_radius_sm;
    let (bg, border_color, border_width) = match state {
        CellState::Empty => (ui.bg_button_empty, Color::TRANSPARENT, 0.0),
        CellState::Normal => (ui.bg_button, Color::TRANSPARENT, 0.0),
        CellState::Hovered => (ui.bg_button_hover, ui.border_color, 1.0),
        CellState::Pressed => (ui.bg_button_active, ui.accent, 1.0),
        CellState::DragSource => (with_alpha(ui.bg_button, 0.4), ui.text_muted, 1.0),
        CellState::DropTarget => (ui.bg_button_hover, ui.accent, 2.0),
        CellState::Focused => (ui.bg_button, ui.accent, 2.0),
    };
    move |_| container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            color: border_color,
            width: border_width,
            radius: radius.into(),
        },
        ..Default::default()
    }
}

/// タブボタン
pub fn tab_button(
    ui: &UiTheme,
    active: bool,
    drop_hover: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    let bg_active = ui.bg_button_active;
    let bg_idle = Color::TRANSPARENT;
    let bg_hover = ui.bg_button_hover;
    let text_active = ui.text_primary;
    let text_idle = ui.text_secondary;
    let accent = ui.accent;
    let radius = ui.border_radius_sm;
    move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        let bg = if active {
            bg_active
        } else if hovered || drop_hover {
            bg_hover
        } else {
            bg_idle
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: if active { text_active } else { text_idle },
            border: Border {
                color: if drop_hover {
                    accent
                } else {
                    Color::TRANSPARENT
                },
                width: if drop_hover { 1.0 } else { 0.0 },
                radius: radius.into(),
            },
            ..Default::default()
        }
    }
}

/// アイコンだけの小ボタン（タイトルバー等）
pub fn icon_button(
    ui: &UiTheme,
    highlighted: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    let hover_bg = ui.bg_button_hover;
    let text = if highlighted {
        ui.accent
    } else {
        ui.text_secondary
    };
    let radius = ui.border_radius_sm;
    move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: if hovered {
                Some(Background::Color(hover_bg))
            } else {
                None
            },
            text_color: text,
            border: Border {
                radius: radius.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// ピン留めトグルボタン。ON/OFF を背景・枠でハッキリ示す
/// （絵文字📌は色を変えられないため、背景で状態を表現する）。
pub fn pin_button(ui: &UiTheme, active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    let accent = ui.accent;
    let accent_soft = with_alpha(ui.accent, 0.20);
    let hover_bg = ui.bg_button_hover;
    let text = ui.text_secondary;
    let radius = ui.border_radius_sm;
    move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        // ON: アクセントの薄い塗り + アクセントの枠（囲まれてアクティブな見た目）
        // OFF: 背景なし（ホバー時のみ薄い背景）
        let bg = if active {
            Some(Background::Color(accent_soft))
        } else if hovered {
            Some(Background::Color(hover_bg))
        } else {
            None
        };
        button::Style {
            background: bg,
            text_color: text,
            border: Border {
                color: if active { accent } else { Color::TRANSPARENT },
                width: if active { 1.5 } else { 0.0 },
                radius: radius.into(),
            },
            ..Default::default()
        }
    }
}

/// 標準ボタン（ダイアログの保存/キャンセル等）。primary=true でアクセント色。
pub fn dialog_button(
    ui: &UiTheme,
    primary: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    let (bg, bg_hover, text) = if primary {
        (ui.accent, ui.accent_hover, ui.bg_primary)
    } else {
        (ui.bg_button, ui.bg_button_hover, ui.text_primary)
    };
    let radius = ui.border_radius_sm;
    move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(Background::Color(if hovered { bg_hover } else { bg })),
            text_color: text,
            border: Border {
                radius: radius.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// 危険操作ボタン
pub fn danger_button(ui: &UiTheme) -> impl Fn(&Theme, button::Status) -> button::Style {
    let bg = ui.danger;
    let radius = ui.border_radius_sm;
    move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(Background::Color(if hovered {
                with_alpha(bg, 0.85)
            } else {
                bg
            })),
            text_color: Color::WHITE,
            border: Border {
                radius: radius.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

/// メニュー項目ボタン（コンテキストメニュー）
pub fn menu_item(ui: &UiTheme, danger: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    let hover_bg = ui.bg_button_hover;
    let text = if danger { ui.danger } else { ui.text_primary };
    move |_, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: if hovered {
                Some(Background::Color(hover_bg))
            } else {
                None
            },
            text_color: text,
            ..Default::default()
        }
    }
}

/// オーバーレイのパネル（ダイアログ・メニューの箱）。
/// 影は付けない — tiny-skia では影がウィジェット bounds 外に描かれ、
/// メニュー/ダイアログの表示・消去やホバー再描画で残像化するため。
/// 立体感は border（枠）で表現する。
pub fn panel(ui: &UiTheme) -> impl Fn(&Theme) -> container::Style {
    let bg = ui.bg_secondary;
    let border = ui.border_color;
    let radius = ui.border_radius;
    move |_| container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            color: border,
            width: 1.0,
            radius: radius.into(),
        },
        ..Default::default()
    }
}

/// オーバーレイの背面幕
pub fn scrim() -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.35))),
        ..Default::default()
    }
}

/// テキスト入力
pub fn input(ui: &UiTheme) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    let bg = ui.bg_primary;
    let border = ui.border_color;
    let accent = ui.accent;
    let text = ui.text_primary;
    let placeholder = ui.text_muted;
    let radius = ui.border_radius_sm;
    move |_, status| {
        let focused = matches!(status, text_input::Status::Focused { .. });
        text_input::Style {
            background: Background::Color(bg),
            border: Border {
                color: if focused { accent } else { border },
                width: 1.0,
                radius: radius.into(),
            },
            icon: text,
            placeholder,
            value: text,
            selection: with_alpha(accent, 0.4),
        }
    }
}

/// ツールチップのパネル（影なし。影は tiny-skia で bounds 外に描かれ残像化するため）
pub fn tooltip_panel(ui: &UiTheme) -> impl Fn(&Theme) -> container::Style {
    let bg = ui.bg_secondary;
    let border = ui.border_color;
    let radius = ui.border_radius_sm;
    move |_| container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            color: border,
            width: 1.0,
            radius: radius.into(),
        },
        ..Default::default()
    }
}

/// トースト通知（影なし。tiny-skia の残像回避のため。枠で縁取る）
pub fn toast(ui: &UiTheme) -> impl Fn(&Theme) -> container::Style {
    let bg = ui.bg_secondary;
    let border = ui.accent;
    let radius = ui.border_radius;
    move |_| container::Style {
        background: Some(Background::Color(bg)),
        border: Border {
            color: border,
            width: 1.0,
            radius: radius.into(),
        },
        ..Default::default()
    }
}
