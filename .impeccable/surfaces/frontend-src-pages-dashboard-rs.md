---
version: 1
slug: "frontend-src-pages-dashboard-rs"
primary_target: "frontend/src/pages/dashboard.rs"
related_targets: []
---

# Dashboard surface brief

Scope: the authenticated dashboard at `/` in `frontend/src/pages/dashboard.rs`.
Mode: Operate. Existing visual system inherited; no new visual world.
Audience: one person planning the month's transfers into accounts.
Job: answer "how much do I transfer into each account this month, and what is still pending from today?" in one scan.
Success: the fixed month budget and the date-based pending amount are unmistakable, per account and in total.

## Direction contract

THESIS: A monthly transfer ledger, not a spend dashboard. The hero is the fixed budget and the pending amount; the account rows are the plan. It refuses the generic KPI-card grid and the category donut as the first thing seen.

OWN-WORLD: The incumbent Recurrency language: Cabinet Grotesk, JetBrains Mono for figures, restrained card surfaces, thin borders, muted uppercase kickers, and account colors used only as small identity marks. No new palette, no new component family.

STORY: The user opens the app at the start of the month, sees the exact transfer per account, and returns during the month to see what is still pending. Reports owns history and yearly totals.

FIRST VIEWPORT: A full-width hero card with "Plan for <month>" and a very large fixed budget on the left; on the right, "Pending from today" with its own large figure, a short "N payments · D days left" line, and a thin split meter. Directly below, the account ledger fills the rest of the first viewport: color/icon, account name, monthly breakdown, budget, and pending. The primary action is scanning, not clicking; the "Add subscription" action sits quietly in the header.

FORM: Code-led. Ranked first among three examined structures: monthly ledger, split meter, calendar-first. The ledger won because the task is reconciliation across accounts, not timeline browsing. Signature interaction: hovering/focusing an account row lifts its identity mark and links it to its upcoming payments; no decorative animation.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance.
