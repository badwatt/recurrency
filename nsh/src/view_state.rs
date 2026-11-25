//! Which list layout each collection uses, saved on the device.

use nsh_core::storage::Storage;

const SUBSCRIPTIONS_KEY: &str = "recurrency.subscriptions-view";
const TRANSACTIONS_KEY: &str = "recurrency.transactions-view";

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
