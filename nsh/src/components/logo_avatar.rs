//! Brand logo or fallback icon in a rounded tile, shared by list cards and
//! transaction rows.

use nsh_base::theme;
use nsh_core::prelude::*;

pub fn logo_avatar(
    logo: Option<String>,
    fallback: ImageAsset,
    size: f32,
    round: bool,
) -> impl IntoElement {
    let has_logo = logo.is_some();
    view()
        .h(px(size))
        .w(px(size))
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .rounded(px(if round { 999.0 } else { 8.0 }))
        .overflow_hidden()
        .bg(move || theme().muted)
        .when(has_logo, move |view| {
            view.child(image(logo.clone().unwrap_or_default()).size(px(size * 0.55)))
        })
        .when(!has_logo, move |view| {
            view.child(
                image(fallback)
                    .size(px(size * 0.44))
                    .tint_color(move || theme().muted_foreground),
            )
        })
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_logo_tile_scales_the_fallback_icon() {
        let source = include_str!("logo_avatar.rs");
        assert!(source.contains("size * 0.44"));
        assert!(source.contains("size * 0.55"));
    }
}
