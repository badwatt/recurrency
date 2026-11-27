//! UUID-valued select with the same trigger and panel as every other select.

use std::rc::Rc;

use nsh_core::prelude::*;
use uuid::Uuid;

use crate::components::select::SelectProps;

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
    pub create: Option<Rc<dyn Fn(String, String, String, Rc<dyn Fn(Result<(), String>)>)>>,
    pub search_placeholder: String,
    pub create_title: String,
    pub show_label: bool,
    pub searchable: bool,
    pub create_icon: String,
    pub create_color: String,
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
            height: 36.0,
            create: None,
            search_placeholder: "Search...".to_string(),
            create_title: "Create new".to_string(),
            show_label: true,
            searchable: true,
            create_icon: "tag".to_string(),
            create_color: crate::components::color_picker::DEFAULT_COLOR.to_string(),
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn create(
        mut self,
        create: impl Fn(String, String, String, Rc<dyn Fn(Result<(), String>)>) + 'static,
    ) -> Self {
        self.create = Some(Rc::new(create));
        self
    }

    pub fn search_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.search_placeholder = placeholder.into();
        self
    }

    pub fn create_title(mut self, title: impl Into<String>) -> Self {
        self.create_title = title.into();
        self
    }

    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }

    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
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

pub fn select_menu(props: SelectMenuProps) -> impl IntoElement {
    let SelectMenuProps {
        label,
        placeholder,
        selected,
        options,
        height,
        create,
        search_placeholder,
        create_title,
        show_label,
        searchable,
        create_icon,
        create_color,
    } = props;
    let label_options = options.clone();
    let panel_options = options.clone();
    let mut props = SelectProps::new(
        label,
        placeholder,
        move || {
            let Some(current) = selected.try_get() else {
                return String::new();
            };
            label_options()
                .into_iter()
                .find(|option| Some(option.value) == current)
                .map_or_else(String::new, |option| option.label.clone())
        },
        move || {
            selected
                .try_get()
                .flatten()
                .map_or_else(String::new, |id| id.to_string())
        },
        move || selected.try_get().flatten().is_some(),
        move || {
            panel_options()
                .into_iter()
                .map(|option| (option.value.to_string(), option.label.clone()))
                .collect()
        },
        move |value| {
            if let Ok(id) = Uuid::parse_str(&value) {
                let _ = selected.try_set(Some(id));
            }
        },
    )
    .height(height)
    .show_label(show_label)
    .searchable(searchable)
    .create_icon(create_icon)
    .create_color(create_color)
    .search_placeholder(search_placeholder)
    .clearable(move || {
        let _ = selected.try_set(None);
    });
    if let Some(create) = create {
        props = props.create(
            move |name, icon, color, done| create(name, icon, color, done),
            create_title,
        );
    }
    crate::components::select::select(props)
}

#[cfg(test)]
mod tests {
    #[test]
    fn select_menu_uses_the_shared_select() {
        let source = include_str!("select_menu.rs");
        assert!(source.contains("SelectProps::new"));
        assert!(source.contains("clearable"));
    }

    #[test]
    fn select_menu_survives_a_disposed_signal() {
        // The panel may outlive the dialog that made it: reads and writes go
        // through `try_get`/`try_set` so a closed form cannot panic.
        let source = include_str!("select_menu.rs");
        let get = format!("selected.{}()", "get");
        let set = format!("selected.{}(", "set");
        assert!(!source.contains(&get));
        assert!(!source.contains(&set));
        assert!(source.contains("try_get"));
        assert!(source.contains("try_set"));
    }
}
