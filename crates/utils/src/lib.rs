//! The inevitable catchall "utils" crate. Generally only add
//! things here that only depend on the standard library and
//! "core" crates.
//!
mod command;
pub use command::*;
mod path;
pub use path::*;
mod iterators;
pub use iterators::*;
mod timestamp;
pub use timestamp::*;
mod tracing_util;
pub use tracing_util::*;
/// Re-execute the current process
pub mod reexec;
mod result_ext;
pub use result_ext::*;

/// The name of our binary
pub const NAME: &str = "bootc";

/// Intended for use in `main`, calls an inner function and
/// handles errors by printing them.
pub fn run_main<F>(f: F)
where
    F: FnOnce() -> anyhow::Result<()>,
{
    use std::io::Write as _;

    use owo_colors::OwoColorize;

    if let Err(e) = f() {
        let mut stderr = anstream::stderr();
        // Don't panic if writing fails.
        let _ = writeln!(stderr, "{}{:#}", "error: ".red(), e);
        std::process::exit(1);
    }
}
