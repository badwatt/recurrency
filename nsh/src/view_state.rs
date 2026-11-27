//! Which list layout each collection uses, saved on the device.

use nsh_core::storage::Storage;

const SUBSCRIPTIONS_KEY: &str = "recurrency.subscriptions-view";
const TRANSACTIONS_KEY: &str = "recurrency.transactions-view.v3";
const MONEY_KEY: &str = "recurrency.money-view";
const MONEY_SORT_FIELD_KEY: &str = "recurrency.money-sort-field";
const MONEY_SORT_DESC_KEY: &str = "recurrency.money-sort-desc";

fn value(key: &str, default: &str) -> String {
    Storage::get_item(key).unwrap_or_else(|| default.to_string())
}

pub fn subscriptions_list() -> bool {
    value(SUBSCRIPTIONS_KEY, "grid") == "list"
}

pub fn save_subscriptions_list(list: bool) {
    let _ = Storage::set_item(SUBSCRIPTIONS_KEY, if list { "list" } else { "grid" });
}

pub fn transactions_list() -> bool {
    value(TRANSACTIONS_KEY, "list") == "list"
}

pub fn save_transactions_list(list: bool) {
    let _ = Storage::set_item(TRANSACTIONS_KEY, if list { "list" } else { "grid" });
}

pub fn money_list() -> bool {
    value(MONEY_KEY, "grid") == "list"
}

pub fn save_money_list(list: bool) {
    let _ = Storage::set_item(MONEY_KEY, if list { "list" } else { "grid" });
}

/// The money list opens ordered by what charges first.
pub fn money_sort_field() -> String {
    value(MONEY_SORT_FIELD_KEY, "date")
}

pub fn money_sort_descending() -> bool {
    value(MONEY_SORT_DESC_KEY, "asc") == "desc"
}

pub fn save_money_sort(field: String, descending: bool) {
    let _ = Storage::set_item(MONEY_SORT_FIELD_KEY, field);
    let _ = Storage::set_item(MONEY_SORT_DESC_KEY, if descending { "desc" } else { "asc" });
}
