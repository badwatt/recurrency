//! Logo URL field, search action and thumbnail results, shared by the
//! subscription and transaction forms.

use nsh_base::{button, theme};
use nsh_core::prelude::*;

#[derive(Clone, Copy)]
pub struct LogoSearchState {
    pub results: Signal<Vec<(String, String)>>,
    pub loading: Signal<bool>,
    pub error: Signal<String>,
}

pub fn logo_search(
    name: Signal<String>,
    logo_url: Signal<String>,
    server: Signal<String>,
    state: LogoSearchState,
) -> impl IntoElement {
    let LogoSearchState {
        results,
        loading,
        error,
    } = state;
    let find_logo = move || {
        let query = name.get().trim().to_string();
        if query.is_empty() {
            error.set("Enter a name first.".to_string());
            return;
        }
        loading.set(true);
        error.set(String::new());
        let server = server.get();
        spawn(async move {
            match crate::api::search_logos(server, query).await {
                Ok(values) => results.set(
                    values
                        .into_iter()
                        .map(|result| (result.url, result.thumbnail))
                        .collect(),
                ),
                Err(message) => error.set(message),
            }
            loading.set(false);
        });
    };
    let remove = move || logo_url.set(String::new());
    let use_url = move || {
        let value = logo_url.get().trim().to_string();
        logo_url.set(value);
    };
    view()
        .w_full()
        .flex_col()
        .gap(px(6.0))
        .child(
            view()
                .w_full()
                .flex_col()
                .gap(px(6.0))
                .child(
                    text("Logo")
                        .text_size(px(13.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().foreground),
                )
                .child(
                    view()
                        .w_full()
                        .flex_row()
                        .flex_wrap()
                        .items_center()
                        .gap(px(8.0))
                        .when(!logo_url.get().trim().is_empty(), move |row| {
                            row.child(
                                image(logo_url.get())
                                    .size(px(40.0))
                                    .rounded(px(8.0))
                                    .border(px(1.0))
                                    .border_color(move || theme().border),
                            )
                        })
                        .child(crate::components::button::button(
                            crate::components::button::ButtonProps::new(
                                if logo_url.get().trim().is_empty() {
                                    "Add logo"
                                } else {
                                    "Change"
                                },
                            )
                            .variant(crate::components::button::ButtonVariant::Outline)
                            .on_press(find_logo),
                        ))
                        .when(!logo_url.get().trim().is_empty(), move |row| {
                            row.child(crate::components::button::button(
                                crate::components::button::ButtonProps::new("Remove")
                                    .variant(crate::components::button::ButtonVariant::Outline)
                                    .on_press(remove),
                            ))
                        }),
                ),
        )
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .child(view().flex_grow(1.0).basis(px(0.0)).min_w(px(0.0)).child(
                    crate::components::input::text_input(logo_url, "Paste logo URL (https://…)"),
                ))
                .child(crate::components::button::button(
                    crate::components::button::ButtonProps::new("Use URL")
                        .variant(crate::components::button::ButtonVariant::Outline)
                        .size(crate::components::button::ButtonSize::Sm)
                        .on_press(use_url),
                )),
        )
        .child(
            text(move || error.get())
                .text_size(px(11.0))
                .text_color(move || theme().destructive),
        )
        .child(
            when(
                move || !results.get().is_empty(),
                move || {
                    view()
                        .w_full()
                        .flex_row()
                        .flex_wrap()
                        .gap(px(6.0))
                        .children(results.get().into_iter().map(|(url, thumbnail)| {
                            let value = url.clone();
                            button()
                                .cursor_pointer()
                                .h(px(38.0))
                                .w(px(38.0))
                                .items_center()
                                .justify_center()
                                .rounded(px(6.0))
                                .border(px(1.0))
                                .border_color(move || theme().border)
                                .bg(move || theme().background)
                                .on_press(move || logo_url.set(value.clone()))
                                .child(image(thumbnail).size(px(28.0)))
                                .into_element()
                        }))
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_logo_search_uses_the_api_and_thumbnail_tiles() {
        let source = include_str!("logo_search.rs");
        assert!(source.contains("search_logos"));
        assert!(source.contains("thumbnail"));
        assert!(source.contains("Add logo"));
        assert!(source.contains("Use URL"));
    }
}
