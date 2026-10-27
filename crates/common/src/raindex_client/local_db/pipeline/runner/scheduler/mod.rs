#[cfg(not(target_family = "wasm"))]
pub mod native;
#[cfg(target_family = "wasm")]
pub mod wasm;

#[cfg(not(target_family = "wasm"))]
pub use native::*;
#[cfg(target_family = "wasm")]
pub use wasm::*;

/// Longest wait between sync cycles while a network keeps failing.
pub(crate) const MAX_FAILURE_BACKOFF_MS: u64 = 5 * 60 * 1000;

/// Delay before the next sync cycle.
///
/// After a success this is the configured interval. Each consecutive failure
/// doubles it, up to [`MAX_FAILURE_BACKOFF_MS`] (or the interval itself if
/// that is longer). A cycle that fails before persisting keeps its old
/// watermark, so retrying at the normal pace re-reads the same blocks every
/// few seconds; backing off bounds that RPC load while the cause is fixed.
pub(crate) fn next_cycle_delay_ms(interval_ms: u64, consecutive_failures: u32) -> u64 {
    if consecutive_failures == 0 {
        return interval_ms;
    }
    let ceiling = MAX_FAILURE_BACKOFF_MS.max(interval_ms);
    let factor = 1u64
        .checked_shl(consecutive_failures.min(32))
        .unwrap_or(u64::MAX);
    interval_ms.saturating_mul(factor).min(ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_uses_the_configured_interval() {
        assert_eq!(next_cycle_delay_ms(5_000, 0), 5_000);
    }

    #[test]
    fn failures_double_the_delay() {
        assert_eq!(next_cycle_delay_ms(5_000, 1), 10_000);
        assert_eq!(next_cycle_delay_ms(5_000, 2), 20_000);
        assert_eq!(next_cycle_delay_ms(5_000, 3), 40_000);
    }

    #[test]
    fn failures_stop_at_the_ceiling() {
        assert_eq!(next_cycle_delay_ms(5_000, 6), MAX_FAILURE_BACKOFF_MS);
        assert_eq!(next_cycle_delay_ms(5_000, 64), MAX_FAILURE_BACKOFF_MS);
        assert_eq!(next_cycle_delay_ms(5_000, u32::MAX), MAX_FAILURE_BACKOFF_MS);
    }

    #[test]
    fn interval_longer_than_ceiling_is_kept() {
        assert_eq!(next_cycle_delay_ms(600_000, 3), 600_000);
    }

    #[test]
    fn zero_interval_stays_zero() {
        assert_eq!(next_cycle_delay_ms(0, 5), 0);
    }
}
