//! Global configuration for observation-tools
//!
//! By default, observations are **disabled**. To enable them, set the `OBSERVE`
//! environment variable to `1` or `true`, or call [`enable()`] programmatically.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

/// Global enabled state. Defaults to false (observations disabled).
static ENABLED: AtomicBool = AtomicBool::new(false);

/// Track whether we've checked the environment variable
static ENV_CHECKED: OnceLock<()> = OnceLock::new();

/// Initialize from environment variables.
///
/// Called automatically on first check, but can be called explicitly.
/// Checks the `OBSERVE` environment variable - if set to `1` or `true`,
/// observations will be enabled.
pub fn init_from_env() {
  ENV_CHECKED.get_or_init(|| {
    if std::env::var("OBSERVE")
      .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
      .unwrap_or(false)
    {
      ENABLED.store(true, Ordering::Release);
    }
  });
}

/// Check if observations are globally enabled.
///
/// This is a fast, lock-free check that should be called early in the
/// observation path to avoid unnecessary work.
///
/// By default, observations are disabled. To enable them:
/// - Set the `OBSERVE=1` environment variable, or
/// - Call [`enable()`] programmatically
#[inline]
pub fn is_enabled() -> bool {
  init_from_env();
  ENABLED.load(Ordering::Acquire)
}

/// Check if observations are globally disabled.
#[inline]
pub fn is_disabled() -> bool {
  !is_enabled()
}

/// Enable observations globally.
///
/// This takes effect immediately for all subsequent observations.
/// Observations already in-flight will still be sent.
pub fn enable() {
  init_from_env();
  ENABLED.store(true, Ordering::Release);
}

/// Disable all observations globally.
///
/// This reverses a previous [`enable()`] call or `OBSERVE=1` environment
/// variable setting. Takes effect immediately for all subsequent observations.
pub fn disable() {
  init_from_env();
  ENABLED.store(false, Ordering::Release);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_default_disabled() {
    // Reset state for test (note: this is imperfect due to OnceLock)
    ENABLED.store(false, Ordering::Release);
    assert!(is_disabled());
    assert!(!is_enabled());
  }

  #[test]
  fn test_enable_disable() {
    enable();
    assert!(is_enabled());
    assert!(!is_disabled());

    disable();
    assert!(is_disabled());
    assert!(!is_enabled());
  }
}
