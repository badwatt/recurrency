//! Label + control + error/hint, the single form-field wrapper.

use nsh_base::theme;
use nsh_core::prelude::*;

pub struct FieldProps {
    pub label: String,
    pub required: bool,
    pub error: String,
    pub hint: String,
}

impl FieldProps {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            required: false,
            error: String::new(),
            hint: String::new(),
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.error = error.into();
        self
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = hint.into();
        self
    }
}

pub fn field(props: FieldProps, control: AnyElement) -> impl IntoElement {
    let FieldProps {
        label,
        required,
        error,
        hint,
    } = props;
    let has_error = !error.is_empty();
    let has_hint = has_error == false && !hint.is_empty();
    view()
        .w_full()
        .flex_col()
        .gap(px(6.0))
        .min_w(px(0.0))
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(4.0))
                .child(
                    text(label)
                        .text_size(px(13.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || theme().foreground),
                )
                .when(required, |view| {
                    view.child(
                        text("*")
                            .text_size(px(13.0))
                            .text_color(move || theme().destructive),
                    )
                }),
        )
        .child(control)
        .when(has_error, |view| {
            view.child(
                text(error.clone())
                    .text_size(px(11.0))
                    .text_color(move || theme().destructive),
            )
        })
        .when(has_hint, |view| {
            view.child(
                text(hint.clone())
                    .text_size(px(11.0))
                    .text_color(move || theme().muted_foreground),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_fields_do_not_stretch_vertically() {
        let source = include_str!("field.rs");
        assert!(source.contains("w_full()"));
        let stretch = format!("flex_{}", "grow(1.0)");
        assert!(!source.contains(&stretch));
    }

    #[test]
    fn field_defaults_to_optional_without_messages() {
        let props = FieldProps::new("Name");
        assert!(!props.required);
        assert!(props.error.is_empty());
        assert!(props.hint.is_empty());
    }
}
