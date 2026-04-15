use dioxus::prelude::*;
use crate::models::notification::Notification;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NotificationsState {
    pub notifications: Vec<Notification>,
    pub unread_count: i64,
    pub is_loading: bool,
    pub error: Option<String>,
    pub is_open: bool,
}

impl NotificationsState {
    pub fn set_loading(&mut self) {
        self.is_loading = true;
        self.error = None;
    }

    pub fn set_data(&mut self, notifications: Vec<Notification>, unread_count: i64) {
        self.notifications = notifications;
        self.unread_count = unread_count;
        self.is_loading = false;
        self.error = None;
    }

    pub fn mark_as_read(&mut self, id: &uuid::Uuid) {
        for n in &mut self.notifications {
            if n.id == *id {
                n.is_read = true;
            }
        }
        self.unread_count = self.notifications.iter().filter(|n| !n.is_read).count() as i64;
    }

    pub fn mark_all_as_read(&mut self) {
        for n in &mut self.notifications {
            n.is_read = true;
        }
        self.unread_count = 0;
    }

    pub fn delete(&mut self, id: &uuid::Uuid) {
        self.notifications.retain(|n| n.id != *id);
        self.unread_count = self.notifications.iter().filter(|n| !n.is_read).count() as i64;
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }
}

pub static NOTIFICATIONS: GlobalSignal<NotificationsState> = Signal::global(|| NotificationsState::default());

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    fn make_notification(read: bool) -> Notification {
        Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            notification_type: "upcoming_payment".into(),
            title: "Test".into(),
            message: None,
            is_read: read,
            created_at: DateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn default_empty() {
        let state = NotificationsState::default();
        assert_eq!(state.unread_count, 0);
        assert!(!state.is_open);
    }

    #[test]
    fn mark_as_read_updates_count() {
        let mut state = NotificationsState::default();
        let n1 = make_notification(false);
        let n2 = make_notification(false);
        let id1 = n1.id;
        state.notifications = vec![n1, n2];
        state.unread_count = 2;

        state.mark_as_read(&id1);
        assert_eq!(state.unread_count, 1);
    }

    #[test]
    fn mark_all_as_read() {
        let mut state = NotificationsState::default();
        state.notifications = vec![make_notification(false), make_notification(false)];
        state.unread_count = 2;
        state.mark_all_as_read();
        assert_eq!(state.unread_count, 0);
    }

    #[test]
    fn delete_notification() {
        let mut state = NotificationsState::default();
        let n = make_notification(false);
        let id = n.id;
        state.notifications = vec![n];
        state.unread_count = 1;
        state.delete(&id);
        assert!(state.notifications.is_empty());
        assert_eq!(state.unread_count, 0);
    }

    #[test]
    fn toggle_dropdown() {
        let mut state = NotificationsState::default();
        assert!(!state.is_open);
        state.toggle();
        assert!(state.is_open);
        state.toggle();
        assert!(!state.is_open);
    }
}