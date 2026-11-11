use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CreateIntent {
    Subscription,
    Transaction,
}

pub static CREATE_INTENT: GlobalSignal<Option<CreateIntent>> = Signal::global(|| None);
