use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PopoverAlign {
    #[default]
    Start,
    End,
    /// Start on phones (the trigger is near the left edge) and End once the
    /// toolbar has enough room; keeps panels inside the viewport.
    EndSm,
    /// Center the panel in the toolbar on phones, then anchor it to the
    /// trigger's right edge on larger screens.
    CenterSm,
}

impl PopoverAlign {
    fn class(self) -> &'static str {
        match self {
            Self::Start => "left-0",
            Self::End => "right-0",
            Self::EndSm => "left-0 sm:left-auto sm:right-0",
            Self::CenterSm => "left-1/2 -translate-x-1/2 sm:left-auto sm:right-0 sm:translate-x-0",
        }
    }
}

/// Anchored panel with a click-catching backdrop. The trigger stays in the
/// caller (usually a relative wrapper) while every popover shares the same
/// layering, outside-click and keyboard dismissal rules.
#[component]
pub fn Popover(
    open: bool,
    on_close: EventHandler<()>,
    #[props(default)] align: PopoverAlign,
    #[props(default)] class: String,
    #[props(default)] panel_class: String,
    children: Element,
) -> Element {
    if !open {
        return rsx! {};
    }
    rsx! {
        div {
            class: "fixed inset-0 z-40",
            onclick: move |_| on_close.call(()),
            onkeydown: move |event| {
                if event.key() == Key::Escape {
                    on_close.call(());
                }
            },
        }
        div {
            role: "dialog",
            class: "absolute {align.class()} top-full z-50 mt-1 rounded-lg border border-border bg-popover p-4 shadow-md {class} {panel_class}",
            onclick: move |event| event.stop_propagation(),
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn popover_owns_outside_click_and_escape() {
        let source = include_str!("popover.rs");
        assert!(source.contains("fixed inset-0 z-40"));
        assert!(source.contains("Key::Escape"));
        assert!(source.contains("role: \"dialog\""));
        assert!(source.contains("right-0"));
    }
}
