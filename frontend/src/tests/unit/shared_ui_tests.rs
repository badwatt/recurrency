//! Render smoke tests for the shared UI primitives. These execute the
//! component bodies, so every primitive is covered as a real Dioxus node.

use crate::components::badge::{Badge, BadgeVariant};
use crate::components::button::{Button, ButtonSize, ButtonVariant, IconButton};
use crate::components::card::{Card, CardRadius, CardContent, CardHeader, CardTitle, SectionCard};
use crate::components::checkbox::Checkbox;
use crate::components::empty_state::EmptyState;
use crate::components::field::Field;
use crate::components::input::{Input, InputSize};
use crate::components::list_toolbar::ListToolbar;
use crate::components::page_header::PageHeader;
use crate::components::popover::Popover;
use crate::components::search_input::SearchInput;
use crate::components::segmented::{SegmentOption, SegmentVariant, SegmentedControl};
use crate::components::select_menu::{SelectMenu, SelectOption};
use crate::components::textarea::Textarea;
use crate::components::toggle::Toggle;
use dioxus::prelude::*;

fn render(app: fn() -> Element) {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
}

#[test]
fn every_button_variant_and_size_renders() {
    render(|| {
        rsx! {
            for variant in [
                ButtonVariant::Primary,
                ButtonVariant::Secondary,
                ButtonVariant::Outline,
                ButtonVariant::Ghost,
                ButtonVariant::Destructive,
                ButtonVariant::Link,
                ButtonVariant::Unstyled,
            ] {
                for size in [
                    ButtonSize::Xs,
                    ButtonSize::Sm,
                    ButtonSize::Md,
                    ButtonSize::Lg,
                    ButtonSize::IconSm,
                    ButtonSize::Icon,
                    ButtonSize::IconLg,
                    ButtonSize::Unstyled,
                ] {
                    Button {
                        variant,
                        size,
                        class: "extra".to_string(),
                        title: Some("Title".to_string()),
                        aria_label: Some("Action".to_string()),
                        role: Some("button".to_string()),
                        aria_selected: Some("false".to_string()),
                        aria_pressed: Some("false".to_string()),
                        aria_expanded: Some("false".to_string()),
                        aria_haspopup: Some("menu".to_string()),
                        style: Some("color: red".to_string()),
                        onclick: move |_| {},
                        onmousedown: move |_| {},
                        onmouseenter: move |_| {},
                        onmouseleave: move |_| {},
                        "Action"
                    }
                }
            }
            Button { disabled: true, full_width: true, "Disabled" }
            IconButton {
                label: "Icon action".to_string(),
                aria_pressed: Some("true".to_string()),
                onclick: move |_| {},
                onmousedown: move |_| {},
                onmouseenter: move |_| {},
                onmouseleave: move |_| {},
                "i"
            }
        }
    });
}

#[test]
fn every_badge_variant_renders() {
    render(|| {
        rsx! {
            for variant in [
                BadgeVariant::Primary,
                BadgeVariant::Secondary,
                BadgeVariant::Destructive,
                BadgeVariant::Outline,
                BadgeVariant::Success,
                BadgeVariant::Warning,
                BadgeVariant::Muted,
            ] {
                Badge { variant, class: "extra".to_string(), "Status" }
            }
        }
    });
}

#[test]
fn card_composition_and_header_slot_render() {
    render(|| {
        rsx! {
            Card {
                CardHeader { class: "extra".to_string(),
                    CardTitle { class: "title".to_string(), "Title" }
                }
                CardContent { class: "extra".to_string(), "Body" }
            }
            Card { radius: CardRadius::Lg, class: "p-4".to_string(), "Large radius" }
            SectionCard {
                title: "Section".to_string(),
                description: "Description".to_string(),
                actions: rsx! { Button { "Action" } },
                title_class: "text-primary".to_string(),
                header_class: "border-primary".to_string(),
                content_class: "p-0".to_string(),
                "Content"
            }
            SectionCard { title: "Bare".to_string(), "Content" }
        }
    });
}

#[test]
fn form_primitives_render_all_states() {
    render(|| {
        rsx! {
            for size in [InputSize::Xs, InputSize::Sm, InputSize::Md, InputSize::Lg] {
                Input {
                    size,
                    r#type: "email".to_string(),
                    value: "value".to_string(),
                    placeholder: "Placeholder".to_string(),
                    class: "extra".to_string(),
                    disabled: true,
                    required: true,
                    readonly: true,
                    autofocus: true,
                    autocomplete: "off".to_string(),
                    autocapitalize: "off".to_string(),
                    autocorrect: "off".to_string(),
                    spellcheck: "false".to_string(),
                    inputmode: "email".to_string(),
                    id: "input".to_string(),
                    name: "input".to_string(),
                    aria_label: "Input".to_string(),
                    min: "0".to_string(),
                    max: "10".to_string(),
                    step: "1".to_string(),
                    minlength: "2".to_string(),
                    maxlength: "20".to_string(),
                    on_input: move |_| {},
                    on_blur: move |_| {},
                    on_keydown: move |_| {},
                }
            }
            Input { value: "empty".to_string() }
            Textarea {
                value: "text".to_string(),
                placeholder: "Notes".to_string(),
                class: "extra".to_string(),
                rows: Some(5),
                disabled: true,
                required: true,
                on_input: move |_| {},
                on_keydown: move |_| {},
            }
            Field {
                label: "Name".to_string(),
                required: true,
                error: "Required".to_string(),
                hint: "Hint".to_string(),
                class: "extra".to_string(),
                html_for: "name".to_string(),
                label_class: "extra".to_string(),
                Input { value: "x".to_string() }
            }
            Field { label: "Hint".to_string(), hint: "Only a hint".to_string(), "control" }
            Field { label: "Bare".to_string(), "control" }
            Checkbox { checked: true, on_change: move |_| {}, id: "check".to_string(), label: "Check".to_string(), disabled: true, class: "extra".to_string() }
            Checkbox { checked: false, on_change: move |_| {} }
            Toggle { checked: true, on_change: move |_| {}, label: "Toggle".to_string(), disabled: true, class: "extra".to_string() }
            Toggle { checked: false, on_change: move |_| {}, label: "Toggle".to_string() }
        }
    });
}

#[test]
fn navigation_and_feedback_primitives_render() {
    render(|| {
        rsx! {
            PageHeader {
                title: "Page".to_string(),
                subtitle: "Subtitle".to_string(),
                actions: rsx! { Button { "Action" } },
                class: "extra".to_string(),
            }
            PageHeader { title: "Bare".to_string(), subtitle: "Subtitle".to_string() }
            EmptyState {
                title: "Empty".to_string(),
                description: "Nothing".to_string(),
                icon: rsx! { span { "icon" } },
                action: rsx! { Button { "Create" } },
                class: "extra".to_string(),
            }
            EmptyState { title: "Empty".to_string(), description: "Nothing".to_string() }
            SearchInput {
                value: "query".to_string(),
                on_input: move |_| {},
                placeholder: "Search".to_string(),
                aria_label: "Search".to_string(),
                class: "extra".to_string(),
            }
            SearchInput { value: String::new(), on_input: move |_| {} }
            ListToolbar {
                value: "query".to_string(),
                oninput: move |_| {},
                view: "grid".to_string(),
                onview: move |_| {},
                show_view: true,
                actions: rsx! { span { "actions" } },
            }
            ListToolbar {
                value: String::new(),
                oninput: move |_| {},
                view: "list".to_string(),
                onview: move |_| {},
                show_view: false,
                actions: rsx! { span { "actions" } },
            }
            SegmentedControl {
                value: "a".to_string(),
                options: vec![SegmentOption::new("a", "A"), SegmentOption::new("b", "B")],
                on_change: move |_| {},
                aria_label: "Tabs".to_string(),
                class: "extra".to_string(),
            }
            SegmentedControl {
                value: "b".to_string(),
                options: vec![SegmentOption::new("a", "A"), SegmentOption::new("b", "B")],
                on_change: move |_| {},
                variant: SegmentVariant::Underline,
            }
        }
    });
}

#[test]
fn select_menu_and_popover_render_closed_and_open() {
    render(|| {
        rsx! {
            SelectMenu {
                value: "a".to_string(),
                options: vec![SelectOption::new("a", "A"), SelectOption::new("b", "B")],
                on_change: move |_| {},
                searchable: Some(true),
                placeholder: Some("Choose".to_string()),
                disabled: Some(false),
                class: Some("extra".to_string()),
                id: Some("select".to_string()),
            }
            SelectMenu {
                value: String::new(),
                options: vec![],
                on_change: move |_| {},
            }
            Popover {
                open: true,
                on_close: move |_| {},
                class: "p-3".to_string(),
                panel_class: "extra".to_string(),
                div { "Panel" }
            }
            Popover { open: false, on_close: move |_| {}, "Hidden" }
        }
    });
}
