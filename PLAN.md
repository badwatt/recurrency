# PLAN.md

- This file contains a list of features that need to be implemented in the app.
- Use TDD always.
- You should work on 1 task at a time, and then commit, and then continue with the next task.
- When you are done with a task, you should mark it as done in this file, and if necessary,
update the README.md too.

## TASKS:

- [x] if in settings, date format is set to DD-MM-YYYY, then every date should be formatted as DD-MM-YYYY,
including for example the date visible on every subscription card.
- [ ] Start Date format do not get updated when switching between DD-MM-YYYY or YYYY-MM-DD. (HTML date inputs use browser locale - requires custom picker)
- [x] why frontend/assets/icon.png exists? if is not being used then remove it.
- [x] I want to change the email placeholder on the login page to "email@example.com"
- [x] if SMTP is not configured, then email notifications should not be able to be enabled.
- [x] I want to use lucide icons for the whole app. (already using inline SVG lucide-style icons)
- [x] I want to always and in anywhere (labels, dropdowns, etc...) show the chosen logo of a category (or default logo if no logo is set) next to de category name. Order: logo -> category name. (subscription cards + dropdowns done)
- [x] I want to restore the functionality from the tauri branch that allows user to set a custom logo for the subscriptions/transactions using duckduckgo.
- [x] when user enables push notifications from the mobile app, then a notification should be sent to the user to let them know that push notifications are enabled.
- [x] when user enables system notifications from the desktop app, then a notification should be sent to the user to let them know that system notifications are enabled.



