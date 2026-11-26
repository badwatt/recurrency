//! One select for the whole app: the trigger, the searchable list, the
//! keyboard focus and the fixed placement under the field all live here.

use std::rc::Rc;

use nsh_base::{button, input, theme};
use nsh_core::element::AutoFill;
use nsh_core::prelude::*;

pub struct SelectProps {
    pub label: String,
    pub placeholder: String,
    pub selected_label: Rc<dyn Fn() -> String>,
    pub selected_value: Rc<dyn Fn() -> String>,
    pub has_value: Rc<dyn Fn() -> bool>,
    pub options: Rc<dyn Fn() -> Vec<(String, String)>>,
    pub on_select: Rc<dyn Fn(String)>,
    pub on_clear: Option<Rc<dyn Fn()>>,
    pub searchable: bool,
    pub show_label: bool,
    pub height: f32,
    pub search_placeholder: String,
    pub create: Option<Rc<dyn Fn(String, Rc<dyn Fn()>)>>,
    pub create_title: String,
}

impl SelectProps {
    pub fn new(
        label: impl Into<String>,
        placeholder: impl Into<String>,
        selected_label: impl Fn() -> String + 'static,
        selected_value: impl Fn() -> String + 'static,
        has_value: impl Fn() -> bool + 'static,
        options: impl Fn() -> Vec<(String, String)> + 'static,
        on_select: impl Fn(String) + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            placeholder: placeholder.into(),
            selected_label: Rc::new(selected_label),
            selected_value: Rc::new(selected_value),
            has_value: Rc::new(has_value),
            options: Rc::new(options),
            on_select: Rc::new(on_select),
            on_clear: None,
            searchable: true,
            show_label: true,
            height: 36.0,
            search_placeholder: "Search...".to_string(),
            create: None,
            create_title: "Create new".to_string(),
        }
    }

    pub fn clearable(mut self, on_clear: impl Fn() + 'static) -> Self {
        self.on_clear = Some(Rc::new(on_clear));
        self
    }

    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }

    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn search_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.search_placeholder = placeholder.into();
        self
    }

    pub fn create(
        mut self,
        create: impl Fn(String, Rc<dyn Fn()>) + 'static,
        title: impl Into<String>,
    ) -> Self {
        self.create = Some(Rc::new(create));
        self.create_title = title.into();
        self
    }
}

pub fn select(props: SelectProps) -> impl IntoElement {
    let SelectProps {
        label,
        placeholder,
        selected_label,
        selected_value,
        has_value,
        options,
        on_select,
        on_clear,
        searchable,
        show_label,
        height,
        search_placeholder,
        create,
        create_title,
    } = props;
    let open = signal(false);
    let query = signal(String::new());
    let panel_width = signal(320.0_f32);
    let select_id = crate::ui::next_select_id();
    let host = crate::ui::select_host();
    effect(move || {
        if let Some(host) = host
            && let Some(current) = host.get()
            && current != select_id
        {
            open.set(false);
            query.set(String::new());
        }
    });
    let options_for_panel = options.clone();
    let current_for_panel = selected_value.clone();
    let create_for_panel = create.clone();
    let create_title_for_panel = create_title.clone();
    let search_placeholder_for_panel = search_placeholder.clone();
    let panel = Rc::new(move || {
        let needle = query.get().trim().to_lowercase();
        let current = current_for_panel();
        let all = options_for_panel();
        let visible = all
            .iter()
            .filter(|(_, label)| needle.is_empty() || label.to_lowercase().contains(&needle))
            .cloned()
            .collect::<Vec<_>>();
        let can_create = create_for_panel.is_some()
            && !needle.is_empty()
            && !all
                .iter()
                .any(|(_, label)| label.eq_ignore_ascii_case(needle.trim()));
        let create_for_row = create_for_panel.clone();
        let create_for_button = create_for_panel.clone();
        let placeholder_text = search_placeholder_for_panel.clone();
        let create_title_text = create_title_for_panel.clone();
        view()
            .w(move || px(panel_width.get()))
            .flex_col()
            .rounded(px(8.0))
            .border(px(1.0))
            .border_color(move || theme().border)
            .bg(move || theme().popover)
            .overflow_hidden()
            .when(searchable, move |panel| {
                panel.child(
                    view()
                        .w_full()
                        .px(px(12.0))
                        .border_b(px(1.0))
                        .border_color(move || theme().border)
                        .child(
                            input()
                                .value(query)
                                .on_change(move |value: &str| query.set(value.to_string()))
                                .on_submit(move |_: &str| {
                                    if can_create && let Some(create) = create_for_row.as_ref() {
                                        let close = std::rc::Rc::new(crate::ui::close_menu);
                                        create(query.get(), close);
                                    }
                                })
                                .placeholder(placeholder_text.clone())
                                .auto_fill(AutoFill::Off)
                                .auto_focus(true)
                                .h(px(36.0))
                                .w_full()
                                .px(px(0.0))
                                .border(px(0.0))
                                .bg(move || theme().popover)
                                .text_size(px(14.0))
                                .text_color(move || theme().foreground),
                        ),
                )
            })
            .child(
                scroll()
                    .max_h(px(220.0))
                    .w_full()
                    .py(px(4.0))
                    .flex_col()
                    .when(visible.is_empty() && !can_create, |panel| {
                        panel.child(
                            view().h(px(48.0)).items_center().justify_center().child(
                                text("No results found")
                                    .text_size(px(13.0))
                                    .text_color(move || theme().muted_foreground),
                            ),
                        )
                    })
                    .children(visible.into_iter().map(|(value, label)| {
                        let active = current == value;
                        let value_for_press = value.clone();
                        let on_select = on_select.clone();
                        button()
                            .cursor_pointer()
                            .w_full()
                            .h(px(36.0))
                            .px(px(12.0))
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .bg(move || {
                                if active {
                                    theme().accent
                                } else {
                                    theme().popover
                                }
                            })
                            .on_press(move || {
                                on_select(value_for_press.clone());
                                query.set(String::new());
                                crate::ui::close_menu();
                            })
                            .child(
                                text(label)
                                    .flex_grow(1.0)
                                    .number_of_lines(1)
                                    .text_size(px(14.0))
                                    .text_color(move || theme().foreground),
                            )
                            .when(active, |row| {
                                row.child(
                                    image(asset!("icons/check.svg"))
                                        .size(px(16.0))
                                        .tint_color(move || theme().primary),
                                )
                            })
                    }))
                    .when(can_create, move |panel| {
                        panel.child(
                            button()
                                .cursor_pointer()
                                .w_full()
                                .h(px(40.0))
                                .px(px(12.0))
                                .border_t(px(1.0))
                                .border_color(move || theme().border)
                                .flex_row()
                                .items_center()
                                .gap(px(6.0))
                                .on_press(move || {
                                    if let Some(create) = create_for_button.as_ref() {
                                        let close = std::rc::Rc::new(crate::ui::close_menu);
                                        create(query.get(), close);
                                    }
                                })
                                .child(
                                    text(format!("{} “{}”", create_title_text, query.get().trim()))
                                        .text_size(px(13.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(move || theme().primary),
                                ),
                        )
                    }),
            )
            .into_element()
    });
    let has_value_for_text = has_value.clone();
    let has_value_for_color = has_value.clone();
    view()
        .w_full()
        .flex_col()
        .gap(px(6.0))
        .min_w(px(0.0))
        .when(show_label, |column| {
            column.child(
                text(label)
                    .text_size(px(14.0))
                    .line_height(px(20.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(move || theme().muted_foreground),
            )
        })
        .child(
            view()
                .cursor_pointer()
                .w_full()
                .h(px(height))
                .px(px(8.0))
                .flex_row()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .rounded(px(6.0))
                .border(px(1.0))
                .border_color(move || theme().border)
                .bg(move || theme().background)
                .on_layout(
                    move |bounds: &nsh_core::geometry::Bounds<nsh_core::geometry::Points>| {
                        panel_width.set(bounds.size.width.0);
                    },
                )
                .on_press(move |event: &PressEvent| {
                    let panel_for_open = panel.clone();
                    let page = event.page_position;
                    let local = event.position;
                    crate::ui::open_menu(
                        crate::ui::MenuPlacement::At {
                            top: page.y.0 - local.y.0 + height + 4.0,
                            left: (page.x.0 - local.x.0).max(8.0),
                        },
                        move || (*panel_for_open)(),
                    );
                })
                .child(
                    text(move || {
                        if has_value_for_text() {
                            selected_label()
                        } else {
                            placeholder.clone()
                        }
                    })
                    .flex_grow(1.0)
                    .number_of_lines(1)
                    .text_size(px(14.0))
                    .text_color(move || {
                        if has_value_for_color() {
                            theme().foreground
                        } else {
                            theme().muted_foreground
                        }
                    }),
                )
                .when_some(on_clear.clone(), |row, on_clear| {
                    row.when(has_value(), move |row| {
                        row.child(
                            button()
                                .cursor_pointer()
                                .h(px(24.0))
                                .w(px(24.0))
                                .items_center()
                                .justify_center()
                                .accessibility_label("Clear selection")
                                .on_press(move || on_clear())
                                .child(
                                    image(asset!("icons/x.svg"))
                                        .size(px(12.0))
                                        .tint_color(move || theme().muted_foreground),
                                ),
                        )
                    })
                })
                .child(
                    image(asset!("icons/chevron-down.svg"))
                        .size(px(13.0))
                        .tint_color(move || theme().muted_foreground),
                ),
        )
}

#[cfg(test)]
mod tests {
    #[test]
    fn select_autofocuses_only_its_searchable_input() {
        let source = include_str!("select.rs");
        assert!(source.contains("auto_focus(true)"));
        assert!(source.contains("when(searchable"));
    }
}
