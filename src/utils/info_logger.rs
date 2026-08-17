//! Console logging setup.
//!
//! One function, [`initialize_env_logger`], which installs the `env_logger`
//! format the runners' `log::info!`/`log::error!` calls are printed with. For
//! the on-disk record of what the system did, see
//! [`activity_log`](crate::activity_log).

use log::Level;

/// Install the crate's `env_logger` format as the global logger.
///
/// Idempotent: every runner calls it on startup and later calls are no-ops.
/// `RUST_LOG` selects the level (default `info`), and setting `LOG_SHOW_TIME` to
/// `true` prepends a local timestamp to each line.
pub fn initialize_env_logger() {
    let env = env_logger::Env::default().filter_or("RUST_LOG", "info");
    let _ = env_logger::Builder::from_env(env)
        .format(|buf, record| {
            use chrono::Local;
            use std::io::Write;

            let level_style = buf.default_level_style(record.level());

            // Check environment variable to see if time should be included
            let show_time =
                std::env::var("LOG_SHOW_TIME").unwrap_or_else(|_| "false".into()) == "true";

            if show_time {
                if record.level() == Level::Info || record.level() == Level::Warn {
                    writeln!(
                        buf,
                        "[{level_style}{:<4}{level_style:#}] [{}] [{}] {}",
                        record.level(),
                        record.target(),
                        Local::now().format("%Y-%m-%d %H:%M:%S%.6f"),
                        record.args()
                    )
                } else {
                    writeln!(
                        buf,
                        "[{level_style}{:<5}{level_style:#}][{}] [{}] {}",
                        record.level(),
                        record.target(),
                        Local::now().format("%Y-%m-%d %H:%M:%S%.6f"),
                        record.args()
                    )
                }
            } else {
                if record.level() == Level::Info || record.level() == Level::Warn {
                    writeln!(
                        buf,
                        "[{level_style}{:<4}{level_style:#}] [{}] {}",
                        record.level(),
                        record.target(),
                        record.args()
                    )
                } else {
                    writeln!(
                        buf,
                        "[{level_style}{:<5}{level_style:#}][{}] {}",
                        record.level(),
                        record.target(),
                        record.args()
                    )
                }
            }
        })
        .try_init();
}

/// `initialize_env_logger` installs a *global* logger (`env_logger`), which
/// can only ever be set up once per process (`try_init` swallows the "already
/// initialised" error on purpose - every runner in `running/*.rs` calls it
/// unconditionally on startup, so it has to be idempotent). That makes the
/// format closure above hard to unit test in-process: whichever test runs
/// first wins the global logger race, and toggling `LOG_SHOW_TIME` would leak
/// into every other test in the binary since env vars are process-global.
///
/// To actually exercise the closure (both the `show_time` branches and both
/// the `Info`/`Warn` vs. other-level width branches) and check its real
/// output, the test below re-executes this very test in a fresh child
/// process, with `LOG_SHOW_TIME` and a marker env var set only for that
/// child, and inspects the child's captured stderr.
#[cfg(test)]
mod tests {
    use super::*;

    const CHILD_MARKER: &str = "MICRO_SP_INFO_LOGGER_CHILD";

    fn run_child(log_show_time: &str) -> String {
        let exe = std::env::current_exe().expect("test binary path");
        let output = std::process::Command::new(exe)
            .arg("utils::info_logger::tests::logger_output_reflects_level_width_and_log_show_time")
            .arg("--exact")
            .arg("--nocapture")
            .env(CHILD_MARKER, "1")
            .env("LOG_SHOW_TIME", log_show_time)
            .env_remove("RUST_LOG")
            .output()
            .expect("failed to spawn child test process");

        assert!(
            output.status.success(),
            "child test process failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stderr).into_owned()
    }

    #[test]
    fn logger_output_reflects_level_width_and_log_show_time() {
        if std::env::var(CHILD_MARKER).is_ok() {
            // We are the re-executed child: install the real logger and emit
            // one line per level, then let the process exit normally so the
            // parent can inspect what got written to stderr. `LOG_SHOW_TIME`
            // is already set by the parent for this process.
            initialize_env_logger();
            log::info!(target: "info_logger_test", "info-line");
            log::error!(target: "info_logger_test", "error-line");
            return;
        }

        // show_time = false: no timestamp, just level/target/message.
        let without_time = run_child("false");
        assert!(
            without_time.contains("[info_logger_test] info-line"),
            "info line missing/misformatted, got: {without_time}"
        );
        assert!(
            without_time.contains("[info_logger_test] error-line"),
            "error line missing/misformatted, got: {without_time}"
        );
        // `Info`/`Warn` are padded to width 4 (`INFO`), everything else
        // (including `ERROR`) is padded to width 5 - that's the whole reason
        // the closure branches on level in the first place.
        assert!(without_time.contains("INFO"));
        assert!(without_time.contains("ERROR"));

        // show_time = true: a timestamp of the form YYYY-MM-DD is prepended.
        let with_time = run_child("true");
        assert!(
            with_time.contains("info-line") && with_time.contains("error-line"),
            "messages missing, got: {with_time}"
        );
        let year_prefix = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert!(
            with_time.contains(&year_prefix),
            "expected a timestamp like {year_prefix} in output, got: {with_time}"
        );
        assert!(
            with_time.len() > without_time.len(),
            "the show_time=true output should be longer (it has an extra timestamp field)"
        );
    }
}
