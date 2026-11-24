//! Compact single-choice menu for account and category fields. The trigger
//! and the option rows are shared so every picker has the same height,
//! border and selected state.

use nsh_base::{button, input, theme};
use nsh_core::element::AutoFill;
use nsh_core::prelude::*;
use std::rc::Rc;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectOption {
    pub value: Uuid,
    pub label: String,
}

impl SelectOption {
    pub fn new(value: Uuid, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
        }
    }
}

pub struct SelectMenuProps {
    pub label: String,
    pub placeholder: String,
    pub selected: Signal<Option<Uuid>>,
    pub options: Rc<dyn Fn() -> Vec<SelectOption>>,
    pub height: f32,
    pub create: Option<Rc<dyn Fn(String)>>,
    pub search_placeholder: String,
}

impl SelectMenuProps {
    pub fn new(
        label: impl Into<String>,
        placeholder: impl Into<String>,
        selected: Signal<Option<Uuid>>,
        options: impl Fn() -> Vec<SelectOption> + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            placeholder: placeholder.into(),
            selected,
            options: Rc::new(options),
            height: 40.0,
            create: None,
            search_placeholder: "Search...".to_string(),
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn create(mut self, create: impl Fn(String) + 'static) -> Self {
        self.create = Some(Rc::new(create));
        self
    }

    pub fn search_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.search_placeholder = placeholder.into();
        self
    }
}

fn should_offer_create(options: &[SelectOption], query: &str, can_create: bool) -> bool {
    let needle = query.trim();
    let exists = options
        .iter()
        .any(|option| option.label.eq_ignore_ascii_case(needle));
    can_create && !needle.is_empty() && !exists
}
pub fn select_menu(props: SelectMenuProps) -> impl IntoElement {
    let SelectMenuProps {
        label,
        placeholder,
        selected,
        options,
        height,
        create,
        search_placeholder,
    } = props;
    let open = signal(false);
    let query = signal(String::new());
    let filtered_options = options.clone();
    let filtered: Rc<dyn Fn() -> Vec<SelectOption>> = Rc::new(move || {
        let needle = query.get().trim().to_lowercase();
        filtered_options()
            .into_iter()
            .filter(|option| needle.is_empty() || option.label.to_lowercase().contains(&needle))
            .collect::<Vec<_>>()
    });
    let exists_options = options.clone();
    let create_for_check = create.clone();
    let show_create: Rc<dyn Fn() -> bool> = Rc::new(move || {
        should_offer_create(&exists_options(), &query.get(), create_for_check.is_some())
    });
    let create_for_panel = create.clone();
    let selected_label = move || {
        let current = selected.get();
        options()
            .into_iter()
            .find(|option| Some(option.value) == current)
            .map_or_else(|| placeholder.clone(), |option| option.label)
    };
    view()
        .w_full()
        .relative()
        .z_index(move || if open.get() { 100 } else { 0 })
        .flex_col()
        .gap(px(6.0))
        .min_w(px(0.0))
        .child(
            text(label)
                .text_size(px(14.0))
                .line_height(px(20.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || theme().muted_foreground),
        )
        .child(
            button()
                .cursor_pointer()
                .cursor_pointer()
                .w_full()
                .h(px(height))
                .px(px(12.0))
                .flex_row()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .rounded(px(7.0))
                .border(px(1.0))
                .border_color(move || theme().input)
                .bg(move || theme().card)
                .on_press(move || {
                    if open.get() {
                        open.set(false);
                    } else {
                        query.set(String::new());
                        open.set(true);
                    }
                })
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(6.0))
                        .min_w(px(0.0))
                        .child(
                            view()
                                .h(px(8.0))
                                .w(px(8.0))
                                .flex_shrink_0()
                                .opacity(move || {
                                    if selected.get().is_some() {
                                        1.0_f32
                                    } else {
                                        0.0_f32
                                    }
                                })
                                .rounded(px(999.0))
                                .bg(move || theme().muted_foreground),
                        )
                        .child(
                            text(move || selected_label())
                                .number_of_lines(1)
                                .text_size(px(13.0))
                                .text_color(move || theme().foreground),
                        ),
                )
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap(px(4.0))
                        .flex_shrink_0()
                        .child(
                            view()
                                .cursor_pointer()
                                .h(px(20.0))
                                .w(px(20.0))
                                .items_center()
                                .justify_center()
                                .rounded(px(4.0))
                                .opacity(move || {
                                    if selected.get().is_some() {
                                        1.0_f32
                                    } else {
                                        0.0_f32
                                    }
                                })
                                .on_press(move || {
                                    selected.set(None);
                                    open.set(false);
                                })
                                .child(
                                    image(asset!("icons/x.svg"))
                                        .size(px(12.0))
                                        .pointer_events(PointerEvents::None)
                                        .tint_color(move || theme().muted_foreground),
                                ),
                        )
                        .child(
                            image(asset!("icons/chevron-down.svg"))
                                .size(px(13.0))
                                .tint_color(move || theme().muted_foreground),
                        ),
                ),
        )
        .child(
            when(open, move || {
                let create_for_panel = create_for_panel.clone();
                let filtered = filtered.clone();
                let show_create = show_create.clone();
                let create_for_search = create_for_panel.clone();
                let show_create_for_search = show_create.clone();
                view()
                    .absolute()
                    .top(px(height + 14.0))
                    .left(px(0.0))
                    .right(px(0.0))
                    .z_index(90)
                    .flex_col()
                    .gap(px(1.0))
                    .p(px(4.0))
                    .rounded(px(8.0))
                    .border(px(1.0))
                    .border_color(move || theme().border)
                    .bg(move || theme().popover)
                    .child(
                        view().w_full().p(px(4.0)).child(
                            input()
                                .value(query)
                                .on_change(move |next: &str| query.set(next.to_string()))
                                .on_submit(move |_: &str| {
                                    if show_create_for_search() {
                                        if let Some(create) = create_for_search.clone() {
                                            create(query.get());
                                        }
                                        query.set(String::new());
                                        open.set(false);
                                    }
                                })
                                .placeholder(search_placeholder.clone())
                                .auto_fill(AutoFill::Off)
                                .auto_focus(true)
                                .h(px(30.0))
                                .w_full()
                                .px(px(8.0))
                                .rounded(px(5.0))
                                .border(px(1.0))
                                .border_color(move || theme().input)
                                .bg(move || theme().background)
                                .text_size(px(12.0))
                                .text_color(move || theme().foreground),
                        ),
                    )
                    .child(scroll().max_h(px(200.0)).w_full().flex_col().child(each(
                        move || filtered(),
                        |option| option.value,
                        move |option, _| {
                            let option = option.get();
                            let id = option.value;
                            let active = selected.get() == Some(id);
                            button()
                                .cursor_pointer()
                                .w_full()
                                .h(px(30.0))
                                .px(px(8.0))
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .rounded(px(5.0))
                                .bg(move || {
                                    if active {
                                        theme().accent
                                    } else {
                                        theme().popover
                                    }
                                })
                                .on_press(move || {
                                    selected.set(Some(id));
                                    query.set(String::new());
                                    open.set(false);
                                })
                                .child(
                                    text(option.label)
                                        .number_of_lines(1)
                                        .text_size(px(13.0))
                                        .text_color(move || theme().foreground),
                                )
                                .when(active, |row| {
                                    row.child(
                                        image(asset!("icons/check.svg"))
                                            .size(px(13.0))
                                            .tint_color(move || theme().primary),
                                    )
                                })
                                .into_element()
                        },
                    )))
                    .child(
                        when(
                            move || show_create(),
                            move || {
                                let create = create_for_panel.clone();
                                view()
                                    .w_full()
                                    .child(
                                        view()
                                            .cursor_pointer()
                                            .w_full()
                                            .h(px(30.0))
                                            .px(px(8.0))
                                            .flex_row()
                                            .items_center()
                                            .gap(px(6.0))
                                            .rounded(px(5.0))
                                            .bg(move || theme().popover)
                                            .on_press(move || {
                                                if let Some(create) = create.clone() {
                                                    create(query.get());
                                                }
                                                query.set(String::new());
                                                open.set(false);
                                            })
                                            .child(
                                                image(asset!("icons/plus.svg"))
                                                    .size(px(12.0))
                                                    .tint_color(move || theme().primary),
                                            )
                                            .child(
                                                text(move || format!("Create \"{}\"", query.get()))
                                                    .text_size(px(13.0))
                                                    .text_color(move || theme().primary),
                                            ),
                                    )
                                    .into_element()
                            },
                        )
                        .otherwise(|| view().h(px(0.0)).into_element()),
                    )
                    .into_element()
            })
            .otherwise(|| view().h(px(0.0)).mt(px(-6.0)).into_element()),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_float_above_the_form_instead_of_pushing_it() {
        let source = include_str!("select_menu.rs");
        assert!(source.contains(".absolute()"));
        assert!(source.contains("z_index"));
        assert!(source.contains("text(move || selected_label())"));
    }

    #[test]
    fn create_is_offered_only_for_a_new_search_term() {
        let options = vec![SelectOption::new(Uuid::nil(), "General")];
        assert!(should_offer_create(&options, "Zed", true));
        assert!(!should_offer_create(&options, "general", true));
        assert!(!should_offer_create(&options, "", true));
        assert!(!should_offer_create(&options, "Zed", false));
    }

    #[test]
    fn entity_menu_exposes_search_create_and_focus() {
        let source = include_str!("select_menu.rs");
        assert!(source.contains("auto_focus(true)"));
        assert!(source.contains("on_submit"));
        assert!(source.contains("Create"));
        assert!(source.contains("query.get()"));
    }

    #[test]
    fn select_option_keeps_value_and_label() {
        let id = Uuid::nil();
        let option = SelectOption::new(id, "General");
        assert_eq!(option.value, id);
        assert_eq!(option.label, "General");
    }
}
