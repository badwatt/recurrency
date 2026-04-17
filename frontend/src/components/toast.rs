use dioxus::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static TOAST_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToastItem {
    pub id: usize,
    pub kind: ToastKind,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ToastState {
    pub items: Vec<ToastItem>,
}

impl ToastState {
    pub fn success(&mut self, msg: impl Into<String>) {
        let id = TOAST_COUNTER.fetch_add(1, Ordering::Relaxed);
        self.items.push(ToastItem {
            id,
            kind: ToastKind::Success,
            message: msg.into(),
        });
    }

    pub fn error(&mut self, msg: impl Into<String>) {
        let id = TOAST_COUNTER.fetch_add(1, Ordering::Relaxed);
        self.items.push(ToastItem {
            id,
            kind: ToastKind::Error,
            message: msg.into(),
        });
    }

    pub fn remove(&mut self, id: usize) {
        self.items.retain(|t| t.id != id);
    }
}

pub static TOAST: GlobalSignal<ToastState> = Signal::global(ToastState::default);

#[component]
pub fn ToastContainer() -> Element {
    let items = TOAST().items.clone();

    rsx! {
        div { class: "fixed top-4 right-4 z-[100] flex flex-col gap-2 max-w-sm",
            for item in items {
                ToastEntry { key: "{item.id}", item: item.clone() }
            }
        }
    }
}

#[component]
fn ToastEntry(item: ToastItem) -> Element {
    let _visible = use_signal(|| true);
    let id = item.id;

    // Auto-dismiss after 3s
    spawn(async move {
        #[cfg(target_arch = "wasm32")]
        gloo_timers::future::TimeoutFuture::new(3000).await;
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
        TOAST.write().remove(id);
    });

    let (bg, text_color) = match item.kind {
        ToastKind::Success => ("bg-green-600", "text-white"),
        ToastKind::Error => ("bg-red-600", "text-white"),
    };

    rsx! {
        div { class: "flex items-center gap-2 {bg} {text_color} px-4 py-3 rounded-md shadow-lg text-sm",
            span { class: "flex-1", {item.message.clone()} }
            button {
                r#type: "button",
                class: "text-white/70 hover:text-white",
                onclick: move |_| {
                    TOAST.write().remove(id);
                },
                "✕"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toast_state_success() {
        let mut state = ToastState::default();
        state.success("ok");
        assert_eq!(state.items.len(), 1);
        assert!(matches!(state.items[0].kind, ToastKind::Success));
    }

    #[test]
    fn toast_state_error() {
        let mut state = ToastState::default();
        state.error("fail");
        assert_eq!(state.items.len(), 1);
        assert!(matches!(state.items[0].kind, ToastKind::Error));
    }

    #[test]
    fn toast_state_remove() {
        let mut state = ToastState::default();
        state.success("a");
        state.success("b");
        let id = state.items[0].id;
        state.remove(id);
        assert_eq!(state.items.len(), 1);
    }

    #[test]
    fn toast_ids_unique() {
        let mut state = ToastState::default();
        state.success("a");
        state.success("b");
        assert_ne!(state.items[0].id, state.items[1].id);
    }

    #[test]
    fn toast_messages_preserved() {
        let mut state = ToastState::default();
        state.success("saved");
        state.error("failed to save");
        assert_eq!(state.items[0].message, "saved");
        assert_eq!(state.items[1].message, "failed to save");
    }

    #[test]
    fn toast_remove_nonexistent_id() {
        let mut state = ToastState::default();
        state.success("a");
        state.remove(9999); // nonexistent
        assert_eq!(state.items.len(), 1);
    }

    #[test]
    fn toast_remove_all_one_by_one() {
        let mut state = ToastState::default();
        state.success("a");
        state.success("b");
        state.success("c");
        for item in state.items.clone() {
            state.remove(item.id);
        }
        assert!(state.items.is_empty());
    }

    #[test]
    fn toast_kind_equality() {
        assert_eq!(ToastKind::Success, ToastKind::Success);
        assert_eq!(ToastKind::Error, ToastKind::Error);
        assert_ne!(ToastKind::Success, ToastKind::Error);
    }

    #[test]
    fn toast_default_empty() {
        let state = ToastState::default();
        assert!(state.items.is_empty());
    }

    // ── ToastContainer component test ──
    #[test]
    fn toast_container_function_exists() {
        // Verify ToastContainer is properly defined and public
        // This is a compile-time check - if ToastContainer doesn't exist or has wrong signature, this won't compile
        let _container_fn = ToastContainer;
    }

    // ── Toast remove functionality tests ──
    #[test]
    fn toast_state_remove_existing_item() {
        // RED: Add toast, then remove it - should be empty
        let mut state = ToastState::default();
        state.success("Test message");
        assert_eq!(state.items.len(), 1);
        
        let id = state.items[0].id;
        state.remove(id);
        
        assert_eq!(state.items.len(), 0);
    }

    #[test]
    fn toast_state_remove_preserves_other_items() {
        // RED: Add 3 toasts, remove middle one - others stay
        let mut state = ToastState::default();
        state.success("First");
        state.success("Second");
        state.success("Third");
        assert_eq!(state.items.len(), 3);
        
        let second_id = state.items[1].id;
        state.remove(second_id);
        
        assert_eq!(state.items.len(), 2);
        assert!(state.items.iter().any(|t| t.message == "First"));
        assert!(state.items.iter().any(|t| t.message == "Third"));
        assert!(!state.items.iter().any(|t| t.message == "Second"));
    }

    #[test]
    fn toast_state_remove_nonexistent_id_does_nothing() {
        // RED: Remove non-existent id - state unchanged
        let mut state = ToastState::default();
        state.success("Test");
        let original_len = state.items.len();
        
        state.remove(9999); // Non-existent ID
        
        assert_eq!(state.items.len(), original_len);
    }

    #[test]
    fn toast_state_can_add_and_remove_multiple_times() {
        // RED: Add/remove cycle multiple times
        let mut state = ToastState::default();
        
        state.success("Toast 1");
        let id1 = state.items[0].id;
        state.remove(id1);
        assert!(state.items.is_empty());
        
        state.error("Toast 2");
        let id2 = state.items[0].id;
        state.remove(id2);
        assert!(state.items.is_empty());
    }
}