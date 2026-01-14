//! Exponential backoff logic for daemon restarts.

use std::time::Duration;

/// Calculates the next backoff delay using exponential backoff.
///
/// The formula is: min_delay * 2^attempt, capped at max_delay.
///
/// # Arguments
/// * `attempt` - The restart attempt number (1-indexed)
/// * `min_delay` - Minimum backoff delay
/// * `max_delay` - Maximum backoff delay (cap)
///
/// # Returns
/// The calculated backoff duration.
pub fn calculate_backoff(attempt: u32, min_delay: Duration, max_delay: Duration) -> Duration {
    // Cap the exponent to prevent overflow
    let exponent = attempt.min(20);
    let multiplier = 2u64.saturating_pow(exponent);
    let delay_secs = min_delay.as_secs().saturating_mul(multiplier);
    Duration::from_secs(delay_secs.min(max_delay.as_secs()))
}

/// Determines if a process should be restarted based on runtime.
///
/// If the process ran for longer than the max delay, it's considered stable
/// and the backoff should be reset.
///
/// # Arguments
/// * `runtime` - How long the process ran before crashing
/// * `max_delay` - The maximum backoff delay
///
/// # Returns
/// True if the process should reset its backoff (was stable).
pub fn should_reset_backoff(runtime: Duration, max_delay: Duration) -> bool {
    runtime > max_delay
}

/// Default minimum backoff delay (5 seconds).
pub const DEFAULT_MIN_BACKOFF: Duration = Duration::from_secs(5);

/// Default maximum backoff delay (15 minutes).
pub const DEFAULT_MAX_BACKOFF: Duration = Duration::from_secs(900);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_backoff_progression() {
        let min = Duration::from_secs(5);
        let max = Duration::from_secs(900);

        assert_eq!(calculate_backoff(0, min, max), Duration::from_secs(5)); // 5 * 2^0 = 5
        assert_eq!(calculate_backoff(1, min, max), Duration::from_secs(10)); // 5 * 2^1 = 10
        assert_eq!(calculate_backoff(2, min, max), Duration::from_secs(20)); // 5 * 2^2 = 20
        assert_eq!(calculate_backoff(3, min, max), Duration::from_secs(40)); // 5 * 2^3 = 40
        assert_eq!(calculate_backoff(4, min, max), Duration::from_secs(80)); // 5 * 2^4 = 80
        assert_eq!(calculate_backoff(5, min, max), Duration::from_secs(160)); // 5 * 2^5 = 160
        assert_eq!(calculate_backoff(6, min, max), Duration::from_secs(320)); // 5 * 2^6 = 320
        assert_eq!(calculate_backoff(7, min, max), Duration::from_secs(640)); // 5 * 2^7 = 640
        assert_eq!(calculate_backoff(8, min, max), Duration::from_secs(900)); // capped at max
    }

    #[test]
    fn test_calculate_backoff_capped_at_max() {
        let min = Duration::from_secs(5);
        let max = Duration::from_secs(900);

        // High attempt numbers should all return max
        assert_eq!(calculate_backoff(10, min, max), Duration::from_secs(900));
        assert_eq!(calculate_backoff(20, min, max), Duration::from_secs(900));
        assert_eq!(calculate_backoff(100, min, max), Duration::from_secs(900));
    }

    #[test]
    fn test_calculate_backoff_no_overflow() {
        let min = Duration::from_secs(5);
        let max = Duration::from_secs(900);

        // Should not panic or overflow with very high values
        let result = calculate_backoff(u32::MAX, min, max);
        assert_eq!(result, max);
    }

    #[test]
    fn test_should_reset_backoff() {
        let max = Duration::from_secs(900);

        // Process ran for less than max - don't reset
        assert!(!should_reset_backoff(Duration::from_secs(100), max));
        assert!(!should_reset_backoff(Duration::from_secs(899), max));

        // Process ran for exactly max - don't reset
        assert!(!should_reset_backoff(Duration::from_secs(900), max));

        // Process ran for more than max - reset (was stable)
        assert!(should_reset_backoff(Duration::from_secs(901), max));
        assert!(should_reset_backoff(Duration::from_secs(3600), max)); // 1 hour
    }

    #[test]
    fn test_default_constants() {
        assert_eq!(DEFAULT_MIN_BACKOFF, Duration::from_secs(5));
        assert_eq!(DEFAULT_MAX_BACKOFF, Duration::from_secs(900));
    }
}
