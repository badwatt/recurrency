//! In-app notification centre opened from the top bar.

use nsh_base::theme;
use nsh_core::prelude::*;

use crate::api::{self, NotificationList};
use crate::auth::{self, AuthSession};

pub fn notifications_modal(auth: AuthSession) -> AnyElement {
    let server = auth::stored_server();
    let token = auth.token.clone();
    let resource_token = token.clone();
    let data = resource(
        || Some(()),
        move |(), _| {
            let server = server.clone();
            let token = resource_token.clone();
            async move { api::load_notifications(server, token).await }
        },
    );
    let list_server = auth::stored_server();
    let list_token = token.clone();
    let mark_all = {
        let server = list_server.clone();
        let token = list_token.clone();
        move || {
            let server = server.clone();
            let token = token.clone();
            spawn(async move {
                let _ = api::mark_all_notifications_read(server, token).await;
                data.refetch();
            });
        }
    };
    let rows = move || {
        data.get()
            .unwrap_or_default()
            .notifications
            .into_iter()
            .map(|notification| {
                let id = notification.id;
                let unread = !notification.is_read;
                let title = notification.title;
                let message = notification.message.unwrap_or_default();
                let date = notification.created_at.chars().take(10).collect::<String>();
                let server = list_server.clone();
                let token = list_token.clone();
                let delete_server = list_server.clone();
                let delete_token = list_token.clone();
                let delete = move || {
                    let server = delete_server.clone();
                    let token = delete_token.clone();
                    spawn(async move {
                        let _ = api::delete_notification(server, token, id).await;
                        data.refetch();
                    });
                };
                view()
                    .w_full()
                    .flex_row()
                    .items_start()
                    .gap(px(8.0))
                    .p(px(8.0))
                    .rounded(px(6.0))
                    .bg(move || if unread { theme().accent } else { theme().card.opacity(0.0) })
                    .on_press(move || {
                        let server = server.clone();
                        let token = token.clone();
                        spawn(async move {
                            let _ = api::mark_notification_read(server, token, id).await;
                            data.refetch();
                        });
                    })
                    .child(
                        view()
                            .h(px(7.0))
                            .w(px(7.0))
                            .mt(px(5.0))
                            .rounded(px(999.0))
                            .bg(move || if unread { theme().primary } else { theme().border }),
                    )
                    .child(
                        view()
                            .flex_col()
                            .flex_grow(1.0)
                            .min_w(px(0.0))
                            .child(text(title).text_size(px(12.0)).font_weight(FontWeight::MEDIUM).text_color(move || theme().foreground))
                            .child(text(message).text_size(px(11.0)).text_color(move || theme().muted_foreground))
                            .child(text(date).text_size(px(9.0)).text_color(move || theme().muted_foreground)),
                    )
                    .child(crate::ui::mini_icon_button(asset!("icons/trash.svg"), true, delete))
                    .into_element()
            })
            .collect::<Vec<_>>()
    };
    let unread = move || data.get().map_or(0, |list: NotificationList| list.unread_count);
    crate::components::dialog::dialog(
        crate::components::dialog::DialogProps::new(
            "Notifications",
            format!("{} unread", unread()),
            move || crate::ui::close_notifications(),
        )
        .width(520.0)
        .body(
            view()
                .p(px(20.0))
                .gap(px(6.0))
                .child(
                    when(
                        move || data.loading() && data.get().is_none(),
                        || text("Loading notifications…").text_size(px(12.0)).text_color(move || theme().muted_foreground).into_element(),
                    )
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(
                    when(
                        move || !data.loading() && data.get().is_some_and(|list| list.notifications.is_empty()),
                        || text("You're all caught up.").text_size(px(12.0)).text_color(move || theme().muted_foreground).into_element(),
                    )
                    .otherwise(|| view().h(px(0.0))),
                )
                .child(view().w_full().flex_col().gap(px(2.0)).children(rows()))
                .into_element(),
        )
        .footer(
            view()
                .w_full()
                .flex_row()
                .justify_end()
                .gap(px(8.0))
                .child(crate::ui::action_button("Mark all read", false, mark_all))
                .child(crate::ui::action_button("Close", false, move || crate::ui::close_notifications()))
                .into_element(),
        ),
    )
    .into_element()
}
