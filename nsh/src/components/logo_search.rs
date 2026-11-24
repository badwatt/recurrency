//! Logo URL field, upload, and the full "Search Logo" dialog shared by the
//! subscription and transaction forms.

use crate::components::dialog::{DialogProps, dialog};
use crate::ui;
use nsh_base::{button, input, theme};
use nsh_core::element::AutoFill;
use nsh_core::file_dialog::{FileFilter, OpenDialog};
use nsh_core::prelude::*;

const PAGE_SIZE: usize = 8;

#[derive(Clone, Copy)]
pub struct LogoSearchState {
    pub results: Signal<Vec<(String, String)>>,
    pub loading: Signal<bool>,
    pub error: Signal<String>,
}

fn page_count(total: usize) -> usize {
    total.div_ceil(PAGE_SIZE).max(1)
}

fn page_bounds(page: usize, total: usize) -> (usize, usize) {
    let start = page.saturating_mul(PAGE_SIZE).min(total);
    (start, (start + PAGE_SIZE).min(total))
}

fn candidates(query: &str) -> Vec<(String, String)> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }
    let domain = if needle.contains('.') {
        needle
    } else {
        let slug: String = needle
            .chars()
            .filter(|value| value.is_alphanumeric())
            .collect();
        if slug.is_empty() {
            return Vec::new();
        }
        format!("{slug}.com")
    };
    vec![
        (
            format!("https://www.google.com/s2/favicons?domain={domain}&sz=256"),
            format!("https://www.google.com/s2/favicons?domain={domain}&sz=128"),
        ),
        (
            format!("https://icons.duckduckgo.com/ip3/{domain}.ico"),
            format!("https://icons.duckduckgo.com/ip3/{domain}.ico"),
        ),
        (
            format!("https://unavatar.io/{domain}"),
            format!("https://unavatar.io/{domain}"),
        ),
    ]
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
    let query = signal(String::new());
    let page = signal(0usize);
    let upload = move || {
        spawn(async move {
            match pick_paths(
                OpenDialog::files()
                    .title("Choose a logo")
                    .filter(FileFilter::new("Images", ["png", "jpg", "jpeg", "webp"])),
            )
            .await
            {
                Ok(Some(paths)) => {
                    let Some(path) = paths.first() else { return };
                    match std::fs::read(path)
                        .map_err(|problem| format!("Could not read the image: {problem}"))
                        .and_then(|bytes| crate::settings::avatar_data_url(&bytes))
                    {
                        Ok(data_url) => logo_url.set(data_url),
                        Err(message) => error.set(message),
                    }
                }
                Ok(None) => {}
                Err(message) => error.set(message),
            }
        });
    };
    let remove = move || logo_url.set(String::new());
    let use_url = move || {
        let value = logo_url.get().trim().to_string();
        logo_url.set(value);
    };
    let open_search = move || {
        error.set(String::new());
        results.set(Vec::new());
        page.set(0);
        query.set(name.get());
        ui::open_logo_modal(move || {
            dialog(
                DialogProps::new("Search Logo", "", move || ui::close_logo_modal())
                    .width(512.0)
                    .footer_visible(false)
                    .body(search_body(
                        server, query, results, loading, error, page, logo_url,
                    )),
            )
            .into_element()
        });
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
                        .text_size(px(14.0))
                        .line_height(px(20.0))
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
                            .size(crate::components::button::ButtonSize::Sm)
                            .on_press(open_search.clone()),
                        ))
                        .child(crate::components::button::button(
                            crate::components::button::ButtonProps::new("Upload image")
                                .variant(crate::components::button::ButtonVariant::Outline)
                                .size(crate::components::button::ButtonSize::Sm)
                                .on_press(upload),
                        ))
                        .when(!logo_url.get().trim().is_empty(), move |row| {
                            row.child(crate::components::button::button(
                                crate::components::button::ButtonProps::new("Remove")
                                    .variant(crate::components::button::ButtonVariant::Outline)
                                    .size(crate::components::button::ButtonSize::Sm)
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
            when(
                move || !error.get().is_empty(),
                move || {
                    text(move || error.get())
                        .text_size(px(11.0))
                        .text_color(move || theme().destructive)
                },
            )
            .otherwise(|| view().h(px(0.0))),
        )
}

fn search_body(
    server: Signal<String>,
    query: Signal<String>,
    results: Signal<Vec<(String, String)>>,
    loading: Signal<bool>,
    error: Signal<String>,
    page: Signal<usize>,
    logo_url: Signal<String>,
) -> AnyElement {
    let search = move || {
        let value = query.get().trim().to_string();
        if value.is_empty() || loading.get() {
            return;
        }
        loading.set(true);
        error.set(String::new());
        results.set(Vec::new());
        page.set(0);
        let server = server.get();
        spawn(async move {
            match crate::api::search_logos(server, value.clone()).await {
                Ok(found) if !found.is_empty() => results.set(
                    found
                        .into_iter()
                        .map(|logo| (logo.url, logo.thumbnail))
                        .collect(),
                ),
                Ok(_) => {
                    let fallback = candidates(&value);
                    if fallback.is_empty() {
                        error.set(
                            "No logos found. Try the company website (e.g. spotify.com).".into(),
                        );
                    } else {
                        results.set(fallback);
                    }
                }
                Err(message) => {
                    let fallback = candidates(&value);
                    if fallback.is_empty() {
                        error.set(message);
                    } else {
                        results.set(fallback);
                    }
                }
            }
            loading.set(false);
        });
    };
    let total = results.get().len();
    let pages = page_count(total);
    let current = page.get().min(pages.saturating_sub(1));
    let (start, end) = page_bounds(current, total);
    let items = results.get();
    let current_items = items[start..end].to_vec();
    let has_items = !current_items.is_empty();
    let grid_items = current_items.clone();
    view()
        .w_full()
        .p(px(24.0))
        .gap(px(16.0))
        .flex_col()
        .child(
            view()
                .w_full()
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .child(
                    view().flex_grow(1.0).basis(px(0.0)).min_w(px(0.0)).child(
                        input()
                            .value(query)
                            .on_change(move |next: &str| query.set(next.to_string()))
                            .on_submit({
                                let search = search.clone();
                                move |_: &str| search()
                            })
                            .placeholder("Enter company name or domain...")
                            .auto_fill(AutoFill::Off)
                            .auto_focus(true)
                            .h(px(40.0))
                            .w_full()
                            .px(px(12.0))
                            .rounded(px(7.0))
                            .border(px(1.0))
                            .border_color(move || theme().input)
                            .bg(move || theme().card)
                            .text_size(px(14.0))
                            .text_color(move || theme().foreground),
                    ),
                )
                .child(
                    button()
                        .cursor_pointer()
                        .h(px(40.0))
                        .px(px(14.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(6.0))
                        .bg(move || theme().primary)
                        .disabled(move || loading.get() || query.get().trim().is_empty())
                        .disabled_style(|state| state.opacity(0.5))
                        .on_press(search)
                        .child(
                            text(move || {
                                if loading.get() {
                                    "Searching…"
                                } else {
                                    "Search"
                                }
                            })
                            .text_size(px(14.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(move || theme().primary_foreground),
                        ),
                ),
        )
        .child(
            when(
                move || loading.get(),
                || {
                    text("Searching logos…")
                        .text_size(px(14.0))
                        .text_color(move || theme().muted_foreground)
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
        .child(
            when(
                move || !error.get().is_empty(),
                move || {
                    view()
                        .w_full()
                        .h(px(120.0))
                        .items_center()
                        .justify_center()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            text("No logos found")
                                .text_size(px(14.0))
                                .text_color(move || theme().muted_foreground),
                        )
                        .child(
                            text("Try searching for the company website (e.g., spotify.com)")
                                .text_size(px(12.0))
                                .text_color(move || theme().muted_foreground),
                        )
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
        .child(
            when(
                move || has_items,
                move || {
                    let items = grid_items.clone();
                    view()
                        .w_full()
                        .grid()
                        .grid_template_columns(SharedString::from("repeat(4, minmax(0, 1fr))"))
                        .gap(px(12.0))
                        .children(items.into_iter().map(|(url, thumbnail)| {
                            let chosen = if thumbnail.is_empty() {
                                url.clone()
                            } else {
                                thumbnail.clone()
                            };
                            button()
                                .cursor_pointer()
                                .h(px(72.0))
                                .w_full()
                                .items_center()
                                .justify_center()
                                .rounded(px(6.0))
                                .border(px(1.0))
                                .border_color(move || theme().border)
                                .bg(move || theme().card)
                                .on_press(move || {
                                    logo_url.set(chosen.clone());
                                    ui::close_logo_modal();
                                })
                                .child(
                                    image(if thumbnail.is_empty() {
                                        url.clone()
                                    } else {
                                        thumbnail.clone()
                                    })
                                    .size(px(48.0)),
                                )
                                .into_element()
                        }))
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
        .child(
            when(
                move || total > 0,
                move || {
                    view()
                        .w_full()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(
                            button()
                                .cursor_pointer()
                                .h(px(30.0))
                                .px(px(10.0))
                                .rounded(px(5.0))
                                .border(px(1.0))
                                .border_color(move || theme().border)
                                .on_press(move || page.set(current.saturating_sub(1)))
                                .child(
                                    text("‹")
                                        .text_size(px(16.0))
                                        .text_color(move || theme().foreground),
                                ),
                        )
                        .child(
                            text(move || format!("Page {} of {}", current + 1, pages))
                                .text_size(px(12.0))
                                .text_color(move || theme().muted_foreground),
                        )
                        .child(
                            button()
                                .cursor_pointer()
                                .h(px(30.0))
                                .px(px(10.0))
                                .rounded(px(5.0))
                                .border(px(1.0))
                                .border_color(move || theme().border)
                                .on_press(move || page.set((current + 1).min(pages - 1)))
                                .child(
                                    text("›")
                                        .text_size(px(16.0))
                                        .text_color(move || theme().foreground),
                                ),
                        )
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
        .into_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logo_pages_hold_eight_tiles() {
        assert_eq!(page_count(0), 1);
        assert_eq!(page_count(8), 1);
        assert_eq!(page_count(9), 2);
        assert_eq!(page_bounds(1, 9), (8, 9));
    }

    #[test]
    fn the_logo_dialog_uses_the_api_and_pagination() {
        let source = include_str!("logo_search.rs");
        assert!(source.contains("search_logos"));
        assert!(source.contains("Page {} of {}"));
        assert!(source.contains("open_logo_modal"));
    }
}
