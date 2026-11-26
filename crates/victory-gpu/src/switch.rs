//! The `VICTORY_*` on/off switches (`VICTORY_D3D12_DEBUG=1` and its siblings).

use std::ffi::OsStr;

/// Whether a switch's value turns it on: it is set, and is not `0`.
///
/// A switch used to be on whenever its variable existed, so `VICTORY_D3D12_WARP=0` (typed to switch it off, or left
/// over from comparing two runs) still selected the software adapter. Any other value keeps the old meaning.
fn value_is_on(value: Option<&OsStr>) -> bool {
    value.is_some_and(|v| v != "0")
}

/// Whether a switch is on when unset falls back to `default`.
fn value_is_on_or_default(value: Option<&OsStr>, default: bool) -> bool {
    match value {
        Some(v) => v != "0",
        None => default,
    }
}

/// Whether the environment variable `name` switches something on.
pub(crate) fn on(name: &str) -> bool {
    value_is_on(std::env::var_os(name).as_deref())
}

/// Whether the environment variable `name` switches something on, using
/// `default` when the variable is unset.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn on_or_default(name: &str, default: bool) -> bool {
    value_is_on_or_default(std::env::var_os(name).as_deref(), default)
}

#[cfg(test)]
mod tests {
    use super::{value_is_on, value_is_on_or_default};
    use std::ffi::OsStr;

    #[test]
    fn unset_is_off() {
        assert!(!value_is_on(None));
    }

    #[test]
    fn zero_is_off() {
        assert!(!value_is_on(Some(OsStr::new("0"))));
    }

    #[test]
    fn one_is_on() {
        assert!(value_is_on(Some(OsStr::new("1"))));
    }

    #[test]
    fn other_values_keep_their_old_meaning() {
        // Documented as `=1`; anything else that was on before stays on: only `0` is new.
        for value in ["", "true", "yes", "2", "00", " 0", "0 "] {
            assert!(value_is_on(Some(OsStr::new(value))), "{value:?}");
        }
    }

    #[test]
    fn default_enabled_switch_respects_explicit_zero() {
        assert!(value_is_on_or_default(None, true));
        assert!(!value_is_on_or_default(None, false));
        assert!(!value_is_on_or_default(Some(OsStr::new("0")), true));
        assert!(value_is_on_or_default(Some(OsStr::new("1")), false));
    }
}
