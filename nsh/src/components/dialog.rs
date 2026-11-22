//! One dialog host for every form and confirmation. Screens register a
//! builder at the shell root; the host re-renders it while it is open.

use nsh_core::prelude::*;
use std::rc::Rc;

pub type ModalBuilder = Rc<dyn Fn() -> AnyElement>;

#[derive(Clone, Copy)]
pub struct DialogHost {
    slot: Stored<Option<ModalBuilder>>,
    version: Signal<u64>,
}

impl DialogHost {
    pub fn new() -> Self {
        Self { slot: stored(None), version: signal(0) }
    }

    pub fn open(&self, build: impl Fn() -> AnyElement + 'static) {
        self.slot.replace(Some(Rc::new(build)));
        self.version.set(self.version.get().wrapping_add(1));
    }

    pub fn close(&self) {
        self.slot.replace(None);
        self.version.set(self.version.get().wrapping_add(1));
    }

    pub fn active(&self) -> bool {
        self.version.get();
        self.slot.with(|slot| slot.is_some())
    }

    pub fn render(&self) -> AnyElement {
        self.slot
            .with(|slot| slot.clone())
            .map_or_else(|| view().h(px(0.0)).into_element(), |build| build())
    }
}

pub fn dialog_overlay(modal: DialogHost) -> AnyElement {
    view()
        .absolute()
        .inset_0()
        .z_index(80)
        .items_center()
        .justify_center()
        .p(px(24.0))
        .bg(rgba(0x000000cc))
        .child(modal.render())
        .into_element()
}
