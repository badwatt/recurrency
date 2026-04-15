use dioxus::prelude::*;
use crate::models::category::Category;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CategoriesState {
    pub categories: Vec<Category>,
    pub is_loading: bool,
    pub error: Option<String>,
}

impl CategoriesState {
    /// Get the default category (if any).
    pub fn default_category(&self) -> Option<&Category> {
        self.categories.iter().find(|c| c.is_default)
    }

    /// Get a category by ID.
    pub fn get_by_id(&self, id: &uuid::Uuid) -> Option<&Category> {
        self.categories.iter().find(|c| c.id == *id)
    }

    pub fn set_loading(&mut self) {
        self.is_loading = true;
        self.error = None;
    }

    pub fn set_data(&mut self, cats: Vec<Category>) {
        self.categories = cats;
        self.is_loading = false;
        self.error = None;
    }

    pub fn set_error(&mut self, err: String) {
        self.error = Some(err);
        self.is_loading = false;
    }
}

pub static CATEGORIES: GlobalSignal<CategoriesState> = Signal::global(|| CategoriesState::default());

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    fn make_cat(name: &str, is_default: bool) -> Category {
        Category {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: name.into(),
            icon: None,
            color: None,
            is_system: false,
            is_default,
            created_at: DateTime::UNIX_EPOCH,
            updated_at: DateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn default_category_lookup() {
        let mut state = CategoriesState::default();
        state.categories = vec![
            make_cat("Other", false),
            make_cat("Default", true),
        ];
        assert_eq!(state.default_category().unwrap().name, "Default");
    }

    #[test]
    fn get_by_id() {
        let mut state = CategoriesState::default();
        let cat = make_cat("Test", false);
        let id = cat.id;
        state.categories.push(cat);
        assert_eq!(state.get_by_id(&id).unwrap().name, "Test");
    }

    #[test]
    fn no_default_returns_none() {
        let state = CategoriesState::default();
        assert!(state.default_category().is_none());
    }
}