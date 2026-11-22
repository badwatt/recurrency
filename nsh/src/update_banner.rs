//! Dismissible banner for a newer GitHub release.

use nsh_base::{button, theme};
use nsh_core::prelude::*;

fn newer(current: &str, latest: &str) -> bool {
    let parse = |value: &str| {
        value
            .trim()
            .trim_start_matches('v')
            .split(['.', '-'])
            .take(3)
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    parse(latest) > parse(current)
}

pub fn update_banner() -> AnyElement {
    let release = resource(|| Some(()), |(), _| async { Ok::<_, String>(crate::api::latest_release().await) });
    let dismissed = signal(false);
    let visible = move || {
        !dismissed.get()
            && release
                .get()
                .flatten()
                .is_some_and(|info: crate::api::Release| newer(env!("CARGO_PKG_VERSION"), &info.tag_name))
    };
    when(visible, move || {
        let Some(info) = release.get().flatten() else {
            return view().h(px(0.0)).into_element();
        };
        let version = info.tag_name.trim_start_matches('v').to_string();
        let url = info.html_url.clone();
        view()
            .w_full()
            .items_center()
            .justify_center()
            .border_b(px(1.0))
            .border_color(move || theme().primary.opacity(0.3))
            .bg(move || theme().primary.opacity(0.1))
            .child(
                view()
                    .w_full()
                    .max_w(px(1280.0))
                    .px(px(24.0))
                    .py(px(8.0))
                    .flex_row()
                    .items_center()
                    .gap(px(10.0))
                    .child(
                        view()
                            .flex_col()
                            .flex_grow(1.0)
                            .min_w(px(0.0))
                            .child(text(format!("Recurrency {version} is available")).text_size(px(12.0)).font_weight(FontWeight::SEMIBOLD).text_color(move || theme().foreground))
                            .child(text(format!("You are on {}.", env!("CARGO_PKG_VERSION"))).text_size(px(10.0)).text_color(move || theme().muted_foreground)),
                    )
                    .child(crate::ui::action_button("View release", false, move || {
                        let url = url.clone();
                        spawn(async move {
                            let _ = open_url(url).await;
                        });
                    }))
                    .child(button().items_center().justify_center().h(px(26.0)).w(px(26.0)).rounded(px(5.0)).on_press(move || dismissed.set(true)).child(image(asset!("icons/x.svg")).size(px(13.0)).tint_color(move || theme().muted_foreground))),
            )
            .into_element()
    })
    .otherwise(|| view().h(px(0.0)).into_element())
    .into_element()
}
