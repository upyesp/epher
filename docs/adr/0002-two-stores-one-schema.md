# Two physical Stores, one schema: Native Store and Web Store

- Status: accepted
- Date: 2026-08-13

Persisted user data lives in one of two Stores that share an identical logical
schema. The Native Store is shared on a single device by the command-line,
terminal, and desktop frontends; the Web Store lives inside the browser/PWA
sandbox and is physically separate from it.

We rejected per-frontend silos (data stranded in each app) and cross-device
sync (impossible offline with no backend). Data is portable between Stores by
explicit import/export, not by automatic roaming.
