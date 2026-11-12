use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CreateIntent {
    Subscription,
    Transaction,
    Debt,
}

pub static CREATE_INTENT: GlobalSignal<Option<CreateIntent>> = Signal::global(|| None);
