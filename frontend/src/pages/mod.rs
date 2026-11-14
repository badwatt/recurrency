pub mod accounts_page;
pub mod calendar_page;
pub mod categories_page;
pub mod dashboard;
pub mod debts_page;
pub mod reports_page;
pub mod settings_page;
pub mod subscriptions_page;
pub mod transactions_page;

pub use accounts_page::AccountsPage;
pub use calendar_page::CalendarPage;
pub use categories_page::CategoriesPage;
pub use dashboard::Dashboard;
pub use debts_page::DebtsPage;
pub use reports_page::ReportsPage;
pub use settings_page::{
    SettingsAboutPage, SettingsDataPage, SettingsMoneyPage, SettingsNotificationsPage,
    SettingsPreferencesPage, SettingsProfilePage, SettingsServerPage,
};
pub use transactions_page::TransactionsPage;
