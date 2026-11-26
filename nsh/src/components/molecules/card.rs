//! Shared card surfaces and section headers.

use nsh_base::theme;
use nsh_core::prelude::*;

pub fn card() -> nsh_core::element::View {
    view()
        .w_full()
        .rounded(px(crate::theme::CARD_RADIUS))
        .border(px(1.0))
        .border_color(move || theme().border)
        .bg(move || theme().card)
}

pub fn section_card(
    title: &'static str,
    subtitle: &'static str,
    action: AnyElement,
) -> impl IntoElement {
    card().p(px(20.0)).gap(px(8.0)).child(
        view()
            .w_full()
            .flex_row()
            .items_start()
            .justify_between()
            .gap(px(12.0))
            .child(
                view()
                    .flex_col()
                    .gap(px(3.0))
                    .child(
                        text(title)
                            .text_size(px(15.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(move || theme().foreground),
                    )
                    .child(
                        text(subtitle)
                            .text_size(px(12.0))
                            .text_color(move || theme().muted_foreground),
                    ),
            )
            .child(action),
    )
}
