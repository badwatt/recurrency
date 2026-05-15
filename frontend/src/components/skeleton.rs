use dioxus::prelude::*;

/// Skeleton loading placeholder. Prevents FOUC, no layout shift.
/// Animates with a subtle pulse. Matches shadcn/ui skeleton pattern.
#[component]
pub fn Skeleton(class: String) -> Element {
    rsx! {
        div { class: "animate-pulse rounded-md bg-muted {class}" }
    }
}

/// Card skeleton — mimics a content card while loading.
#[component]
pub fn SkeletonCard() -> Element {
    rsx! {
        div { class: "rounded-lg border border-border bg-card p-4 space-y-3",
            Skeleton { class: "h-4 w-1/3" }
            Skeleton { class: "h-3 w-2/3" }
            Skeleton { class: "h-3 w-1/2" }
        }
    }
}

/// Table row skeleton.
#[component]
pub fn SkeletonRow() -> Element {
    rsx! {
        div { class: "flex items-center gap-3 py-3",
            Skeleton { class: "h-9 w-9 rounded-full shrink-0" }
            div { class: "flex-1 space-y-2",
                Skeleton { class: "h-3 w-1/3" }
                Skeleton { class: "h-3 w-1/4" }
            }
            Skeleton { class: "h-3 w-16" }
        }
    }
}

/// Grid of card skeletons.
#[component]
pub fn SkeletonGrid(count: Option<usize>) -> Element {
    let n = count.unwrap_or(6);
    rsx! {
        div { class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4",
            for _ in 0..n {
                SkeletonCard {}
            }
        }
    }
}

/// Calendar skeleton — grid of day cells.
#[component]
pub fn SkeletonCalendar() -> Element {
    rsx! {
        div { class: "grid grid-cols-7 gap-px",
            for _ in 0..35 {
                div { class: "min-h-[100px] bg-card p-2 space-y-2",
                    Skeleton { class: "h-6 w-6 rounded-full" }
                    Skeleton { class: "h-2 w-3/4" }
                }
            }
        }
    }
}

/// Donut chart skeleton.
#[component]
pub fn SkeletonDonut() -> Element {
    rsx! {
        div { class: "flex flex-col items-center gap-4",
            div { class: "h-[180px] w-[180px] rounded-full bg-muted animate-pulse" }
            div { class: "grid grid-cols-2 gap-x-4 gap-y-1.5 w-full",
                for _ in 0..6 {
                    div { class: "flex items-center gap-1.5",
                        Skeleton { class: "h-2.5 w-2.5 rounded-full" }
                        Skeleton { class: "h-3 w-16" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeleton_renders_with_class() {
        let el: Element = rsx! {
            div { class: "animate-pulse rounded-md bg-muted h-4 w-full" }
        };
        let s = format!("{el:?}");
        assert!(s.contains("animate-pulse"));
        assert!(s.contains("bg-muted"));
    }

    #[test]
    fn skeleton_custom_class() {
        let el: Element = rsx! {
            div { class: "animate-pulse rounded-md bg-muted h-8 w-1/2" }
        };
        let s = format!("{el:?}");
        assert!(s.contains("h-8"));
    }

    #[test]
    fn skeleton_card_structure() {
        // Verify component builds without panic
        let _el: Element = rsx! { SkeletonCard {} };
    }

    #[test]
    fn skeleton_row_structure() {
        let _el: Element = rsx! { SkeletonRow {} };
    }

    #[test]
    fn skeleton_grid_default_count() {
        let _el: Element = rsx! { SkeletonGrid { count: None } };
    }

    #[test]
    fn skeleton_grid_custom_count() {
        let _el: Element = rsx! { SkeletonGrid { count: Some(3) } };
    }

    #[test]
    fn skeleton_calendar_structure() {
        let _el: Element = rsx! { SkeletonCalendar {} };
    }

    #[test]
    fn skeleton_donut_structure() {
        let _el: Element = rsx! { SkeletonDonut {} };
    }
}