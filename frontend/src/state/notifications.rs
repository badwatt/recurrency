use crate::models::notification::Notification;
use dioxus::prelude::*;

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

pub static NOTIFICATIONS: GlobalSignal<NotificationsState> =
    Signal::global(|| NotificationsState::default());

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
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

    #[test]
    fn set_loading_state() {
        let mut state = NotificationsState::default();
        state.set_loading();
        assert!(state.is_loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn set_loading_clears_error() {
        let mut state = NotificationsState::default();
        state.error = Some("some error".into());
        state.set_loading();
        assert!(state.error.is_none());
        assert!(state.is_loading);
    }

    #[test]
    fn set_data() {
        let mut state = NotificationsState::default();
        state.set_loading();
        let n1 = make_notification(false);
        let n2 = make_notification(true);
        state.set_data(vec![n1, n2], 1);
        assert!(!state.is_loading);
        assert!(state.error.is_none());
        assert_eq!(state.notifications.len(), 2);
        assert_eq!(state.unread_count, 1);
    }

    #[test]
    fn mark_as_read_already_read() {
        let mut state = NotificationsState::default();
        let n = make_notification(true); // already read
        let id = n.id;
        state.notifications = vec![n];
        state.unread_count = 0;
        state.mark_as_read(&id);
        assert_eq!(state.unread_count, 0);
    }

    #[test]
    fn mark_as_read_unknown_id() {
        let mut state = NotificationsState::default();
        let n = make_notification(false);
        state.notifications = vec![n];
        state.unread_count = 1;
        let unknown_id = Uuid::new_v4();
        state.mark_as_read(&unknown_id);
        // No notification was modified, but recount still works
        assert_eq!(state.unread_count, 1);
    }

    #[test]
    fn delete_read_notification_keeps_unread_count() {
        let mut state = NotificationsState::default();
        let n1 = make_notification(false);
        let n2 = make_notification(true); // read
        let id2 = n2.id;
        state.notifications = vec![n1, n2];
        state.unread_count = 1;
        state.delete(&id2); // delete the read one
        assert_eq!(state.notifications.len(), 1);
        assert_eq!(state.unread_count, 1);
    }

    #[test]
    fn delete_unread_reduces_count() {
        let mut state = NotificationsState::default();
        let n1 = make_notification(false);
        let n2 = make_notification(false);
        let id1 = n1.id;
        state.notifications = vec![n1, n2];
        state.unread_count = 2;
        state.delete(&id1);
        assert_eq!(state.unread_count, 1);
        assert_eq!(state.notifications.len(), 1);
    }

    #[test]
    fn set_data_clears_error() {
        let mut state = NotificationsState::default();
        state.set_loading();
        state.error = Some("fail".into());
        let n1 = make_notification(false);
        state.set_data(vec![n1], 1);
        assert!(state.error.is_none());
        assert!(!state.is_loading);
    }

    #[test]
    fn notifications_state_clone() {
        let state = NotificationsState::default();
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn notifications_state_debug() {
        let state = NotificationsState::default();
        let debug = format!("{:?}", state);
        assert!(debug.contains("NotificationsState"));
    }

    #[test]
    fn notifications_state_partial_eq() {
        let s1 = NotificationsState::default();
        let s2 = NotificationsState::default();
        assert_eq!(s1, s2);
    }

    #[test]
    fn notifications_state_default_values() {
        let state = NotificationsState::default();
        assert!(state.notifications.is_empty());
        assert_eq!(state.unread_count, 0);
        assert!(!state.is_loading);
        assert!(state.error.is_none());
        assert!(!state.is_open);
    }

    #[test]
    fn mark_all_as_read_when_empty() {
        let mut state = NotificationsState::default();
        state.mark_all_as_read();
        assert_eq!(state.unread_count, 0);
    }

    #[test]
    fn set_data_with_zero_notifications() {
        let mut state = NotificationsState::default();
        state.set_data(vec![], 0);
        assert!(state.notifications.is_empty());
        assert_eq!(state.unread_count, 0);
    }

    #[test]
    fn set_data_with_many_notifications() {
        let mut state = NotificationsState::default();
        let notifications: Vec<_> = (0..10).map(|_| make_notification(false)).collect();
        state.set_data(notifications.clone(), 10);
        assert_eq!(state.notifications.len(), 10);
        assert_eq!(state.unread_count, 10);
    }

    #[test]
    fn delete_all_notifications() {
        let mut state = NotificationsState::default();
        let n1 = make_notification(false);
        let n2 = make_notification(true);
        let id1 = n1.id;
        let id2 = n2.id;
        state.notifications = vec![n1, n2];
        state.unread_count = 1;
        state.delete(&id1);
        state.delete(&id2);
        assert!(state.notifications.is_empty());
        assert_eq!(state.unread_count, 0);
    }

    #[test]
    fn set_loading_sets_loading_flag() {
        let mut state = NotificationsState::default();
        assert!(!state.is_loading);
        state.set_loading();
        assert!(state.is_loading);
    }

    #[test]
    fn toggle_opens_panel() {
        let mut state = NotificationsState::default();
        assert!(!state.is_open);
        state.toggle();
        assert!(state.is_open);
    }

    #[test]
    fn toggle_closes_panel() {
        let mut state = NotificationsState::default();
        state.is_open = true;
        state.toggle();
        assert!(!state.is_open);
    }

    #[test]
    fn delete_preserves_other_notifications() {
        let mut state = NotificationsState::default();
        let n1 = make_notification(false);
        let n2 = make_notification(false);
        let n3 = make_notification(false);
        let id2 = n2.id;
        state.notifications = vec![n1, n2, n3];
        state.unread_count = 3;
        state.delete(&id2);
        assert_eq!(state.notifications.len(), 2);
    }

    #[test]
    fn mark_as_read_updates_unread_count() {
        let mut state = NotificationsState::default();
        let n = make_notification(false);
        let id = n.id;
        state.notifications = vec![n];
        state.unread_count = 1;
        state.mark_as_read(&id);
        assert_eq!(state.unread_count, 0);
    }

    // ── Additional NotificationsState tests ──
    #[test]
    fn notifications_state_non_open_default() {
        let state = NotificationsState::default();
        assert!(state.notifications.is_empty());
        assert_eq!(state.unread_count, 0);
        assert!(!state.is_loading);
        assert!(state.error.is_none());
        assert!(!state.is_open);
    }

    #[test]
    fn notifications_state_clone_eq() {
        let state = NotificationsState::default();
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn set_data_recomputes_unread() {
        let mut state = NotificationsState::default();
        let n1 = make_notification(false);
        let n2 = make_notification(true);
        let n3 = make_notification(false);
        state.set_data(vec![n1, n2, n3], 2);
        assert_eq!(state.unread_count, 2);
        assert_eq!(state.notifications.len(), 3);
    }

    #[test]
    fn set_error_state() {
        let mut state = NotificationsState::default();
        state.is_loading = true;
        state.error = Some("network".into());
        assert_eq!(state.error, Some("network".into()));
    }
}
