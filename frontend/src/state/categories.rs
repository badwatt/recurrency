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
    use chrono::DateTime;
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

    #[test]
    fn get_by_id_not_found() {
        let state = CategoriesState::default();
        let random_id = Uuid::new_v4();
        assert!(state.get_by_id(&random_id).is_none());
    }

    #[test]
    fn set_loading_state() {
        let mut state = CategoriesState::default();
        state.set_error("Some error".into());
        state.set_loading();
        assert!(state.is_loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn set_data_clears_loading() {
        let mut state = CategoriesState::default();
        state.set_loading();
        assert!(state.is_loading);
        let cats = vec![make_cat("Test", false)];
        state.set_data(cats);
        assert!(!state.is_loading);
        assert!(state.error.is_none());
        assert_eq!(state.categories.len(), 1);
    }

    #[test]
    fn set_error_state() {
        let mut state = CategoriesState::default();
        state.set_loading();
        state.set_error("Network error".into());
        assert!(!state.is_loading);
        assert_eq!(state.error, Some("Network error".into()));
    }

    #[test]
    fn set_data_replaces_categories() {
        let mut state = CategoriesState::default();
        state.categories = vec![make_cat("Old", false)];
        state.set_data(vec![make_cat("New", true)]);
        assert_eq!(state.categories.len(), 1);
        assert_eq!(state.categories[0].name, "New");
    }

    #[test]
    fn default_category_none_when_multiple_non_default() {
        let mut state = CategoriesState::default();
        state.categories = vec![make_cat("A", false), make_cat("B", false)];
        assert!(state.default_category().is_none());
    }

    // ── Additional CategoriesState tests ──
    #[test]
    fn categories_state_clone() {
        let state = CategoriesState::default();
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn categories_state_debug() {
        let state = CategoriesState::default();
        let debug = format!("{:?}", state);
        assert!(debug.contains("CategoriesState"));
    }

    #[test]
    fn categories_state_default_values() {
        let state = CategoriesState::default();
        assert!(state.categories.is_empty());
        assert!(!state.is_loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn set_error_then_loading_clears_it() {
        let mut state = CategoriesState::default();
        state.set_error("fail".into());
        state.set_loading();
        assert!(state.is_loading);
        assert!(state.error.is_none());
    }

    #[test]
    fn get_by_id_multiple_categories() {
        let mut state = CategoriesState::default();
        let cat_a = make_cat("A", false);
        let cat_b = make_cat("B", true);
        let id_a = cat_a.id;
        let id_b = cat_b.id;
        state.categories = vec![cat_a, cat_b];
        assert_eq!(state.get_by_id(&id_a).unwrap().name, "A");
        assert_eq!(state.get_by_id(&id_b).unwrap().name, "B");
    }
}