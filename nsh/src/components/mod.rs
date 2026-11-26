//! Shared component library, organized as atoms → molecules → organisms.
//!
//! Pages compose these primitives; they do not paint their own buttons,
//! inputs, cards, dialogs or selects.

pub mod atoms;
pub mod molecules;
pub mod organisms;

// Flat re-exports keep existing call sites stable while the tree stays atomic.
pub use atoms::{
    avatar, badge, button, checkbox, color_picker, input, logo_avatar, segmented, skeleton,
    textarea, toggle,
};
pub use molecules::{
    card, charge_days, compact_select, converted_amount, date_picker, empty_state, entity_form,
    entity_icon, field, list_toolbar, privacy_toggle, search_input, select, select_menu,
    selection_bar, string_select, timezone_select, toast,
};
pub use organisms::billing_schedule::{BillingScheduleProps, billing_schedule};
pub use organisms::{
    confirm, dialog, dropdown_menu, fab_menu, filter_menu, logo_search, page_header, sort_menu,
    theme_picker, web_server_qr,
};
