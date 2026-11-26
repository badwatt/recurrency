//! Placeholder surface while a resource loads.

use nsh_base::theme;
use nsh_core::prelude::*;

pub fn skeleton(height: f32) -> impl IntoElement {
    view()
        .w_full()
        .h(px(height))
        .rounded(px(8.0))
        .bg(move || theme().muted.opacity(0.5))
}
