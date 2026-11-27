//! The ambient colour a card takes from its logo: the image itself, blurred
//! and enlarged behind the content, dimmed so the text stays readable. It is
//! the native version of the frontend's logo wash.

use nsh_base::theme;
use nsh_core::prelude::*;

/// Blur radius, saturation and opacity the wash is drawn with.
const BLUR: f32 = 52.0;
const SATURATION: f32 = 1.7;
const SCALE: f32 = 1.35;
const OPACITY: f32 = 0.28;
/// The veil over it, so a bright logo cannot wash the text out.
const VEIL: f32 = 0.75;

/// The wash for `logo`, filling the card it is placed in. Cards must be
/// `relative` (and clip) for it to cover them.
pub fn logo_wash(logo: Option<String>) -> AnyElement {
    let Some(url) = logo.filter(|url| !url.trim().is_empty()) else {
        return view().h(px(0.0)).into_element();
    };
    view()
        .absolute()
        .inset_0()
        .child(
            image(url)
                .w_full()
                .h_full()
                .object_fit(ResizeMode::Cover)
                .blur(px(BLUR))
                .saturate(SATURATION)
                .scale(SCALE)
                .opacity(OPACITY),
        )
        .child(
            view()
                .absolute()
                .inset_0()
                .bg(move || theme().card.opacity(VEIL)),
        )
        .into_element()
}
