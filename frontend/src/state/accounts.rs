use crate::models::account::Account;
use dioxus::prelude::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AccountsState {
    pub accounts: Vec<Account>,
    pub is_loading: bool,
    pub error: Option<String>,
}

impl AccountsState {
    pub fn default_account(&self) -> Option<&Account> {
        self.accounts.iter().find(|a| a.is_default)
    }

    pub fn get_by_id(&self, id: &uuid::Uuid) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == *id)
    }

    pub fn name_for(&self, id: &uuid::Uuid) -> String {
        self.get_by_id(id)
            .map(|a| a.name.clone())
            .unwrap_or_else(|| "No account".to_string())
    }

    pub fn set_loading(&mut self) {
        self.is_loading = true;
        self.error = None;
    }

    pub fn set_data(&mut self, accounts: Vec<Account>) {
        self.accounts = accounts;
        self.is_loading = false;
        self.error = None;
    }

    pub fn set_error(&mut self, err: String) {
        self.error = Some(err);
        self.is_loading = false;
    }
}

pub static ACCOUNTS: GlobalSignal<AccountsState> = Signal::global(AccountsState::default);

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    fn make_account(name: &str, is_default: bool) -> Account {
        Account {
            id: uuid::Uuid::new_v4(),
            user_id: uuid::Uuid::new_v4(),
            name: name.into(),
            icon: None,
            color: None,
            is_default,
            created_at: DateTime::UNIX_EPOCH,
            updated_at: DateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn default_account_lookup() {
        let mut state = AccountsState::default();
        state.accounts = vec![make_account("A", false), make_account("B", true)];
        assert_eq!(state.default_account().unwrap().name, "B");
    }

    #[test]
    fn get_by_id_returns_none_for_unknown() {
        let state = AccountsState::default();
        assert!(state.get_by_id(&uuid::Uuid::new_v4()).is_none());
    }

    #[test]
    fn name_for_falls_back_to_sin_cuenta() {
        let state = AccountsState::default();
        assert_eq!(state.name_for(&uuid::Uuid::new_v4()), "No account");
    }

    #[test]
    fn set_data_clears_loading_and_error() {
        let mut state = AccountsState::default();
        state.set_error("fail".into());
        state.set_loading();
        assert!(state.error.is_none());
        state.set_data(vec![make_account("A", true)]);
        assert!(!state.is_loading);
        assert_eq!(state.accounts.len(), 1);
    }
}
