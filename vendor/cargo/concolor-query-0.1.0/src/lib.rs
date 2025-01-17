#[cfg(feature = "windows")]
pub mod windows;

/// Check [CLICOLOR] status
///
/// ANSI colors are supported and should be used when the program isn't piped.
///
/// See also:
/// - [terminfo](https://crates.io/crates/terminfo) or [term](https://crates.io/crates/term) for
///   checking termcaps
/// - [termbg](https://crates.io/crates/termbg) for detecting background color
///
/// [CLICOLOR]: https://bixense.com/clicolors/
pub fn clicolor() -> bool {
    let value = std::env::var_os("CLICOLOR");
    value
        .as_deref()
        .unwrap_or_else(|| std::ffi::OsStr::new("1"))
        != "0"
}

/// Check [CLICOLOR_FORCE] status
///
/// ANSI colors should be enabled no matter what.
///
/// [CLICOLOR_FORCE]: https://bixense.com/clicolors/
pub fn clicolor_force() -> bool {
    let value = std::env::var_os("CLICOLOR_FORCE");
    value
        .as_deref()
        .unwrap_or_else(|| std::ffi::OsStr::new("0"))
        != "0"
}

/// Check [NO_COLOR] status
///
/// User-level configuration files and per-instance command-line arguments should override
/// [NO_COLOR]. A user should be able to export `$NO_COLOR` in their shell configuration file as a
/// default, but configure a specific program in its configuration file to specifically enable
/// color.
///
/// [NO_COLOR]: https://no-color.org/
pub fn no_color() -> bool {
    std::env::var_os("NO_COLOR").is_some()
}

/// Check `TERM` for color support
#[cfg(not(windows))]
pub fn term_supports_color() -> bool {
    match std::env::var_os("TERM") {
        // If TERM isn't set, then we are in a weird environment that
        // probably doesn't support colors.
        None => return false,
        Some(k) => {
            if k == "dumb" {
                return false;
            }
        }
    }
    true
}

/// Check `TERM` for color support
#[cfg(windows)]
pub fn term_supports_color() -> bool {
    // On Windows, if TERM isn't set, then we shouldn't automatically
    // assume that colors aren't allowed. This is unlike Unix environments
    // where TERM is more rigorously set.
    if let Some(k) = std::env::var_os("TERM") {
        if k == "dumb" {
            return false;
        }
    }
    true
}

/// Check `TERM` for ANSI color support
#[cfg(not(windows))]
pub fn term_supports_ansi_color() -> bool {
    term_supports_color()
}

/// Check `TERM` for ANSI color support
#[cfg(windows)]
pub fn term_supports_ansi_color() -> bool {
    match std::env::var_os("TERM") {
        // If TERM isn't set, then we are in a weird environment that
        // probably doesn't support ansi.
        None => return false,
        Some(k) => {
            // cygwin doesn't seem to support ANSI escape sequences
            // and instead has its own variety. However, the Windows
            // console API may be available.
            if k == "dumb" || k == "cygwin" {
                return false;
            }
        }
    }
    true
}

/// Check [COLORTERM] for truecolor support
///
/// [COLORTERM]: https://github.com/termstandard/colors
pub fn truecolor() -> bool {
    let value = std::env::var_os("COLORTERM");
    let value = value.as_deref().unwrap_or_default();
    value == "truecolor" || value == "24bit"
}
