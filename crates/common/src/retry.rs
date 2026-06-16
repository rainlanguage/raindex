use crate::{erc20::Error as ERC20Error, local_db::LocalDbError};
use std::future::Future;

#[cfg(not(target_family = "wasm"))]
use backon::{ConstantBuilder, ExponentialBuilder, Retryable};
#[cfg(target_family = "wasm")]
use gloo_timers::future::TimeoutFuture;
#[cfg(not(target_family = "wasm"))]
use std::time::Duration;

pub const DEFAULT_BASE_DELAY_MILLIS: u64 = 500;

#[derive(Debug)]
pub enum RetryError<E> {
    InvalidMaxAttempts,
    Operation(E),
}

#[inline]
fn ensure_max_attempts<E>(max_attempts: usize) -> Result<(), RetryError<E>> {
    if max_attempts == 0 {
        Err(RetryError::InvalidMaxAttempts)
    } else {
        Ok(())
    }
}

#[cfg(any(target_family = "wasm", test))]
#[inline]
fn compute_sleep_and_next_delay_ms(
    current_delay_ms: u64,
    rate_limit_delay_ms: u64,
    is_rate_limited: bool,
) -> (u32, u64) {
    let sleep_ms = if is_rate_limited && rate_limit_delay_ms > 0 {
        rate_limit_delay_ms
    } else {
        current_delay_ms
    };
    let delay = sleep_ms.min(u64::from(u32::MAX)) as u32;
    let next_delay_ms = current_delay_ms.saturating_mul(2);
    (delay, next_delay_ms)
}

impl From<RetryError<LocalDbError>> for LocalDbError {
    fn from(err: RetryError<LocalDbError>) -> Self {
        match err {
            RetryError::InvalidMaxAttempts => LocalDbError::InvalidRetryMaxAttemps,
            RetryError::Operation(inner) => inner,
        }
    }
}

impl From<RetryError<ERC20Error>> for ERC20Error {
    fn from(err: RetryError<ERC20Error>) -> Self {
        match err {
            RetryError::InvalidMaxAttempts => ERC20Error::InvalidRetryMaxAttemps,
            RetryError::Operation(inner) => inner,
        }
    }
}

#[cfg(not(target_family = "wasm"))]
pub async fn retry_with_backoff<T, F, Fut, E, ShouldRetry, IsRateLimited>(
    operation: F,
    max_attempts: usize,
    base_delay_ms: u64,
    rate_limit_delay_ms: u64,
    should_retry: ShouldRetry,
    is_rate_limited: IsRateLimited,
) -> Result<T, RetryError<E>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    ShouldRetry: Fn(&E) -> bool,
    IsRateLimited: Fn(&E) -> bool,
{
    ensure_max_attempts::<E>(max_attempts)?;

    let delay = if base_delay_ms > 0 {
        base_delay_ms
    } else {
        DEFAULT_BASE_DELAY_MILLIS
    };

    let backoff = ExponentialBuilder::default()
        .with_min_delay(Duration::from_millis(delay))
        .with_max_times(max_attempts.saturating_sub(1));

    let retryable = || async { operation().await.map_err(RetryError::Operation) };

    retryable
        .retry(backoff)
        .when(|e: &RetryError<E>| matches!(e, RetryError::Operation(err) if should_retry(err)))
        .adjust(|e: &RetryError<E>, dur| {
            dur?;
            if let RetryError::Operation(err) = e {
                if rate_limit_delay_ms > 0 && is_rate_limited(err) {
                    return Some(Duration::from_millis(rate_limit_delay_ms));
                }
            }
            dur
        })
        .await
}

#[cfg(target_family = "wasm")]
pub async fn retry_with_backoff<T, F, Fut, E, ShouldRetry, IsRateLimited>(
    operation: F,
    max_attempts: usize,
    base_delay_ms: u64,
    rate_limit_delay_ms: u64,
    should_retry: ShouldRetry,
    is_rate_limited: IsRateLimited,
) -> Result<T, RetryError<E>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    ShouldRetry: Fn(&E) -> bool,
    IsRateLimited: Fn(&E) -> bool,
{
    ensure_max_attempts::<E>(max_attempts)?;

    let mut delay_ms = if base_delay_ms > 0 {
        base_delay_ms
    } else {
        DEFAULT_BASE_DELAY_MILLIS
    };

    for attempt in 0..max_attempts {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                if attempt + 1 >= max_attempts || !should_retry(&err) {
                    return Err(RetryError::Operation(err));
                }

                let is_rl = is_rate_limited(&err);
                let (delay, next_delay_ms) =
                    compute_sleep_and_next_delay_ms(delay_ms, rate_limit_delay_ms, is_rl);
                TimeoutFuture::new(delay).await;
                delay_ms = next_delay_ms;
            }
        }
    }

    Err(RetryError::InvalidMaxAttempts)
}

#[cfg(not(target_family = "wasm"))]
pub async fn retry_with_constant_interval<T, F, Fut, E, ShouldRetry>(
    operation: F,
    max_attempts: usize,
    interval_ms: u64,
    should_retry: ShouldRetry,
) -> Result<T, RetryError<E>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    ShouldRetry: Fn(&E) -> bool,
{
    ensure_max_attempts::<E>(max_attempts)?;

    let backoff = ConstantBuilder::default()
        .with_delay(Duration::from_millis(interval_ms))
        .with_max_times(max_attempts.saturating_sub(1));

    let retryable = || async { operation().await.map_err(RetryError::Operation) };

    retryable
        .retry(backoff)
        .when(|error: &RetryError<E>| matches!(error, RetryError::Operation(err) if should_retry(err)))
        .await
}

#[cfg(target_family = "wasm")]
pub async fn retry_with_constant_interval<T, F, Fut, E, ShouldRetry>(
    operation: F,
    max_attempts: usize,
    interval_ms: u64,
    should_retry: ShouldRetry,
) -> Result<T, RetryError<E>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    ShouldRetry: Fn(&E) -> bool,
{
    ensure_max_attempts::<E>(max_attempts)?;

    for attempt in 0..max_attempts {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                if attempt + 1 >= max_attempts || !should_retry(&err) {
                    return Err(RetryError::Operation(err));
                }

                let delay = interval_ms.min(u64::from(u32::MAX)) as u32;
                TimeoutFuture::new(delay).await;
            }
        }
    }

    Err(RetryError::InvalidMaxAttempts)
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug)]
    enum TestError {
        Rpc,
        Json,
        RateLimit,
    }

    #[tokio::test]
    async fn retries_and_succeeds_after_transient_error() {
        let attempts = AtomicUsize::new(0);
        let result = retry_with_backoff(
            || async {
                let current = attempts.fetch_add(1, Ordering::SeqCst);
                if current == 0 {
                    Err(TestError::Rpc)
                } else {
                    Ok(42u32)
                }
            },
            3,
            0,
            0,
            |err| matches!(err, TestError::Rpc),
            |_| false,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn stops_after_max_attempts() {
        let attempts = AtomicUsize::new(0);
        let err = retry_with_backoff(
            || async {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(TestError::Rpc)
            },
            2,
            0,
            0,
            |err| matches!(err, TestError::Rpc),
            |_| false,
        )
        .await
        .unwrap_err();

        match err {
            RetryError::Operation(TestError::Rpc) => {}
            other => panic!("expected Rpc error, got {other:?}"),
        }
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn does_not_retry_non_retryable_error() {
        let attempts = AtomicUsize::new(0);
        let err = retry_with_backoff(
            || async {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(TestError::Json)
            },
            3,
            0,
            0,
            |err| matches!(err, TestError::Rpc),
            |_| false,
        )
        .await
        .unwrap_err();

        match err {
            RetryError::Operation(TestError::Json) => {}
            other => panic!("expected Json error, got {other:?}"),
        }
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn zero_attempts_is_config_error() {
        let attempts = AtomicUsize::new(0);
        let err = retry_with_backoff(
            || async {
                attempts.fetch_add(1, Ordering::SeqCst);
                Ok::<u32, TestError>(1)
            },
            0,
            0,
            0,
            |_err| true,
            |_| false,
        )
        .await
        .unwrap_err();

        match err {
            RetryError::InvalidMaxAttempts => {}
            other => panic!("expected config error, got {other:?}"),
        }
        assert_eq!(attempts.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn ensure_max_attempts_rejects_zero() {
        let err = super::ensure_max_attempts::<TestError>(0).unwrap_err();
        assert!(matches!(err, RetryError::InvalidMaxAttempts));
    }

    #[test]
    fn ensure_max_attempts_allows_positive_values() {
        assert!(super::ensure_max_attempts::<TestError>(1).is_ok());
    }

    #[test]
    fn compute_sleep_and_next_delay_uses_rate_limit_delay_and_doubles_backoff() {
        let (sleep, next_delay_ms) = super::compute_sleep_and_next_delay_ms(200, 100, true);
        assert_eq!(sleep, 100);
        assert_eq!(next_delay_ms, 400);
    }

    #[tokio::test]
    async fn rate_limited_error_uses_fixed_delay() {
        let attempts = AtomicUsize::new(0);

        let result = retry_with_backoff(
            || async {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(TestError::RateLimit)
            },
            3,
            1,
            1,
            |err| matches!(err, TestError::RateLimit),
            |err| matches!(err, TestError::RateLimit),
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            RetryError::Operation(TestError::RateLimit) => {}
            other => panic!("expected RateLimit error, got {other:?}"),
        }
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn rate_limited_error_with_mixed_errors() {
        let attempts = AtomicUsize::new(0);

        let result = retry_with_backoff(
            || async {
                let current = attempts.fetch_add(1, Ordering::SeqCst);
                match current {
                    0 => Err(TestError::RateLimit),
                    1 => Err(TestError::Rpc),
                    _ => Ok(42u32),
                }
            },
            5,
            1,
            1,
            |err| matches!(err, TestError::Rpc | TestError::RateLimit),
            |err| matches!(err, TestError::RateLimit),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn constant_interval_retries_and_succeeds() {
        let attempts = AtomicUsize::new(0);
        let result = retry_with_constant_interval(
            || async {
                let current = attempts.fetch_add(1, Ordering::SeqCst);
                if current == 0 {
                    Err(TestError::Rpc)
                } else {
                    Ok(42u32)
                }
            },
            3,
            10,
            |err| matches!(err, TestError::Rpc),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn constant_interval_stops_after_max_attempts() {
        let attempts = AtomicUsize::new(0);
        let err = retry_with_constant_interval(
            || async {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(TestError::Rpc)
            },
            2,
            10,
            |err| matches!(err, TestError::Rpc),
        )
        .await
        .unwrap_err();

        match err {
            RetryError::Operation(TestError::Rpc) => {}
            other => panic!("expected Rpc error, got {other:?}"),
        }
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn constant_interval_does_not_retry_non_retryable() {
        let attempts = AtomicUsize::new(0);
        let err = retry_with_constant_interval(
            || async {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(TestError::Json)
            },
            3,
            10,
            |err| matches!(err, TestError::Rpc),
        )
        .await
        .unwrap_err();

        match err {
            RetryError::Operation(TestError::Json) => {}
            other => panic!("expected Json error, got {other:?}"),
        }
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn constant_interval_zero_attempts_is_error() {
        let attempts = AtomicUsize::new(0);
        let err = retry_with_constant_interval(
            || async {
                attempts.fetch_add(1, Ordering::SeqCst);
                Ok::<u32, TestError>(1)
            },
            0,
            10,
            |_err| true,
        )
        .await
        .unwrap_err();

        match err {
            RetryError::InvalidMaxAttempts => {}
            other => panic!("expected config error, got {other:?}"),
        }
        assert_eq!(attempts.load(Ordering::SeqCst), 0);
    }
}

#[cfg(all(test, target_family = "wasm", feature = "browser-tests"))]
mod wasm_tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[derive(Debug)]
    enum TestError {
        Rpc,
        Json,
    }

    #[wasm_bindgen_test]
    async fn retries_and_succeeds_after_transient_error() {
        let attempts = Rc::new(Cell::new(0));
        let operation_attempts = attempts.clone();

        let result = retry_with_backoff(
            move || {
                let attempts = operation_attempts.clone();
                async move {
                    let current = attempts.get();
                    attempts.set(current + 1);
                    if current == 0 {
                        Err(TestError::Rpc)
                    } else {
                        Ok(42u32)
                    }
                }
            },
            3,
            0,
            0,
            |err| matches!(err, TestError::Rpc),
            |_| false,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.get(), 2);
    }

    #[wasm_bindgen_test]
    async fn stops_after_max_attempts() {
        let attempts = Rc::new(Cell::new(0));
        let operation_attempts = attempts.clone();

        let err = retry_with_backoff(
            move || {
                let attempts = operation_attempts.clone();
                async move {
                    let current = attempts.get();
                    attempts.set(current + 1);
                    Err::<(), _>(TestError::Rpc)
                }
            },
            2,
            0,
            0,
            |err| matches!(err, TestError::Rpc),
            |_| false,
        )
        .await
        .unwrap_err();

        match err {
            RetryError::Operation(TestError::Rpc) => {}
            other => panic!("expected Rpc error, got {other:?}"),
        }
        assert_eq!(attempts.get(), 2);
    }

    #[wasm_bindgen_test]
    async fn does_not_retry_non_retryable_error() {
        let attempts = Rc::new(Cell::new(0));
        let operation_attempts = attempts.clone();

        let err = retry_with_backoff(
            move || {
                let attempts = operation_attempts.clone();
                async move {
                    let current = attempts.get();
                    attempts.set(current + 1);
                    Err::<(), _>(TestError::Json)
                }
            },
            3,
            0,
            0,
            |err| matches!(err, TestError::Rpc),
            |_| false,
        )
        .await
        .unwrap_err();

        match err {
            RetryError::Operation(TestError::Json) => {}
            other => panic!("expected Json error, got {other:?}"),
        }
        assert_eq!(attempts.get(), 1);
    }

    #[wasm_bindgen_test]
    fn ensure_max_attempts_behavior() {
        let err = super::ensure_max_attempts::<TestError>(0).unwrap_err();
        assert!(matches!(err, RetryError::InvalidMaxAttempts));
        assert!(super::ensure_max_attempts::<TestError>(1).is_ok());
    }

    #[wasm_bindgen_test]
    async fn constant_interval_retries_and_succeeds() {
        let attempts = Rc::new(Cell::new(0));
        let operation_attempts = attempts.clone();

        let result = retry_with_constant_interval(
            move || {
                let attempts = operation_attempts.clone();
                async move {
                    let current = attempts.get();
                    attempts.set(current + 1);
                    if current == 0 {
                        Err(TestError::Rpc)
                    } else {
                        Ok(42u32)
                    }
                }
            },
            3,
            10,
            |err| matches!(err, TestError::Rpc),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.get(), 2);
    }

    #[wasm_bindgen_test]
    async fn constant_interval_stops_after_max_attempts() {
        let attempts = Rc::new(Cell::new(0));
        let operation_attempts = attempts.clone();

        let err = retry_with_constant_interval(
            move || {
                let attempts = operation_attempts.clone();
                async move {
                    let current = attempts.get();
                    attempts.set(current + 1);
                    Err::<(), _>(TestError::Rpc)
                }
            },
            2,
            10,
            |err| matches!(err, TestError::Rpc),
        )
        .await
        .unwrap_err();

        match err {
            RetryError::Operation(TestError::Rpc) => {}
            other => panic!("expected Rpc error, got {other:?}"),
        }
        assert_eq!(attempts.get(), 2);
    }
}
