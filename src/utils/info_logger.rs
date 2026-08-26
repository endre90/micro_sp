//! Console logging setup.
//!
//! One function, [`initialize_env_logger`], which installs the `env_logger`
//! format the runners' `log::info!`/`log::error!` calls are printed with - and,
//! wrapped around it, a logger that mirrors those same lines into the on-disk
//! [`activity_log`] so they can be grepped next to the
//! `OP`/`TRANS`/`SOP`/`VAR` records rather than only scrolling past in a
//! terminal.

use log::{Level, Log, Metadata, Record};

use crate::utils::activity_log;

/// Install the crate's `env_logger` format as the global logger.
///
/// Idempotent: every runner calls it on startup and later calls are no-ops.
/// `RUST_LOG` selects the level (default `info`), and setting `LOG_SHOW_TIME` to
/// `true` prepends a local timestamp to each line.
///
/// Whatever reaches the console also reaches the
/// [`activity_log`] when one is installed, via the private
/// `ActivityTee` wrapper below. Nothing has to be initialised in a particular
/// order: the activity log's own emission path is a no-op until it is up, so
/// the runners' existing `initialize_env_logger()` then
/// `activity_log::init_from_env()` pair works either way round.
pub fn initialize_env_logger() {
    let env = env_logger::Env::default().filter_or("RUST_LOG", "info");
    // `build` rather than `try_init`, because the console logger is no longer
    // the global logger - it is the inner half of `ActivityTee`, which is.
    let console = env_logger::Builder::from_env(env)
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
        .build();

    // `try_init` sets the global max level from the filter as part of
    // installing; `build` does not, so do it here - and only when this call is
    // the one that won, or a later no-op call would clobber the winner's filter.
    let max_level = console.filter();
    if log::set_boxed_logger(Box::new(ActivityTee { console })).is_ok() {
        log::set_max_level(max_level);
    }
}

/// The global logger: the `env_logger` above, plus a copy of every line it
/// prints sent to the [`activity_log`].
///
/// It has to be a wrapper rather than something inside the format closure,
/// because a format closure runs only for records the filter already accepted
/// and has no way to reach anything but its output buffer.
struct ActivityTee {
    console: env_logger::Logger,
}

impl Log for ActivityTee {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        self.console.enabled(metadata)
    }

    fn log(&self, record: &Record<'_>) {
        self.console.log(record);

        // `log::set_max_level` only enforces the *global* maximum, so a
        // per-module directive (`RUST_LOG=warn,micro_sp::running=info`) is
        // applied inside `env_logger`'s own `log`, which silently ignores what
        // it rejects. Asking the same question here is what keeps the file and
        // the console showing the same set of lines.
        if !self.console.matches(record) {
            return;
        }
        // Cheap `OnceLock` read, before the two allocations below - a library
        // consumer who never enables the activity log should not pay for it on
        // every log line.
        if !activity_log::is_enabled() {
            return;
        }

        activity_log::log_message(
            record.level(),
            record.target(),
            &location_of(record),
            record.args().to_string(),
        );
    }

    fn flush(&self) {
        self.console.flush();
    }
}

/// `file:line` for the log statement, for the activity log's `subject` column.
///
/// The leading `src/` is dropped because it is on every single line and the
/// column is only so wide. `-` is the placeholder the log already uses for
/// "no subject" when the location is unavailable, which it is for records
/// forwarded from another logger or built by hand.
fn location_of(record: &Record<'_>) -> String {
    match (record.file(), record.line()) {
        (Some(file), Some(line)) => format!("{}:{line}", file.strip_prefix("src/").unwrap_or(file)),
        (Some(file), None) => file.strip_prefix("src/").unwrap_or(file).to_string(),
        _ => "-".to_string(),
    }
}

/// `initialize_env_logger` installs a *global* logger, which can only ever be
/// set up once per process (the `Err` from `set_boxed_logger` is discarded on
/// purpose - every runner in `running/*.rs` calls it unconditionally on
/// startup, so it has to be idempotent). That makes the
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

    use std::path::PathBuf;

    const CHILD_MARKER: &str = "MICRO_SP_INFO_LOGGER_CHILD";
    const BRIDGE_MARKER: &str = "MICRO_SP_INFO_LOGGER_BRIDGE_CHILD";

    /// Spawn the named test in a fresh process, with the activity log
    /// explicitly off so an inherited `MICRO_SP_ACTIVITY_LOG*` from whoever ran
    /// `cargo test` cannot start writing files, and return its stderr.
    fn spawn_child(test: &str, marker: &str, envs: &[(&str, &str)]) -> std::process::Output {
        let exe = std::env::current_exe().expect("test binary path");
        let mut command = std::process::Command::new(exe);
        command
            .arg(test)
            .arg("--exact")
            .arg("--nocapture")
            .env(marker, "1")
            .env_remove("RUST_LOG")
            .env_remove("LOG_SHOW_TIME")
            .env_remove("MICRO_SP_ACTIVITY_LOG")
            .env_remove("MICRO_SP_ACTIVITY_LOG_DIR");
        for (key, value) in envs {
            command.env(key, value);
        }
        let output = command
            .output()
            .expect("failed to spawn child test process");
        assert!(
            output.status.success(),
            "child test process failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }

    fn run_child(log_show_time: &str) -> String {
        let output = spawn_child(
            "utils::info_logger::tests::logger_output_reflects_level_width_and_log_show_time",
            CHILD_MARKER,
            &[("LOG_SHOW_TIME", log_show_time)],
        );
        String::from_utf8_lossy(&output.stderr).into_owned()
    }

    /// A per-run directory under the OS temp dir, keyed like the activity log's
    /// own test helper so a parallel run cannot collide.
    fn temp_dir(tag: &str) -> PathBuf {
        let unique = format!(
            "micro_sp_bridge_{tag}_{:?}_{}",
            std::thread::current().id(),
            std::process::id()
        )
        .replace(['(', ')', ' '], "");
        let dir = std::env::temp_dir().join(unique);
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    /// Run the mirroring child against a fresh directory and return the
    /// activity log it wrote.
    fn run_bridge_child(tag: &str, rust_log: Option<&str>) -> String {
        let dir = temp_dir(tag);
        let dir_str = dir.to_str().expect("utf-8 temp dir").to_string();
        let mut envs = vec![("MICRO_SP_ACTIVITY_LOG_DIR", dir_str.as_str())];
        if let Some(filter) = rust_log {
            envs.push(("RUST_LOG", filter));
        }
        spawn_child(
            "utils::info_logger::tests::console_lines_are_mirrored_into_the_activity_log",
            BRIDGE_MARKER,
            &envs,
        );

        let file = dir.join("micro_sp.log");
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|e| panic!("no activity log at {}: {e}", file.display()));
        let _ = std::fs::remove_dir_all(&dir);
        text
    }

    /// The lines the child emits, and the level tag each one must be filed
    /// under in the log.
    fn emit_one_line_per_level() {
        log::info!(target: "bridge_test", "info-mirrored");
        log::warn!(target: "bridge_test", "warn-mirrored");
        log::error!(target: "bridge_test", "error-mirrored");
        log::debug!(target: "bridge_test", "debug-mirrored");
        // The activity log reports its own write failures through `log::error!`.
        // If those came back round through the bridge, one failed write would
        // queue a record whose write fails, and so on without end - so the
        // log's own target is the one thing that must never reach the file.
        log::error!(target: activity_log::LOG_TARGET, "recursion-guard-marker");
    }

    /// Everything the console prints has to reach the activity log too, tagged
    /// with its level so it can be grepped beside the `OP`/`VAR` lines, and
    /// carrying the `file:line` of the statement that emitted it.
    ///
    /// Another child process, for the same reason as the test below plus one
    /// more: `activity_log`'s handle is a `OnceLock` as well, so a test that
    /// installs one would decide for every other test in the binary where the
    /// file goes.
    #[test]
    fn console_lines_are_mirrored_into_the_activity_log() {
        if std::env::var(BRIDGE_MARKER).is_ok() {
            initialize_env_logger();
            activity_log::init_from_env();
            emit_one_line_per_level();
            assert!(activity_log::flush(), "the child never installed a log");
            return;
        }

        // Default filter (`info`): the three levels the crate leans on are all
        // in the file, each in the kind column and each with its location.
        let log = run_bridge_child("levels", None);
        for (tag, message) in [
            ("INFO ", "info-mirrored"),
            ("WARN ", "warn-mirrored"),
            ("ERR  ", "error-mirrored"),
        ] {
            let line = log
                .lines()
                .find(|line| line.contains(message))
                .unwrap_or_else(|| panic!("{message} missing from the log:\n{log}"));
            assert!(
                line.contains(&format!("| {tag} |")),
                "{message} is not tagged {}: {line}",
                tag.trim()
            );
            assert!(
                line.contains("| bridge_test"),
                "the log target belongs in the source column: {line}"
            );
            assert!(
                line.contains("utils/info_logger.rs:"),
                "the emitting file:line belongs in the subject column: {line}"
            );
        }

        // `debug!` is below the default filter, so it is absent from the file
        // for exactly the reason it is absent from the console.
        assert!(
            !log.contains("debug-mirrored"),
            "a line the console suppressed reached the file:\n{log}"
        );
        assert!(
            !log.contains("recursion-guard-marker"),
            "the activity log recorded one of its own lines:\n{log}"
        );

        // `RUST_LOG=warn` has to move the file's contents exactly as it moves
        // the console's, or the two stop being two views of one thing.
        let quiet = run_bridge_child("quiet", Some("warn"));
        assert!(
            !quiet.contains("info-mirrored"),
            "RUST_LOG=warn still let an info line into the file:\n{quiet}"
        );
        assert!(
            quiet.contains("warn-mirrored") && quiet.contains("error-mirrored"),
            "RUST_LOG=warn dropped lines it should have kept:\n{quiet}"
        );

        // `RUST_LOG=trace` reaches all the way down.
        let loud = run_bridge_child("loud", Some("trace"));
        let debug_line = loud
            .lines()
            .find(|line| line.contains("debug-mirrored"))
            .unwrap_or_else(|| panic!("RUST_LOG=trace kept debug off the file:\n{loud}"));
        assert!(debug_line.contains("| DEBUG |"), "{debug_line}");
    }

    /// The file has to stay self-describing about the kinds it now contains,
    /// since a rotated log is often read on its own.
    #[test]
    fn the_header_documents_the_mirrored_levels() {
        let log = run_bridge_child("header", None);
        let header: String = log
            .lines()
            .take_while(|l| l.starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            header.contains("ERR/WARN/INFO/DEBUG/TRACE"),
            "the banner does not mention the mirrored levels:\n{header}"
        );
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
