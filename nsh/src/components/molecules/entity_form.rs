//! Name, icon and colour form shared by Settings and the inline entity
//! creation flow of Account/Category selects.

use nsh_base::{button, theme};
use nsh_core::prelude::*;

use crate::components::color_picker::{color_picker, color_value};
use crate::components::entity_icon::entity_icon;
use crate::components::field::{FieldProps, field};
use crate::components::input::text_input;

pub const ENTITY_ICONS: &[&str] = &[
    "wallet",
    "tag",
    "receipt",
    "card",
    "subscriptions",
    "transactions",
    "debts",
    "calendar",
    "reports",
    "settings",
    "database",
    "bell",
    "clock",
    "trending",
    "notes",
];

pub struct EntityFormProps {
    pub name: Signal<String>,
    pub icon: Signal<String>,
    pub color: Signal<String>,
    pub error: Signal<String>,
    pub preview_label: String,
}

pub fn entity_form(props: EntityFormProps) -> AnyElement {
    let EntityFormProps {
        name,
        icon,
        color,
        error,
        preview_label,
    } = props;
    view()
        .w_full()
        .flex_col()
        .gap(px(16.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .gap(px(12.0))
                .p(px(12.0))
                .rounded(px(10.0))
                .border(px(1.0))
                .border_color(move || theme().border.opacity(0.6))
                .bg(move || theme().muted.opacity(0.18))
                .child(entity_icon(&icon.get(), color_value(&color.get()), 42.0))
                .child(
                    view()
                        .flex_col()
                        .gap(px(2.0))
                        .child(
                            text(preview_label)
                                .text_size(px(11.0))
                                .letter_spacing(px(1.2))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().muted_foreground),
                        )
                        .child(
                            text(move || {
                                let value = name.get();
                                if value.trim().is_empty() {
                                    "Preview".to_string()
                                } else {
                                    value
                                }
                            })
                            .text_size(px(16.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(move || theme().foreground),
                        ),
                ),
        )
        .child(field(
            FieldProps::new("Name").required(),
            text_input(name, "Name").into_element(),
        ))
        .child(entity_icon_field(icon, color))
        .child(
            view()
                .w_full()
                .flex_col()
                .gap(px(8.0))
                .child(
                    text("Color")
                        .text_size(px(14.0))
                        .line_height(px(20.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().foreground),
                )
                .child(color_picker(color)),
        )
        .child(
            when(
                move || !error.get().is_empty(),
                move || {
                    text(move || error.get())
                        .text_size(px(12.0))
                        .text_color(move || theme().destructive)
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
        .into_element()
}

fn entity_icon_field(icon: Signal<String>, color: Signal<String>) -> AnyElement {
    let open = signal(false);
    view()
        .w_full()
        .flex_col()
        .gap(px(8.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    text("Icon")
                        .text_size(px(14.0))
                        .line_height(px(20.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().foreground),
                )
                .child(
                    button()
                        .cursor_pointer()
                        .h(px(32.0))
                        .px(px(10.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(6.0))
                        .border(px(1.0))
                        .border_color(move || theme().border)
                        .bg(move || theme().background)
                        .on_press(move || open.set(!open.get()))
                        .child(
                            text(move || {
                                if open.get() {
                                    "Hide icons"
                                } else {
                                    "Choose icon"
                                }
                            })
                            .text_size(px(12.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(move || theme().foreground),
                        ),
                ),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .p(px(10.0))
                .rounded(px(8.0))
                .border(px(1.0))
                .border_color(move || theme().border.opacity(0.5))
                .child(entity_icon(&icon.get(), color_value(&color.get()), 34.0))
                .child(
                    text(move || icon.get())
                        .text_size(px(12.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .child(
            when(
                move || open.get(),
                move || {
                    view()
                        .w_full()
                        .grid()
                        .grid_template_columns(SharedString::from("repeat(8, minmax(0, 1fr))"))
                        .gap(px(4.0))
                        .p(px(8.0))
                        .rounded(px(8.0))
                        .border(px(1.0))
                        .border_color(move || theme().border)
                        .bg(move || theme().background)
                        .children(ENTITY_ICONS.iter().map(|name| {
                            let value = (*name).to_string();
                            let active_value = value.clone();
                            let active = move || icon.get() == active_value;
                            button()
                                .cursor_pointer()
                                .h(px(32.0))
                                .w_full()
                                .items_center()
                                .justify_center()
                                .rounded(px(5.0))
                                .border(px(1.0))
                                .border_color(move || {
                                    if active() {
                                        theme().primary
                                    } else {
                                        theme().border
                                    }
                                })
                                .on_press(move || icon.set(value.clone()))
                                .child(entity_icon(name, color_value(&color.get()), 16.0))
                                .into_element()
                        }))
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
        .into_element()
}
