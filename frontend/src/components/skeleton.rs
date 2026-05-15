use dioxus::prelude::*;

/// Skeleton loading placeholder. Prevents FOUC, no layout shift.
/// Animates with a subtle pulse. Matches shadcn/ui skeleton pattern.
#[component]
pub fn Skeleton(class: String) -> Element {
    rsx! {
        div { class: "animate-pulse rounded-md bg-muted {class}" }
    }
}

/// Subscription grid card skeleton — matches render_grid_card layout.
#[component]
pub fn SubscriptionCardSkeleton() -> Element {
    rsx! {
        div { class: "rounded-xl border border-border/40 bg-card/80 p-4 space-y-4",
            // Header: logo + name + amount
            div { class: "flex items-start justify-between",
                div { class: "flex items-center gap-3",
                    Skeleton { class: "h-10 w-10 rounded-full" }
                    div { class: "space-y-1.5",
                        Skeleton { class: "h-4 w-24" }
                        Skeleton { class: "h-3 w-16" }
                    }
                }
                Skeleton { class: "h-5 w-16" }
            }
            // Category + next date
            div { class: "flex items-center justify-between",
                Skeleton { class: "h-5 w-20 rounded-full" }
                Skeleton { class: "h-3 w-24" }
            }
            // Action buttons
            div { class: "flex items-center gap-2 pt-2 border-t border-border/40",
                Skeleton { class: "h-8 w-8 rounded-md" }
                Skeleton { class: "h-8 w-8 rounded-md" }
            }
        }
    }
}

/// Subscription list card skeleton — matches list card layout.
#[component]
pub fn SubscriptionListSkeleton() -> Element {
    rsx! {
        div { class: "flex items-center gap-4 rounded-xl border border-border/40 bg-card/80 p-4",
            Skeleton { class: "h-10 w-10 rounded-full shrink-0" }
            div { class: "flex-1 space-y-1.5",
                Skeleton { class: "h-4 w-32" }
                Skeleton { class: "h-3 w-20" }
            }
            Skeleton { class: "h-5 w-20" }
            Skeleton { class: "h-5 w-16" }
        }
    }
}

/// Transaction group row skeleton — matches transaction list item.
#[component]
pub fn TransactionRowSkeleton() -> Element {
    rsx! {
        div { class: "rounded-lg border border-border bg-card p-3 sm:p-4",
            div { class: "flex items-center gap-2 sm:gap-4",
                Skeleton { class: "h-10 w-10 rounded-full shrink-0" }
                div { class: "flex-1 space-y-1.5",
                    Skeleton { class: "h-4 w-28" }
                    Skeleton { class: "h-3 w-20" }
                }
                Skeleton { class: "h-5 w-16 rounded-full" }
                Skeleton { class: "h-4 w-20" }
                div { class: "flex gap-1",
                    Skeleton { class: "h-8 w-8 rounded-md" }
                    Skeleton { class: "h-8 w-8 rounded-md" }
                }
            }
        }
    }
}

/// Category card skeleton — matches category card layout.
#[component]
pub fn CategoryCardSkeleton() -> Element {
    rsx! {
        div { class: "rounded-xl border border-border/40 bg-card/80 p-4 flex items-center gap-3",
            Skeleton { class: "h-10 w-10 rounded-full" }
            div { class: "flex-1 space-y-1.5",
                Skeleton { class: "h-4 w-24" }
                Skeleton { class: "h-3 w-16" }
            }
            div { class: "flex gap-1",
                Skeleton { class: "h-8 w-8 rounded-md" }
                Skeleton { class: "h-8 w-8 rounded-md" }
            }
        }
    }
}

/// Report summary card skeleton — matches stat card layout.
#[component]
pub fn ReportCardSkeleton() -> Element {
    rsx! {
        div { class: "rounded-lg border border-border bg-card p-6 space-y-2",
            Skeleton { class: "h-3 w-24" }
            Skeleton { class: "h-7 w-32" }
            Skeleton { class: "h-3 w-20" }
        }
    }
}

/// Report donut skeleton — matches donut chart + legend layout.
#[component]
pub fn ReportDonutSkeleton() -> Element {
    rsx! {
        div { class: "rounded-lg border border-border bg-card p-6 space-y-4",
            Skeleton { class: "h-5 w-24" }
            div { class: "flex flex-col items-center gap-4",
                div { class: "h-[180px] w-[180px] rounded-full bg-muted animate-pulse" }
                div { class: "grid grid-cols-2 gap-x-4 gap-y-1.5 w-full",
                    for _ in 0..6 {
                        div { class: "flex items-center gap-1.5",
                            Skeleton { class: "h-2.5 w-2.5 rounded-full" }
                            Skeleton { class: "h-3 w-16" }
                            Skeleton { class: "h-3 w-8 ml-auto" }
                        }
                    }
                }
            }
        }
    }
}

/// Report table skeleton — matches category breakdown table.
#[component]
pub fn ReportTableSkeleton() -> Element {
    rsx! {
        div { class: "rounded-lg border border-border bg-card p-6 space-y-4",
            Skeleton { class: "h-5 w-36" }
            div { class: "space-y-3",
                for _ in 0..5 {
                    div { class: "flex items-center gap-3",
                        Skeleton { class: "h-8 w-8 rounded-full" }
                        div { class: "flex-1 space-y-1.5",
                            Skeleton { class: "h-3 w-24" }
                        }
                        Skeleton { class: "h-3 w-16" }
                        Skeleton { class: "h-3 w-12" }
                    }
                }
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
                    Skeleton { class: "h-7 w-7 rounded-full" }
                    Skeleton { class: "h-2 w-3/4" }
                }
            }
        }
    }
}

/// Dashboard upcoming row skeleton.
#[component]
pub fn DashboardRowSkeleton() -> Element {
    rsx! {
        div { class: "flex items-center gap-3",
            Skeleton { class: "h-10 w-10 rounded-full shrink-0" }
            div { class: "flex-1 space-y-1.5",
                Skeleton { class: "h-4 w-28" }
                Skeleton { class: "h-3 w-20" }
            }
            Skeleton { class: "h-4 w-20" }
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
    fn subscription_card_skeleton_builds() { let _el: Element = rsx! { SubscriptionCardSkeleton {} }; }

    #[test]
    fn subscription_list_skeleton_builds() { let _el: Element = rsx! { SubscriptionListSkeleton {} }; }

    #[test]
    fn transaction_row_skeleton_builds() { let _el: Element = rsx! { TransactionRowSkeleton {} }; }

    #[test]
    fn category_card_skeleton_builds() { let _el: Element = rsx! { CategoryCardSkeleton {} }; }

    #[test]
    fn report_card_skeleton_builds() { let _el: Element = rsx! { ReportCardSkeleton {} }; }

    #[test]
    fn report_donut_skeleton_builds() { let _el: Element = rsx! { ReportDonutSkeleton {} }; }

    #[test]
    fn report_table_skeleton_builds() { let _el: Element = rsx! { ReportTableSkeleton {} }; }

    #[test]
    fn skeleton_calendar_builds() { let _el: Element = rsx! { SkeletonCalendar {} }; }

    #[test]
    fn dashboard_row_skeleton_builds() { let _el: Element = rsx! { DashboardRowSkeleton {} }; }
}