//! Shared native shell primitives. They mirror the current Tailwind shapes
//! without copying markup: one navigation bar, one surface, one page header.

use crate::Route;
use nsh_base::{button, input, theme};
use nsh_core::element::AutoFill;
use uuid::Uuid;
use nsh_core::prelude::*;

pub fn top_bar(route: Signal<Route>) -> impl IntoElement {
    view()
        .h(px(56.0))
        .w_full()
        .flex_row()
        .items_center()
        .justify_center()
        .border_b(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
        .child(
            view()
                .h_full()
                .w_full()
                .max_w(px(1280.0))
                .flex_row()
                .items_center()
                .gap(px(4.0))
                .px(px(24.0))
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(10.0))
                        .mr(px(24.0))
                        .child(image(crate::LOGO).size(px(26.0)))
                        .child(
                            text("Recurrency")
                                .text_size(px(18.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(move || theme().foreground),
                        ),
                )
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(2.0))
                        .children(Route::NAV.iter().map(|item| nav_item(*item, route))),
                ),
        )
}

fn nav_item(item: Route, route: Signal<Route>) -> impl IntoElement {
    let active = move || route.get() == item;
    button()
        .h(px(36.0))
        .px(px(12.0))
        .rounded(px(8.0))
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .bg(move || if active() { theme().accent } else { theme().card })
        .active(|s| s.bg(move || theme().muted))
        .on_press(move || route.set(item))
        .child(
            image(item.icon())
                .size(px(18.0))
                .tint_color(move || if active() {
                    theme().foreground
                } else {
                    theme().muted_foreground
                }),
        )
        .child(
            text(item.label())
                .text_size(px(14.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || if active() {
                    theme().foreground
                } else {
                    theme().muted_foreground
                }),
        )
}

pub fn page_header(title: &'static str, subtitle: &'static str) -> impl IntoElement {
    view()
        .w_full()
        .flex_col()
        .gap(px(4.0))
        .child(
            text(title)
                .text_size(px(34.0))
                .font_weight(FontWeight::BOLD)
                .text_color(move || theme().foreground),
        )
        .child(
            text(subtitle)
                .text_size(px(15.0))
                .text_color(move || theme().muted_foreground),
        )
}

pub fn surface() -> nsh_core::element::View {
    view()
        .w_full()
        .rounded(px(crate::theme::CARD_RADIUS))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
}

pub fn pending_screen(title: &'static str) -> AnyElement {
    view()
        .w_full()
        .flex_col()
        .gap(px(crate::theme::PAGE_GAP))
        .child(page_header(title, "Native Shell migration in progress"))
        .child(
            surface()
                .p(px(24.0))
                .child(
                    text("This screen is being ported from the Dioxus interface.")
                        .text_size(px(15.0))
                        .text_color(move || theme().muted_foreground),
                ),
        )
        .into_element()
}

pub fn count_label(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("1 {singular}")
    } else {
        format!("{count} {plural}")
    }
}

pub fn label(text_value: &'static str) -> impl IntoElement {
    text(text_value)
        .text_size(px(12.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(move || theme().muted_foreground)
}

pub fn field(
    label_text: &'static str,
    value: Signal<String>,
    placeholder: &'static str,
) -> impl IntoElement {
    view()
        .flex_grow(1.0)
        .flex_col()
        .gap(px(6.0))
        .min_w(px(0.0))
        .child(label(label_text))
        .child(
            input()
                .value(value)
                .on_change(move |next: &str| value.set(next.to_string()))
                .placeholder(placeholder)
                .auto_fill(AutoFill::Off)
                .h(px(38.0))
                .px(px(10.0))
                .rounded(px(7.0))
                .border(px(1.0))
                .border_color(move || theme().input)
                .bg(move || theme().card)
                .text_size(px(13.0))
                .text_color(move || theme().foreground),
        )
}

pub fn choice_picker(
    label_text: &'static str,
    items: impl Fn() -> Vec<(Uuid, String)> + Clone + 'static,
    selected: Signal<Option<Uuid>>,
) -> impl IntoElement {
    view()
        .flex_col()
        .gap(px(6.0))
        .child(label(label_text))
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(6.0))
                .child(each(
                    items,
                    |(id, _)| *id,
                    move |entry, _| {
                        let (id, name) = entry.get();
                        let active = selected.get() == Some(id);
                        button()
                            .h(px(30.0))
                            .px(px(10.0))
                            .rounded(px(6.0))
                            .border(px(1.0))
                            .border_color(move || if active { theme().primary } else { theme().border })
                            .bg(move || if active { theme().accent } else { theme().card })
                            .on_press(move || selected.set(Some(id)))
                            .child(
                                text(name)
                                    .text_size(px(12.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(move || if active {
                                        theme().foreground
                                    } else {
                                        theme().muted_foreground
                                    }),
                            )
                    },
                )),
        )
}
