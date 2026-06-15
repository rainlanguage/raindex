pub mod add_order;
pub mod allowance;
pub mod csv;
pub mod deposit;
pub mod dotrain_add_order_lsp;
pub mod dotrain_order;
pub mod erc20;
pub mod fuzz;
pub mod local_db;
pub mod meta;
pub mod oracle;
pub mod parsed_meta;
pub mod raindex_client;
pub mod raindex_order_builder;
pub mod rainlang;
pub mod registry;
pub mod remove_order;
#[cfg(not(target_family = "wasm"))]
pub mod replays;
pub mod retry;
pub mod rpc_client;
pub mod subgraph;
pub mod take_orders;
pub mod transaction;
pub mod types;
#[cfg(all(not(target_family = "wasm"), test))]
pub mod unit_tests;
pub mod utils;
pub mod withdraw;
#[cfg(not(target_family = "wasm"))]
pub mod write_tx;
pub use dotrain;
pub use dotrain_lsp;
#[cfg(test)]
pub mod test_helpers;

// `GH_COMMIT_SHA` embeds the build's git commit for test-side traceability. The
// `env!` read is gated behind `#[cfg(test)]` so only test builds require
// `COMMIT_SHA` to resolve at compile time; production builds neither read the env
// var nor carry the constant.
#[cfg(test)]
pub const GH_COMMIT_SHA: &str = env!("COMMIT_SHA", "$COMMIT_SHA not set.");

#[cfg(test)]
mod commit_sha_tests {
    use super::GH_COMMIT_SHA;

    // `GH_COMMIT_SHA` must carry the same compile-time `COMMIT_SHA` value that
    // `build.rs` resolves (CI's `github.sha`, else `git rev-parse HEAD`, else
    // "unknown"). It must never be the unset-sentinel and never empty.
    #[test]
    fn gh_commit_sha_matches_commit_sha_env() {
        assert_eq!(GH_COMMIT_SHA, env!("COMMIT_SHA"));
        assert_ne!(GH_COMMIT_SHA, "$COMMIT_SHA not set.");
        assert!(!GH_COMMIT_SHA.is_empty());
    }
}
