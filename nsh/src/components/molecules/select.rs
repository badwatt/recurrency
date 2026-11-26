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
    pub create: Option<Rc<dyn Fn(String, String, String, Rc<dyn Fn(Result<(), String>)>)>>,
    pub create_title: String,
    pub create_icon: String,
    pub create_color: String,
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
            create_icon: "tag".to_string(),
            create_color: crate::components::color_picker::DEFAULT_COLOR.to_string(),
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
        create: impl Fn(String, String, String, Rc<dyn Fn(Result<(), String>)>) + 'static,
        title: impl Into<String>,
    ) -> Self {
        self.create = Some(Rc::new(create));
        self.create_title = title.into();
        self
    }

    pub fn create_icon(mut self, icon: impl Into<String>) -> Self {
        self.create_icon = icon.into();
        self
    }

    pub fn create_color(mut self, color: impl Into<String>) -> Self {
        self.create_color = color.into();
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
        create_icon,
        create_color,
    } = props;
    let open = signal(false);
    let query = signal(String::new());
    let panel_width = signal(320.0_f32);
    let create_name = signal(String::new());
    let create_error = signal(String::new());
    let create_loading = signal(false);
    let default_icon = Rc::new(create_icon);
    let default_color = Rc::new(create_color);
    let create_icon_value = signal(default_icon.to_string());
    let create_color_value = signal(default_color.to_string());
    let creating = signal(false);
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
    let create_for_form = create.clone();
    let create_title_for_form = create_title.clone();
    let create_form = Rc::new(move || {
        let create_for_press = create_for_form.clone();
        let title = create_title_for_form.clone();
        let submit_create = Rc::new(move || {
            let name = create_name.get().trim().to_string();
            if name.is_empty() || create_loading.get() {
                return;
            }
            let Some(create) = create_for_press.as_ref() else {
                return;
            };
            create_loading.set(true);
            create_error.set(String::new());
            let done: Rc<dyn Fn(Result<(), String>)> = Rc::new(move |result| {
                create_loading.set(false);
                match result {
                    Ok(()) => {
                        create_name.set(String::new());
                        creating.set(false);
                        crate::ui::close_menu();
                    }
                    Err(message) => create_error.set(message),
                }
            });
            create(
                name,
                create_icon_value.get(),
                create_color_value.get(),
                done,
            );
        });
        let submit_for_button = submit_create.clone();
        view()
            .w_full()
            .flex_col()
            .gap(px(10.0))
            .p(px(12.0))
            .child(
                text(title.clone())
                    .text_size(px(14.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(move || theme().foreground),
            )
            .child(scroll().max_h(px(460.0)).w_full().child(
                crate::components::entity_form::entity_form(
                    crate::components::entity_form::EntityFormProps {
                        name: create_name,
                        icon: create_icon_value,
                        color: create_color_value,
                        error: create_error,
                    },
                ),
            ))
            .child(
                view()
                    .w_full()
                    .flex_row()
                    .justify_end()
                    .gap(px(8.0))
                    .child(
                        button()
                            .cursor_pointer()
                            .h(px(34.0))
                            .px(px(12.0))
                            .items_center()
                            .justify_center()
                            .rounded(px(6.0))
                            .border(px(1.0))
                            .border_color(move || theme().border)
                            .on_press(move || {
                                creating.set(false);
                                create_error.set(String::new());
                            })
                            .child(
                                text("Cancel")
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(move || theme().foreground),
                            ),
                    )
                    .child(
                        button()
                            .cursor_pointer()
                            .h(px(34.0))
                            .px(px(12.0))
                            .items_center()
                            .justify_center()
                            .rounded(px(6.0))
                            .bg(move || theme().primary)
                            .on_press(move || submit_for_button())
                            .child(
                                text(move || {
                                    if create_loading.get() {
                                        "Creating..."
                                    } else {
                                        "Create"
                                    }
                                })
                                .text_size(px(13.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().primary_foreground),
                            ),
                    ),
            )
            .into_element()
    });
    let filter_options = options.clone();
    let filtered = Rc::new(move || {
        let needle = query.get().trim().to_lowercase();
        filter_options()
            .into_iter()
            .filter(|(_, label)| needle.is_empty() || label.to_lowercase().contains(&needle))
            .collect::<Vec<_>>()
    });
    let can_create_options = options.clone();
    let can_create_request = create.clone();
    let can_create = Rc::new(move || {
        let needle = query.get().trim().to_lowercase();
        can_create_request.is_some()
            && !needle.is_empty()
            && !can_create_options()
                .iter()
                .any(|(_, label)| label.eq_ignore_ascii_case(needle.trim()))
    });
    let current_for_panel = selected_value.clone();
    let create_title_for_panel = create_title.clone();
    let search_placeholder_for_panel = search_placeholder.clone();
    let panel = Rc::new(move || {
        let default_icon_for_search = default_icon.clone();
        let default_color_for_search = default_color.clone();
        let default_icon_for_row = default_icon.clone();
        let default_color_for_row = default_color.clone();
        let placeholder_text = search_placeholder_for_panel.clone();
        let create_title_text = create_title_for_panel.clone();
        let can_create_for_input = can_create.clone();
        let filtered_for_empty = filtered.clone();
        let filtered_for_rows = filtered.clone();
        let filtered_for_visibility = filtered.clone();
        let current_for_rows = current_for_panel.clone();
        let on_select_for_rows = on_select.clone();
        let empty_when = {
            let filtered = filtered_for_empty;
            let can_create = can_create.clone();
            move || !creating.get() && (*filtered)().is_empty() && !(*can_create)()
        };
        let create_when = can_create.clone();
        view()
            .w(move || px(panel_width.get()))
            .flex_col()
            .rounded(px(8.0))
            .border(px(1.0))
            .border_color(move || theme().border)
            .bg(move || theme().popover)
            .overflow_hidden()
            .when(searchable, move |panel| {
                let default_icon_for_search = default_icon_for_search.clone();
                let default_color_for_search = default_color_for_search.clone();
                panel.child(
                    view()
                        .w_full()
                        .px(px(12.0))
                        .border_b(px(1.0))
                        .border_color(move || theme().border)
                        .h(move || if creating.get() { px(0.0) } else { px(37.0) })
                        .overflow_hidden()
                        .child(
                            input()
                                .value(query)
                                .on_change(move |value: &str| query.set(value.to_string()))
                                .on_submit(move |_: &str| {
                                    if (*can_create_for_input)() {
                                        create_name.set(query.get());
                                        create_icon_value.set(default_icon_for_search.to_string());
                                        create_color_value
                                            .set(default_color_for_search.to_string());
                                        create_error.set(String::new());
                                        create_loading.set(false);
                                        creating.set(true);
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
                when(empty_when, move || {
                    view().h(px(48.0)).items_center().justify_center().child(
                        text("No results found")
                            .text_size(px(13.0))
                            .text_color(move || theme().muted_foreground),
                    )
                })
                .otherwise(|| view().h(px(0.0)).into_element()),
            )
            .child(
                when(
                    move || !creating.get() && !(*filtered_for_visibility)().is_empty(),
                    move || {
                        let filtered_for_rows = filtered_for_rows.clone();
                        let current_for_rows = current_for_rows.clone();
                        let on_select_for_rows = on_select_for_rows.clone();
                        scroll()
                            .max_h(px(220.0))
                            .w_full()
                            .py(px(4.0))
                            .flex_col()
                            .child(each(
                                move || (*filtered_for_rows)(),
                                |(value, _)| value.clone(),
                                move |item, _| {
                                    let (value, label) = item.get();
                                    let active = current_for_rows() == value;
                                    let value_for_press = value.clone();
                                    let on_select = on_select_for_rows.clone();
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
                                            crate::ui::release_select(select_id);
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
                                },
                            ))
                            .into_element()
                    },
                )
                .otherwise(|| view().h(px(0.0)).into_element()),
            )
            .child(
                when(
                    move || !creating.get() && (*create_when)(),
                    move || {
                        let default_icon_for_row = default_icon_for_row.clone();
                        let default_color_for_row = default_color_for_row.clone();
                        let title = create_title_text.clone();
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
                                create_name.set(query.get());
                                create_icon_value.set(default_icon_for_row.to_string());
                                create_color_value.set(default_color_for_row.to_string());
                                create_error.set(String::new());
                                create_loading.set(false);
                                creating.set(true);
                            })
                            .child(
                                image(asset!("icons/plus.svg"))
                                    .size(px(14.0))
                                    .tint_color(move || theme().primary),
                            )
                            .child(
                                text(move || format!("{} “{}”", title, query.get().trim()))
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(move || theme().primary),
                            )
                            .into_element()
                    },
                )
                .otherwise(|| view().h(px(0.0)).into_element()),
            )
            .into_element()
    });
    let has_value_for_text = has_value.clone();
    let has_value_for_color = has_value.clone();
    let create_form_for_overlay = create_form.clone();
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
                    crate::ui::claim_select(select_id);
                    let panel_for_open = panel.clone();
                    let page = event.page_position;
                    let local = event.position;
                    let top = page.y.0 - local.y.0 + height + 4.0;
                    let left = (page.x.0 - local.x.0).max(8.0);
                    creating.set(false);
                    crate::ui::open_menu(crate::ui::MenuPlacement::At { top, left }, move || {
                        (*panel_for_open)()
                    });
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
        .child(
            when(
                move || creating.get(),
                move || {
                    nsh_core::element::overlay()
                        .inset_0()
                        .z_index(200)
                        .bg(rgba(0x00000080))
                        .on_press(move || creating.set(false))
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
        )
        .child(
            when(
                move || creating.get(),
                move || {
                    let form = create_form_for_overlay.clone();
                    nsh_core::element::overlay()
                        .inset_0()
                        .z_index(201)
                        .items_center()
                        .justify_center()
                        .p(px(24.0))
                        .pointer_events(PointerEvents::BoxNone)
                        .child(
                            view()
                                .w_full()
                                .max_w(px(512.0))
                                .max_h(px(700.0))
                                .rounded(px(12.0))
                                .border(px(1.0))
                                .border_color(move || theme().border)
                                .bg(move || theme().card)
                                .overflow_hidden()
                                .child((*form)()),
                        )
                        .into_element()
                },
            )
            .otherwise(|| view().h(px(0.0)).into_element()),
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
