# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

A single person managing their own recurring subscriptions and one-time payments. Their monthly ritual is to move money into the accounts that pay those commitments, then track what is still due before the month ends.

## Product Purpose

Answer one question first: **how much do I transfer into each account this month, and how much is still pending from today?** Recurrency exists so that question takes seconds, not a spreadsheet reconciliation.

## Positioning

Accounts are first-class budgeting buckets, not decorative categories. Every subscription and payment belongs to an account, so the monthly transfer plan is derived from the same data the user already maintains. Categories remain an optional, secondary classification for future budget analysis.

## Operating Context

- The month starts with a fixed transfer plan per account.
- During the month, the user checks what is still pending based on billing dates and dated manual payments.
- Subscriptions can be weekly, monthly, quarterly, or yearly; the fixed plan normalizes them to a monthly equivalent.
- Manual payments are one-time transactions assigned to an account.
- The app runs on web and Android, online against the backend or offline with local storage.
- Amounts are displayed in the user's chosen currency; there is no conversion.

## Capabilities and Constraints

- Subscriptions, transactions, calendar, reports, notifications, and categories already exist.
- Accounts are new and mandatory: users create, rename, recolor, and delete them like categories.
- Existing categories are migrated into accounts without losing subscription or transaction links.
- Categories stay available as an optional classification; they are no longer the budgeting unit.
- The dashboard budget is fixed from the start of the month. Pending is date-based: charges and manual payments dated today or later.
- The dashboard must work in both server and offline modes.
- The reports surface keeps historical analysis; monthly and yearly totals belong there, not in the dashboard hero.

## Brand Commitments

The product name is Recurrency. The existing visual language stays: Cabinet Grotesk, JetBrains Mono for figures, restrained card surfaces, the existing light/dark/Mocha themes, and the current icon and color language.

## Evidence on Hand

Real user data lives in PostgreSQL and in local storage. No invented customers, benchmarks, or financial claims may be added.

## Product Principles

1. The monthly transfer plan is the dashboard's thesis; everything else is secondary.
2. Separate the fixed monthly budget from the date-based pending amount; never blur the two.
3. Accounts are mandatory and user-owned; categories are optional and secondary.
4. Preserve existing data through migration rather than asking the user to re-enter it.
5. Reports explain the past; the dashboard tells the user what to do now.

## Accessibility & Inclusion

Keep the existing keyboard focus behavior, semantic controls, and readable contrast across themes. Do not encode account identity by color alone; pair color with the account name and icon.
