//! The `epher` argument surface lives in epher-cli (the console crate) so
//! every binary parses identically (ADR-0013); the unified binary's entry
//! point reaches it through this module (ADR-0011). `action_from` maps
//! parsed arguments to an [`Action`] — the frontends then run in thin
//! wrappers over their own entry points.

pub use epher_cli::dispatch::{action_from, Action, Args, Command};
