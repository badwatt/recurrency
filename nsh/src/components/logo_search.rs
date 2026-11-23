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
    view()
        .w_full()
        .flex_col()
        .gap(px(6.0))
        .child(
            view()
                .w_full()
                .flex_row()
                .items_end()
                .gap(px(8.0))
                .child(crate::ui::field(
                    "Logo URL",
                    logo_url,
                    "https://example.com/logo.png",
                ))
                .child(crate::ui::action_button(
                    if loading.get() {
                        "Searching…"
                    } else {
                        "Find logo"
                    },
                    false,
                    find_logo,
                )),
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
        .child(
            text(move || error.get())
                .text_size(px(11.0))
                .text_color(move || theme().destructive),
        )
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_logo_search_uses_the_api_and_thumbnail_tiles() {
        let source = include_str!("logo_search.rs");
        assert!(source.contains("search_logos"));
        assert!(source.contains("thumbnail"));
        assert!(source.contains("Find logo"));
    }
}
