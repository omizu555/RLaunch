//! フォルダブラウザポップアップのビュー。

use crate::app::{layout, App, Message};
use crate::ui::style;
use iced::widget::{button, column, container, mouse_area, row, scrollable, text, Space};
use iced::{Alignment, Element, Length};

fn ext_emoji(name: &str, is_dir: bool) -> &'static str {
    if is_dir {
        return "📁";
    }
    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "exe" | "msi" | "bat" | "cmd" => "⚙",
        "txt" | "md" | "log" => "📄",
        "pdf" => "📕",
        "doc" | "docx" => "📘",
        "xls" | "xlsx" | "csv" => "📗",
        "ppt" | "pptx" => "📙",
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" => "🖼",
        "mp3" | "wav" | "flac" | "ogg" | "m4a" => "🎵",
        "mp4" | "avi" | "mkv" | "mov" | "webm" => "🎬",
        "zip" | "rar" | "7z" | "tar" | "gz" => "📦",
        "lnk" | "url" => "🔗",
        _ => "📄",
    }
}

fn format_size(size: u64) -> String {
    if size >= 1 << 30 {
        format!("{:.1} GB", size as f64 / (1u64 << 30) as f64)
    } else if size >= 1 << 20 {
        format!("{:.1} MB", size as f64 / (1u64 << 20) as f64)
    } else if size >= 1 << 10 {
        format!("{:.1} KB", size as f64 / (1u64 << 10) as f64)
    } else {
        format!("{} B", size)
    }
}

pub fn view(app: &App) -> Element<'_, Message> {
    let ui = &app.ui;
    let Some(folder) = &app.folder else {
        return text("").into();
    };

    let title = folder
        .current
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| folder.current.to_string_lossy().into_owned());

    let header = row![
        button(text("⬆").size(12))
            .style(style::icon_button(ui, false))
            .padding([2, 8])
            .on_press(Message::FolderUp),
        container(
            text(title)
                .size(12)
                .color(ui.text_secondary)
                .wrapping(text::Wrapping::None),
        )
        .padding([0, 6])
        .width(Length::Fill),
        button(text("🗔 Explorer").size(11))
            .style(style::icon_button(ui, false))
            .padding([2, 8])
            .on_press(Message::FolderOpenExplorer),
        button(text("✕").size(11))
            .style(style::icon_button(ui, false))
            .padding([2, 8])
            .on_press(Message::FolderClose),
    ]
    .align_y(Alignment::Center)
    .padding([0, 4]);

    let mut list = column![].spacing(1);
    for (i, e) in folder.entries.iter().enumerate() {
        let size_text = if e.is_dir {
            String::new()
        } else {
            format_size(e.size)
        };
        list = list.push(
            button(
                row![
                    text(ext_emoji(&e.name, e.is_dir)).size(13),
                    text(e.name.clone())
                        .size(12)
                        .color(ui.text_primary)
                        .wrapping(text::Wrapping::None)
                        .width(Length::Fill),
                    text(size_text).size(10).color(ui.text_muted),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .style(style::menu_item(ui, false))
            .padding([4, 10])
            .width(Length::Fill)
            .on_press(Message::FolderEntryClicked(i)),
        );
    }
    if folder.entries.is_empty() {
        list = list
            .push(container(text("（空のフォルダ）").size(11).color(ui.text_muted)).padding(12));
    }

    let base = column![
        container(header)
            .height(Length::Fixed(layout::POPUP_HEADER_HEIGHT))
            .width(Length::Fill)
            .style(style::bar(ui)),
        container(scrollable(list).height(Length::Fill))
            .padding(6)
            .width(Length::Fill)
            .height(Length::Fill),
        Space::new().height(4.0),
    ];

    mouse_area(
        container(base)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::window_root(ui)),
    )
    .on_press(Message::Noop)
    .into()
}
