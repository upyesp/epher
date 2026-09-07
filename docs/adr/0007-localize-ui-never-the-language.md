# Localize the UI; never localize the scripting language

- Status: accepted
- Date: 2026-08-13

The user interface (labels, menus, errors, number/date display) is localized per
locale. The scripting language's keywords and grammar are fixed to English and
never localized.

Localizing keywords would make Scripts and Functions non-portable across locales
— a function saved in French would not run on an English install — which would
break the shared-schema Store (ADR-0002). Keeping the language fixed, like every
portable programming language, preserves that portability. Display formatting
(decimal separators and the like) follows locale; the grammar does not.
