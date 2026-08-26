//! An append-only, self-rotating record of what the system actually did.
//!
//! This is the *file* log, and it is a different thing from
//! [`crate::initialize_env_logger`]: that one formats human prose onto stderr
//! for whoever is watching a terminal right now. This one writes one line per
//! discrete thing that happened - an operation changing state, an auto
//! transition firing, a SOP starting or finishing, a variable taking a new
//! value - so that afterwards you can answer "what was this cell doing at
//! 10:32?" from a file rather than from a scrollback buffer that is long gone.
//!
//! The two are not fully separate any more: the logger `initialize_env_logger`
//! installs also forwards every line it prints here through [`log_message`],
//! tagged `ERR`/`WARN`/`INFO`/`DEBUG`/`TRACE` in the kind column. So the prose
//! saying *why* something happened sits in the same file, in timestamp order,
//! as the structured lines saying *what* happened, and one `grep` sees both.
//!
//! # Why a thread and not a tokio task
//!
//! Every emission site is on a runner's hot path: `process_operation` runs once
//! per active operation per tick, and the state diff runs once per runner per
//! tick. None of them may block on a disk write. So emission is a non-blocking
//! `try_send` into a bounded channel, and a single dedicated OS thread does the
//! formatting and the file I/O.
//!
//! It is a `std::thread` plus a `std::sync::mpsc` rather than a tokio task and
//! a `tokio::sync::mpsc` on purpose:
//!
//!   - blocking file I/O never touches a tokio worker, so a slow disk cannot
//!     stall the runners the way it would from inside an async task;
//!   - `try_send` is a plain synchronous call, so it works unchanged from
//!     `process_transition` (which is sync) and from `process_operation` (which
//!     is async) without either of them growing an `.await`;
//!   - the module has no runtime dependency at all, so it can be tested from an
//!     ordinary `#[test]`.
//!
//! # Backpressure
//!
//! The channel is bounded. If the writer ever falls behind, `try_send` fails and
//! the event is **dropped and counted** rather than blocking the control loop -
//! losing a log line is always preferable to delaying a robot. The dropped count
//! is written into the file as soon as the writer catches up, so a gap is never
//! silent.
//!
//! # Layout on disk
//!
//! The active file is always `{dir}/{stem}.log`, so `tail -f` has a stable
//! target. When it would exceed [`ActivityLogConfig::max_bytes`] (5 MiB by
//! default) it is renamed to `{stem}-{YYYYMMDD}-{HHMMSS}.log` and a fresh
//! active file is opened. The oldest rotated files are deleted once there are
//! more than [`ActivityLogConfig::max_files`] of them.

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};

use chrono::{DateTime, Local};
use log::Level;

use crate::{SPValue, State};

/// Variable-name suffixes that are excluded from `VAR` lines by default.
///
/// `process_operation` adds the caller's tick duration to `_elapsed_executing_ms`
/// / `_elapsed_disabled_ms` for every active operation on every tick, so these
/// two change several times a second per operation and carry no information a
/// reader wants - they would be the overwhelming majority of the file and would
/// push everything interesting out of the retained window. Timing is already
/// recoverable from the timestamps on the `OP` lines.
///
/// Override with `MICRO_SP_ACTIVITY_LOG_SKIP` (comma separated) or
/// [`ActivityLogConfig::skip_suffixes`]; an empty list logs everything.
pub const DEFAULT_SKIPPED_VARIABLE_SUFFIXES: &[&str] =
    &["_elapsed_executing_ms", "_elapsed_disabled_ms"];

/// 5 MiB, the default rotation threshold.
pub const DEFAULT_MAX_BYTES: u64 = 5 * 1024 * 1024;

/// How many rotated files to keep alongside the active one.
pub const DEFAULT_MAX_FILES: usize = 10;

/// Values longer than this are truncated with a trailing `…` on the way out.
/// Transforms and array values serialise to hundreds of characters and would
/// otherwise make the file unreadable and blow through the size budget.
pub const DEFAULT_MAX_VALUE_LEN: usize = 120;

/// How many events may be in flight before emission starts dropping them.
pub const DEFAULT_QUEUE_CAPACITY: usize = 8192;

/// The `log` target this module reports its own troubles under, and the one
/// target [`log_message`] refuses to record. See the guard there for why.
pub const LOG_TARGET: &str = "activity_log";

/// What kind of thing a line describes. This is the second column of the file,
/// and the thing you grep for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityKind {
    /// An operation changed state (`Initial -> Executing`, ...).
    Operation,
    /// An automatic transition fired.
    Transition,
    /// A SOP was started, advanced or torn down.
    Sop,
    /// A state variable took a new value.
    Variable,
    /// A console log line, mirrored here by the logger
    /// [`initialize_env_logger`](crate::initialize_env_logger) installs.
    ///
    /// Carrying the `log::Level` rather than one variant per severity keeps
    /// [`tag`](ActivityKind::tag) exhaustive over every level the `log` crate
    /// has, so none of them can end up silently unlabelled.
    Log(Level),
}

impl ActivityKind {
    /// The fixed-width tag written to the file.
    pub fn tag(&self) -> &'static str {
        match self {
            ActivityKind::Operation => "OP",
            ActivityKind::Transition => "TRANS",
            ActivityKind::Sop => "SOP",
            ActivityKind::Variable => "VAR",
            // `ERR` rather than `ERROR` so every tag fits the five-character
            // kind column that `format_record` pads to.
            ActivityKind::Log(Level::Error) => "ERR",
            ActivityKind::Log(Level::Warn) => "WARN",
            ActivityKind::Log(Level::Info) => "INFO",
            ActivityKind::Log(Level::Debug) => "DEBUG",
            ActivityKind::Log(Level::Trace) => "TRACE",
        }
    }
}

/// One line of the log, stamped when it happened rather than when it was
/// written - the queue can hold an event for a moment, and a timestamp that
/// drifted with writer latency would be useless for reconstructing an ordering.
#[derive(Debug, Clone)]
pub struct ActivityRecord {
    /// When the event happened, not when the line was written.
    pub at: DateTime<Local>,
    /// Which of the four kinds of event this is.
    pub kind: ActivityKind,
    /// Which runner produced this - the `log_target` the runners already carry,
    /// e.g. `sp_operation_runner`.
    pub source: String,
    /// What it is about: an operation name, a transition name, a SOP id, a
    /// variable name.
    pub subject: String,
    /// The change itself, formatted per kind.
    pub detail: String,
}

impl ActivityRecord {
    /// A record stamped with the current local time.
    pub fn new(kind: ActivityKind, source: &str, subject: &str, detail: String) -> Self {
        Self {
            at: Local::now(),
            kind,
            source: source.to_string(),
            subject: subject.to_string(),
            detail,
        }
    }
}

/// Where the log goes and how big it is allowed to get.
#[derive(Debug, Clone)]
pub struct ActivityLogConfig {
    /// Directory for the active and rotated files. Created if missing.
    pub dir: PathBuf,
    /// Base name; the active file is `{stem}.log`.
    pub stem: String,
    /// Rotate once the active file would pass this size.
    pub max_bytes: u64,
    /// Rotated files to keep. `0` keeps all of them.
    pub max_files: usize,
    /// Variable-name suffixes that produce no `VAR` line.
    pub skip_suffixes: Vec<String>,
    /// Truncate rendered values beyond this many characters.
    pub max_value_len: usize,
    /// Bounded queue depth; events are dropped rather than blocking when full.
    pub queue_capacity: usize,
}

impl Default for ActivityLogConfig {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("micro_sp_logs"),
            stem: "micro_sp".to_string(),
            max_bytes: DEFAULT_MAX_BYTES,
            max_files: DEFAULT_MAX_FILES,
            skip_suffixes: DEFAULT_SKIPPED_VARIABLE_SUFFIXES
                .iter()
                .map(|s| s.to_string())
                .collect(),
            max_value_len: DEFAULT_MAX_VALUE_LEN,
            queue_capacity: DEFAULT_QUEUE_CAPACITY,
        }
    }
}

impl ActivityLogConfig {
    /// Build a config from the environment, or `None` if file logging was not
    /// asked for.
    ///
    /// Enabled by either `MICRO_SP_ACTIVITY_LOG` being truthy (`1`, `true`,
    /// `on`, `yes`, case-insensitive) or `MICRO_SP_ACTIVITY_LOG_DIR` being set -
    /// naming a directory is a clear enough statement of intent that it would
    /// be annoying to also require the flag. Being off unless asked matters
    /// because this is a library: importing it must not start writing files
    /// into somebody's working directory.
    ///
    /// Also honoured: `MICRO_SP_ACTIVITY_LOG_MAX_MB`,
    /// `MICRO_SP_ACTIVITY_LOG_MAX_FILES`, `MICRO_SP_ACTIVITY_LOG_SKIP`.
    pub fn from_env() -> Option<Self> {
        let flag = std::env::var("MICRO_SP_ACTIVITY_LOG").ok();
        let dir = std::env::var("MICRO_SP_ACTIVITY_LOG_DIR").ok();

        let flag_on = flag
            .as_deref()
            .map(|v| is_truthy(v))
            .unwrap_or(false);
        if !flag_on && dir.is_none() {
            return None;
        }

        let mut cfg = ActivityLogConfig::default();
        if let Some(dir) = dir {
            cfg.dir = PathBuf::from(dir);
        }
        if let Some(mb) = read_env_number("MICRO_SP_ACTIVITY_LOG_MAX_MB") {
            // A zero here would rotate on every single line; treat it as "use
            // the default" rather than as a foot-gun.
            if mb > 0 {
                cfg.max_bytes = mb * 1024 * 1024;
            }
        }
        if let Some(n) = read_env_number("MICRO_SP_ACTIVITY_LOG_MAX_FILES") {
            cfg.max_files = n as usize;
        }
        if let Ok(skip) = std::env::var("MICRO_SP_ACTIVITY_LOG_SKIP") {
            cfg.skip_suffixes = skip
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        Some(cfg)
    }

    /// True when a variable with this name should produce a `VAR` line.
    pub fn logs_variable(&self, name: &str) -> bool {
        !self
            .skip_suffixes
            .iter()
            .any(|suffix| name.ends_with(suffix.as_str()))
    }
}

fn is_truthy(v: &str) -> bool {
    matches!(
        v.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "on" | "yes"
    )
}

fn read_env_number(key: &str) -> Option<u64> {
    std::env::var(key).ok()?.trim().parse::<u64>().ok()
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

/// Collapse anything that would break the one-record-per-line invariant.
///
/// Operation information strings are genuinely multi-line - the `Disabled` arm
/// of `process_operation` renders a whole predicate tree with embedded newlines
/// and indentation - and a raw newline in the middle of a record would make
/// every line-oriented tool (grep, tail, wc) disagree about where records
/// start.
fn sanitize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("    "),
            c => out.push(c),
        }
    }
    out
}

/// Truncate on a character boundary, never mid-codepoint.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    // Leave room for the ellipsis so the result still fits in `max`.
    let keep = max.saturating_sub(1);
    let mut out: String = s.chars().take(keep).collect();
    out.push('…');
    out
}

/// Render a value for the `old -> new` half of a `VAR` line.
pub fn render_value(value: &SPValue, max_len: usize) -> String {
    truncate(&sanitize(&value.to_string()), max_len)
}

/// Render one record as the exact bytes that go into the file, newline
/// included.
///
/// The columns are padded to fixed widths so the file lines up when read
/// straight, but the padding is a *minimum*: a name longer than its column
/// pushes the rest of the line right rather than being cut, because a truncated
/// operation name is much worse to debug against than a ragged column. Only the
/// free-form `detail` is length-limited, and only per value.
pub fn format_record(record: &ActivityRecord) -> String {
    format!(
        "{} | {:<5} | {:<26} | {:<34} | {}\n",
        record.at.format("%Y-%m-%d %H:%M:%S%.3f"),
        record.kind.tag(),
        sanitize(&record.source),
        sanitize(&record.subject),
        sanitize(&record.detail)
    )
}

/// The banner written at the top of every newly created file, so a file found
/// on its own is self-describing.
fn header(config: &ActivityLogConfig) -> String {
    format!(
        "# micro_sp activity log - opened {}\n\
         # columns: timestamp | kind | source | subject | detail\n\
         # kinds:   OP = operation state change, TRANS = auto transition taken,\n\
         #          SOP = sop lifecycle, VAR = variable value change,\n\
         #          ERR/WARN/INFO/DEBUG/TRACE = a console log line\n\
         # rotates at {} MiB, keeping {}\n\
         #\n",
        Local::now().format("%Y-%m-%d %H:%M:%S%.3f %:z"),
        config.max_bytes as f64 / (1024.0 * 1024.0),
        match config.max_files {
            0 => "every rotated file".to_string(),
            n => format!("the {n} most recent rotated files"),
        }
    )
}

// ---------------------------------------------------------------------------
// The rotating writer
// ---------------------------------------------------------------------------

/// Owns the active file and rotates it. Deliberately free of globals and of
/// any channel, so the rotation, pruning and filtering rules can be tested
/// directly rather than through a background thread.
pub struct ActivityWriter {
    config: ActivityLogConfig,
    file: Option<BufWriter<File>>,
    written: u64,
}

impl ActivityWriter {
    /// Open (or re-open) the active file, creating `dir` if needed.
    ///
    /// An existing active file is appended to rather than truncated, and its
    /// current length seeds the rotation counter - so a restarted process
    /// continues the same file and still rotates it at the right size instead
    /// of destroying the history of the run that just ended.
    pub fn new(config: ActivityLogConfig) -> io::Result<Self> {
        fs::create_dir_all(&config.dir)?;
        let path = active_path(&config);
        let existing = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let fresh = existing == 0;

        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        let mut writer = Self {
            config,
            file: Some(BufWriter::new(file)),
            written: existing,
        };
        if fresh {
            writer.write_header()?;
        }
        Ok(writer)
    }

    /// Path of the file currently being appended to.
    pub fn current_path(&self) -> PathBuf {
        active_path(&self.config)
    }

    /// The configuration this writer was created with.
    pub fn config(&self) -> &ActivityLogConfig {
        &self.config
    }

    fn write_header(&mut self) -> io::Result<()> {
        let banner = header(&self.config);
        if let Some(file) = self.file.as_mut() {
            file.write_all(banner.as_bytes())?;
        }
        self.written += banner.len() as u64;
        Ok(())
    }

    /// Append one record, rotating first if it would not fit.
    pub fn write_record(&mut self, record: &ActivityRecord) -> io::Result<()> {
        self.write_line(&format_record(record))
    }

    /// Append a pre-rendered line. Used for the record path and for the
    /// "events were dropped" notice, which is not an `ActivityRecord`.
    pub fn write_line(&mut self, line: &str) -> io::Result<()> {
        let len = line.len() as u64;
        // Rotate *before* writing, so a file never exceeds its budget. The
        // `written > 0` guard stops a single line larger than `max_bytes` from
        // rotating forever without ever making progress.
        if self.written > 0 && self.written + len > self.config.max_bytes {
            self.rotate()?;
        }
        if let Some(file) = self.file.as_mut() {
            file.write_all(line.as_bytes())?;
        }
        self.written += len;
        Ok(())
    }

    /// Push buffered bytes to the OS. Called after each drained batch, so a
    /// quiet system still has its last events on disk.
    pub fn flush(&mut self) -> io::Result<()> {
        if let Some(file) = self.file.as_mut() {
            file.flush()?;
        }
        Ok(())
    }

    /// Close the active file, move it aside under a timestamped name, and open
    /// a fresh one.
    fn rotate(&mut self) -> io::Result<()> {
        if let Some(mut file) = self.file.take() {
            file.flush()?;
        }

        let from = active_path(&self.config);
        let to = self.next_rotated_path();
        // A failed rename must not lose the writer: fall through and re-open
        // the active file either way, so logging continues (into the same file,
        // which will simply be over budget) rather than stopping silently.
        if let Err(e) = fs::rename(&from, &to) {
            log::warn!(
                target: LOG_TARGET,
                "Could not rotate {} to {}: {e}. Continuing in the current file.",
                from.display(),
                to.display()
            );
            let file = OpenOptions::new().create(true).append(true).open(&from)?;
            self.file = Some(BufWriter::new(file));
            return Ok(());
        }

        let file = OpenOptions::new().create(true).append(true).open(&from)?;
        self.file = Some(BufWriter::new(file));
        self.written = 0;
        self.write_header()?;
        self.prune();
        Ok(())
    }

    /// `{stem}-{YYYYMMDD}-{HHMMSS}.log`, with a `-002`, `-003`, ...
    /// disambiguator if the log rotated more than once inside the same second
    /// (which happens under a small `max_bytes`, and would otherwise silently
    /// overwrite the file just archived).
    ///
    /// The counter is zero-padded because [`prune`](Self::prune) establishes
    /// age by sorting these names as strings. Unpadded, `-10` sorts before
    /// `-2`, so a burst of same-second rotations would make pruning delete the
    /// *newest* files and keep the oldest - the exact opposite of retention.
    fn next_rotated_path(&self) -> PathBuf {
        let stamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
        let base = self.config.dir.join(format!("{}-{stamp}.log", self.config.stem));
        if !base.exists() {
            return base;
        }
        for n in 2..10_000 {
            let candidate = self
                .config
                .dir
                .join(format!("{}-{stamp}-{n:04}.log", self.config.stem));
            if !candidate.exists() {
                return candidate;
            }
        }
        base
    }

    /// Delete the oldest rotated files beyond `max_files`.
    ///
    /// The names embed a zero-padded timestamp, so lexicographic order is
    /// chronological order and no filesystem metadata has to be trusted.
    fn prune(&self) {
        if self.config.max_files == 0 {
            return;
        }
        let mut rotated = self.rotated_files();
        if rotated.len() <= self.config.max_files {
            return;
        }
        rotated.sort();
        let excess = rotated.len() - self.config.max_files;
        for path in rotated.into_iter().take(excess) {
            if let Err(e) = fs::remove_file(&path) {
                log::warn!(
                    target: LOG_TARGET,
                    "Could not remove old activity log {}: {e}", path.display()
                );
            }
        }
    }

    /// Every rotated file in the directory, excluding the active one.
    fn rotated_files(&self) -> Vec<PathBuf> {
        let prefix = format!("{}-", self.config.stem);
        let active = active_path(&self.config);
        let Ok(entries) = fs::read_dir(&self.config.dir) else {
            return Vec::new();
        };
        entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p != &active)
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with(&prefix) && n.ends_with(".log"))
                    .unwrap_or(false)
            })
            .collect()
    }
}

fn active_path(config: &ActivityLogConfig) -> PathBuf {
    config.dir.join(format!("{}.log", config.stem))
}

// ---------------------------------------------------------------------------
// The background thread and the process-wide handle
// ---------------------------------------------------------------------------

enum Msg {
    Record(Box<ActivityRecord>),
    /// Write everything queued so far, then answer. Used by
    /// [`flush`] so a caller (a test, or a shutdown path) can be sure the file
    /// on disk is current.
    Flush(SyncSender<()>),
}

static SENDER: OnceLock<SyncSender<Msg>> = OnceLock::new();
static DROPPED: AtomicU64 = AtomicU64::new(0);

/// Install the process-wide activity log.
///
/// Returns `false` if one is already installed or the directory could not be
/// opened; like [`crate::initialize_env_logger`] this is safe to call from
/// several places, and only the first call wins.
pub fn init(config: ActivityLogConfig) -> bool {
    if SENDER.get().is_some() {
        return false;
    }
    let writer = match ActivityWriter::new(config) {
        Ok(w) => w,
        Err(e) => {
            log::error!(
                target: LOG_TARGET,
                "Could not open the activity log: {e}. File logging is off for this process."
            );
            return false;
        }
    };
    let path = writer.current_path();
    // The emission path filters without touching the writer thread, so it needs
    // its own copy of the filtering rules.
    install_filter(writer.config());
    let (tx, rx) = sync_channel::<Msg>(writer.config().queue_capacity);
    if SENDER.set(tx).is_err() {
        // Another thread won the race between the check above and here.
        return false;
    }

    let spawned = std::thread::Builder::new()
        .name("micro-sp-activity-log".to_string())
        .spawn(move || writer_loop(writer, rx));

    match spawned {
        Ok(_) => {
            log::info!(target: LOG_TARGET, "Activity log writing to {}.", path.display());
            true
        }
        Err(e) => {
            log::error!(target: LOG_TARGET, "Could not start the activity log thread: {e}.");
            false
        }
    }
}

/// Install from the environment; a no-op when file logging was not requested.
/// See [`ActivityLogConfig::from_env`] for the variables.
pub fn init_from_env() -> bool {
    match ActivityLogConfig::from_env() {
        Some(config) => init(config),
        None => false,
    }
}

/// True once a log is installed. Emission sites use this to bail out before
/// doing any formatting work.
pub fn is_enabled() -> bool {
    SENDER.get().is_some()
}

/// How many events have been dropped because the queue was full.
pub fn dropped_count() -> u64 {
    DROPPED.load(Ordering::Relaxed)
}

/// Block until everything queued so far has reached the file.
///
/// Returns `false` if no log is installed or the writer thread is gone.
pub fn flush() -> bool {
    let Some(tx) = SENDER.get() else {
        return false;
    };
    let (ack_tx, ack_rx) = sync_channel::<()>(1);
    if tx.send(Msg::Flush(ack_tx)).is_err() {
        return false;
    }
    ack_rx.recv().is_ok()
}

/// Queue one record, or count it as dropped. Never blocks.
fn emit(record: ActivityRecord) {
    let Some(tx) = SENDER.get() else {
        return;
    };
    match tx.try_send(Msg::Record(Box::new(record))) {
        Ok(()) => {}
        Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {
            DROPPED.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Drain the channel onto disk until every sender is gone.
///
/// Writes are batched: one blocking `recv` followed by a greedy `try_recv`
/// drain, then a single flush. Under load that turns thousands of small writes
/// into a handful of syscalls; when idle it costs one flush per event.
fn writer_loop(mut writer: ActivityWriter, rx: Receiver<Msg>) {
    let mut reported_drops: u64 = 0;

    while let Ok(first) = rx.recv() {
        let mut pending_ack = None;
        let handle = |msg: Msg, writer: &mut ActivityWriter, ack: &mut Option<SyncSender<()>>| {
            match msg {
                Msg::Record(record) => {
                    if let Err(e) = writer.write_record(&record) {
                        log::error!(target: LOG_TARGET, "Activity log write failed: {e}.");
                    }
                }
                Msg::Flush(tx) => *ack = Some(tx),
            }
        };

        handle(first, &mut writer, &mut pending_ack);
        while let Ok(msg) = rx.try_recv() {
            handle(msg, &mut writer, &mut pending_ack);
        }

        // Record any gap before flushing, so the notice lands in the same file
        // as the events around it.
        let dropped = DROPPED.load(Ordering::Relaxed);
        if dropped > reported_drops {
            // Formatted through `format_record` like everything else, but
            // written with `write_line` rather than queued: this runs *on* the
            // writer thread, so it cannot wait on the writer thread.
            let line = format_record(&ActivityRecord::new(
                ActivityKind::Log(Level::Warn),
                LOG_TARGET,
                "-",
                format!("{} events dropped (queue full)", dropped - reported_drops),
            ));
            let _ = writer.write_line(&line);
            reported_drops = dropped;
        }

        if let Err(e) = writer.flush() {
            log::error!(target: LOG_TARGET, "Activity log flush failed: {e}.");
        }
        if let Some(ack) = pending_ack {
            let _ = ack.send(());
        }
    }

    let _ = writer.flush();
}

// ---------------------------------------------------------------------------
// Emission helpers - what the runners call
// ---------------------------------------------------------------------------

/// One console log line, recorded to the file alongside the events.
///
/// The logger [`initialize_env_logger`](crate::initialize_env_logger) installs
/// forwards everything it prints through here, so a single pass over the file
/// shows the prose explaining *why* something happened interleaved with the
/// `OP`/`TRANS`/`SOP`/`VAR` lines saying *what* happened - the two used to live
/// in a file and a terminal scrollback respectively, and correlating them after
/// the fact was hopeless.
///
/// `source` is the `log` target (the runners' `log_target`, so it lands in the
/// same column as an event's source) and `location` is the `file:line` of the
/// statement that emitted it.
pub fn log_message(level: Level, source: &str, location: &str, message: String) {
    if !is_enabled() {
        return;
    }
    // The writer thread reports its own failures with
    // `log::error!(target: LOG_TARGET, "Activity log write failed: ...")`. If
    // those came back through here, one failed write would queue a record whose
    // write fails, which logs, which queues... - an unbounded loop out of a
    // single bad disk. The log never records its own lines, so it cannot feed
    // itself.
    if source == LOG_TARGET {
        return;
    }
    emit(ActivityRecord::new(
        ActivityKind::Log(level),
        source,
        location,
        message,
    ));
}

/// An operation changed state, e.g. `Initial -> Executing`.
///
/// `note` is the short tag the runner already computes (`Starting`,
/// `Completing`, `Retrying 2/3`, ...).
pub fn log_operation(source: &str, operation: &str, from: &str, to: &str, note: &str) {
    if !is_enabled() {
        return;
    }
    let detail = if note.is_empty() {
        format!("{from} -> {to}")
    } else {
        format!("{from} -> {to}  ({note})")
    };
    emit(ActivityRecord::new(
        ActivityKind::Operation,
        source,
        operation,
        detail,
    ));
}

/// An automatic transition fired. `unique_name` carries the per-firing id the
/// runner generates, which is what ties this line to the variable changes it
/// caused on the same tick.
pub fn log_transition(source: &str, transition: &str, unique_name: &str) {
    if !is_enabled() {
        return;
    }
    emit(ActivityRecord::new(
        ActivityKind::Transition,
        source,
        transition,
        format!("taken as '{unique_name}'"),
    ));
}

/// A SOP was started, advanced, or torn down.
pub fn log_sop(source: &str, sop: &str, from: &str, to: &str, note: &str) {
    if !is_enabled() {
        return;
    }
    let detail = if note.is_empty() {
        format!("{from} -> {to}")
    } else {
        format!("{from} -> {to}  ({note})")
    };
    emit(ActivityRecord::new(ActivityKind::Sop, source, sop, detail));
}

/// One variable took a new value. `old` is `None` when the variable is being
/// introduced rather than changed.
pub fn log_variable(source: &str, name: &str, old: Option<&SPValue>, new: &SPValue) {
    if !is_enabled() {
        return;
    }
    emit_variable(source, name, old, new, DEFAULT_MAX_VALUE_LEN);
}

fn emit_variable(source: &str, name: &str, old: Option<&SPValue>, new: &SPValue, max_len: usize) {
    let detail = match old {
        Some(old) => format!(
            "{} -> {}",
            render_value(old, max_len),
            render_value(new, max_len)
        ),
        None => format!("(new) -> {}", render_value(new, max_len)),
    };
    emit(ActivityRecord::new(
        ActivityKind::Variable,
        source,
        name,
        detail,
    ));
}

/// Log every variable in a tick's delta, looking each one's previous value up
/// in the state the delta was computed against.
///
/// This is the call the runners make: they already compute
/// `old.get_diff_partial_state_and_add_missing(&new)` to decide what to write
/// to Redis, and that delta is exactly the set of variables that changed. Doing
/// it here rather than inside `StateManager` keeps the write path free of
/// logging concerns and gives the line the runner's own `log_target`.
///
/// Filtering happens against the installed config, so a variable excluded by
/// [`ActivityLogConfig::skip_suffixes`] costs one suffix check and no
/// allocation.
pub fn log_state_diff(source: &str, old: &State, delta: &State) {
    if !is_enabled() || delta.state.is_empty() {
        return;
    }
    let (skip, max_len) = filter_settings();
    for (name, assignment) in &delta.state {
        if skip.iter().any(|suffix| name.ends_with(suffix.as_str())) {
            continue;
        }
        let previous = old.state.get(name).map(|a| &a.val);
        emit_variable(source, name, previous, &assignment.val, max_len);
    }
}

/// The filter half of the config, kept where the emission path can reach it
/// without going through the writer thread.
static FILTER: OnceLock<(Vec<String>, usize)> = OnceLock::new();

fn filter_settings() -> (Vec<String>, usize) {
    FILTER
        .get()
        .cloned()
        .unwrap_or_else(|| {
            (
                DEFAULT_SKIPPED_VARIABLE_SUFFIXES
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                DEFAULT_MAX_VALUE_LEN,
            )
        })
}

/// Record the filtering rules for the emission path. Called by [`init`].
fn install_filter(config: &ActivityLogConfig) {
    let _ = FILTER.set((config.skip_suffixes.clone(), config.max_value_len));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SPAssignment, SPValueType, SPVariable, ToSPValue};
    use std::path::Path;

    fn temp_dir(tag: &str) -> PathBuf {
        // A per-test directory under the OS temp dir, keyed by test name plus
        // the thread id so a parallel run cannot collide.
        let unique = format!(
            "micro_sp_activity_{tag}_{:?}_{}",
            std::thread::current().id(),
            std::process::id()
        )
        .replace(['(', ')', ' '], "");
        let dir = std::env::temp_dir().join(unique);
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn config_in(dir: &Path) -> ActivityLogConfig {
        ActivityLogConfig {
            dir: dir.to_path_buf(),
            ..Default::default()
        }
    }

    fn record(kind: ActivityKind, subject: &str, detail: &str) -> ActivityRecord {
        ActivityRecord::new(kind, "test_runner", subject, detail.to_string())
    }

    fn state_of(pairs: &[(&str, SPValue)]) -> State {
        let mut state = State::new();
        for (name, value) in pairs {
            let kind = match value {
                SPValue::Bool(_) => SPValueType::Bool,
                SPValue::Int64(_) => SPValueType::Int64,
                _ => SPValueType::String,
            };
            state.add_mut(
                SPAssignment::new(SPVariable::new(name, kind), value.clone()),
                "test",
            );
        }
        state
    }

    // -- formatting ------------------------------------------------------

    /// The whole file is line-oriented, so a record containing a newline - and
    /// `process_operation`'s "disabled" message genuinely does, it renders a
    /// predicate tree across several indented lines - must still occupy
    /// exactly one line. Otherwise `grep`, `wc -l` and every log shipper
    /// disagree about where a record starts.
    #[test]
    fn a_record_is_always_exactly_one_line() {
        let multiline = record(
            ActivityKind::Operation,
            "op_move",
            "Initial -> Disabled\nplease satisfy:\n\tvar == 1\r\n",
        );
        let line = format_record(&multiline);

        assert_eq!(
            line.matches('\n').count(),
            1,
            "exactly one newline, the terminator: {line:?}"
        );
        assert!(line.ends_with('\n'));
        assert!(line.contains("\\n"), "embedded newlines are escaped");
        assert!(line.contains("\\r"), "embedded carriage returns are escaped");
    }

    /// The columns are what make the file skimmable, and the kind tag is what
    /// makes it greppable.
    #[test]
    fn the_columns_are_ordered_and_tagged() {
        let line = format_record(&record(ActivityKind::Variable, "robot_pose", "home -> at_b"));
        let fields: Vec<&str> = line.trim_end().split(" | ").collect();

        assert_eq!(fields.len(), 5, "timestamp|kind|source|subject|detail");
        assert_eq!(fields[1].trim(), "VAR");
        assert_eq!(fields[2].trim(), "test_runner");
        assert_eq!(fields[3].trim(), "robot_pose");
        assert_eq!(fields[4], "home -> at_b");
        // The timestamp has to sort lexicographically, which is why it is
        // %Y-%m-%d and not anything locale-shaped.
        assert!(
            fields[0].starts_with(&Local::now().format("%Y-%m-%d").to_string()),
            "got {:?}",
            fields[0]
        );
    }

    /// Every kind gets a distinct tag; grepping `| OP ` must not also match
    /// SOP lines.
    #[test]
    fn kind_tags_are_distinct_and_greppable() {
        let tags = [
            ActivityKind::Operation.tag(),
            ActivityKind::Transition.tag(),
            ActivityKind::Sop.tag(),
            ActivityKind::Variable.tag(),
            ActivityKind::Log(Level::Error).tag(),
            ActivityKind::Log(Level::Warn).tag(),
            ActivityKind::Log(Level::Info).tag(),
            ActivityKind::Log(Level::Debug).tag(),
            ActivityKind::Log(Level::Trace).tag(),
        ];
        let unique: std::collections::HashSet<_> = tags.iter().collect();
        assert_eq!(
            unique.len(),
            tags.len(),
            "a tag is doing double duty: {tags:?}"
        );

        let op = format_record(&record(ActivityKind::Operation, "x", "d"));
        let sop = format_record(&record(ActivityKind::Sop, "x", "d"));
        assert!(op.contains("| OP    |"));
        assert!(!sop.contains("| OP    |"), "SOP must not match an OP grep");
    }

    /// The mirrored console lines share the kind column with the event kinds,
    /// so their tags have to be the levels a reader would grep for and they
    /// have to fit the column - a sixth character would push every field on
    /// those lines out of alignment with every other line in the file.
    #[test]
    fn log_levels_are_tagged_and_fit_the_kind_column() {
        let expected = [
            (Level::Error, "ERR"),
            (Level::Warn, "WARN"),
            (Level::Info, "INFO"),
            (Level::Debug, "DEBUG"),
            (Level::Trace, "TRACE"),
        ];
        for (level, tag) in expected {
            assert_eq!(ActivityKind::Log(level).tag(), tag);
            assert!(tag.len() <= 5, "{tag} does not fit the kind column");
        }

        // And the columns still line up with an event line written beside it.
        let info = format_record(&record(
            ActivityKind::Log(Level::Info),
            "tick.rs:88",
            "started",
        ));
        let var = format_record(&record(
            ActivityKind::Variable,
            "robot_pose",
            "home -> at_b",
        ));
        let column_of = |line: &str| line.find(" | ").map(|i| i + 3);
        assert_eq!(column_of(&info), column_of(&var));
        assert!(info.contains("| INFO  |"));
    }

    /// A mirrored `log::error!` argument is arbitrary prose, and plenty of the
    /// crate's own messages are multi-line - `process_operation`'s "disabled"
    /// arm renders a whole predicate tree. One log call must still be one line
    /// with the same five fields as everything else.
    #[test]
    fn a_mirrored_log_line_keeps_the_five_columns() {
        let line = format_record(&ActivityRecord::new(
            ActivityKind::Log(Level::Error),
            "sp_operation_runner",
            "running/process_operation.rs:214",
            "Operation disabled, please satisfy:\n\tvar == 1\n".to_string(),
        ));

        assert_eq!(line.matches('\n').count(), 1, "one line: {line:?}");
        let fields: Vec<&str> = line.trim_end().split(" | ").collect();
        assert_eq!(fields.len(), 5, "timestamp|kind|source|subject|detail");
        assert_eq!(fields[1].trim(), "ERR");
        assert_eq!(fields[2].trim(), "sp_operation_runner");
        assert_eq!(fields[3].trim(), "running/process_operation.rs:214");
        assert!(fields[4].contains("\\n"), "embedded newlines are escaped");
    }

    /// Values are truncated so one transform cannot take a whole line, and the
    /// truncation must be UTF-8 safe - `SPValue::String` can hold anything.
    #[test]
    fn long_and_multibyte_values_are_truncated_safely() {
        let long = SPValue::String(crate::StringOrUnknown::String("å".repeat(500)));
        let rendered = render_value(&long, 20);

        assert_eq!(rendered.chars().count(), 20);
        assert!(rendered.ends_with('…'));
        // The real assertion: this did not panic on a char boundary, and the
        // result is still valid UTF-8 that round-trips.
        assert_eq!(rendered, String::from_utf8(rendered.clone().into_bytes()).unwrap());
    }

    /// A value that already fits is left completely alone - no stray ellipsis.
    #[test]
    fn short_values_are_untouched() {
        assert_eq!(render_value(&"home".to_spvalue(), 120), "home");
        assert_eq!(render_value(&42.to_spvalue(), 120), "42");
    }

    // -- the writer, rotation, pruning ------------------------------------

    /// The baseline: a new file gets the self-describing header, and records
    /// land after it in order.
    #[test]
    fn a_new_file_gets_a_header_and_then_the_records() {
        let dir = temp_dir("header");
        let mut writer = ActivityWriter::new(config_in(&dir)).unwrap();

        writer.write_record(&record(ActivityKind::Operation, "op_a", "Initial -> Executing")).unwrap();
        writer.write_record(&record(ActivityKind::Sop, "sop_a", "Initial -> Executing")).unwrap();
        writer.flush().unwrap();

        let text = fs::read_to_string(writer.current_path()).unwrap();
        assert!(text.starts_with("# micro_sp activity log"));
        assert!(text.contains("# columns: timestamp | kind | source | subject | detail"));

        let op_at = text.find("op_a").unwrap();
        let sop_at = text.find("sop_a").unwrap();
        assert!(op_at < sop_at, "records are appended in the order written");

        fs::remove_dir_all(&dir).ok();
    }

    /// The active file has a stable name so `tail -f micro_sp.log` keeps
    /// working across rotations.
    #[test]
    fn the_active_file_has_a_stable_name() {
        let dir = temp_dir("stable");
        let writer = ActivityWriter::new(config_in(&dir)).unwrap();
        assert_eq!(writer.current_path(), dir.join("micro_sp.log"));
        fs::remove_dir_all(&dir).ok();
    }

    /// The point of the whole exercise: the active file never exceeds the size
    /// budget, and what was there before is preserved under a new name rather
    /// than discarded.
    #[test]
    fn the_file_rotates_at_the_size_limit_and_keeps_the_old_content() {
        let dir = temp_dir("rotate");
        // Retention off, so this isolates "rotation preserves what was already
        // written" from "pruning deletes old files on purpose", which is a
        // separate test. With the default limit the assertion below would be
        // testing both at once and would fail for the *right* reason.
        let config = ActivityLogConfig {
            max_bytes: 2_000,
            max_files: 0,
            ..config_in(&dir)
        };
        let mut writer = ActivityWriter::new(config).unwrap();

        for i in 0..200 {
            writer
                .write_record(&record(ActivityKind::Variable, &format!("var_{i}"), "a -> b"))
                .unwrap();
        }
        writer.flush().unwrap();

        let rotated = writer.rotated_files();
        assert!(
            !rotated.is_empty(),
            "200 records over a 2 KB budget must have rotated at least once"
        );

        // The invariant that matters: nothing on disk is over budget.
        let active_len = fs::metadata(writer.current_path()).unwrap().len();
        assert!(
            active_len <= 2_000,
            "active file is {active_len} bytes, over the 2000 byte budget"
        );
        for path in &rotated {
            let len = fs::metadata(path).unwrap().len();
            assert!(len <= 2_000, "{} is {len} bytes, over budget", path.display());
        }

        // And no record was lost across the rotations.
        let mut all = fs::read_to_string(writer.current_path()).unwrap();
        for path in &rotated {
            all.push_str(&fs::read_to_string(path).unwrap());
        }
        for i in 0..200 {
            assert!(all.contains(&format!("var_{i} ")), "var_{i} vanished in a rotation");
        }

        fs::remove_dir_all(&dir).ok();
    }

    /// Every rotated file is independently readable, which is the reason the
    /// header is re-emitted rather than written once per process.
    #[test]
    fn every_rotated_file_carries_its_own_header() {
        let dir = temp_dir("headers");
        let config = ActivityLogConfig {
            max_bytes: 1_500,
            ..config_in(&dir)
        };
        let mut writer = ActivityWriter::new(config).unwrap();
        for i in 0..100 {
            writer
                .write_record(&record(ActivityKind::Operation, &format!("op_{i}"), "x -> y"))
                .unwrap();
        }
        writer.flush().unwrap();

        for path in writer.rotated_files() {
            let text = fs::read_to_string(&path).unwrap();
            assert!(
                text.starts_with("# micro_sp activity log"),
                "{} is not self-describing",
                path.display()
            );
        }
        fs::remove_dir_all(&dir).ok();
    }

    /// Retention: an unattended system must not fill its disk. The *newest*
    /// files are the ones kept.
    #[test]
    fn old_files_are_pruned_to_the_retention_limit() {
        let dir = temp_dir("prune");
        let config = ActivityLogConfig {
            max_bytes: 800,
            max_files: 3,
            ..config_in(&dir)
        };
        let mut writer = ActivityWriter::new(config).unwrap();

        for i in 0..400 {
            writer
                .write_record(&record(ActivityKind::Variable, &format!("v{i}"), "a -> b"))
                .unwrap();
        }
        writer.flush().unwrap();

        let rotated = writer.rotated_files();
        assert!(
            rotated.len() <= 3,
            "kept {} rotated files, limit is 3",
            rotated.len()
        );
        // The retained ones must be the most recent, i.e. the tail of the run.
        let newest = fs::read_to_string(writer.current_path()).unwrap();
        assert!(newest.contains("v399"), "the newest record must survive");

        fs::remove_dir_all(&dir).ok();
    }

    /// `max_files: 0` means "keep everything" - for a run being archived
    /// wholesale, where losing the beginning is worse than using disk.
    #[test]
    fn a_zero_retention_limit_keeps_every_file() {
        let dir = temp_dir("keepall");
        let config = ActivityLogConfig {
            max_bytes: 800,
            max_files: 0,
            ..config_in(&dir)
        };
        let mut writer = ActivityWriter::new(config).unwrap();
        for i in 0..300 {
            writer
                .write_record(&record(ActivityKind::Variable, &format!("v{i}"), "a -> b"))
                .unwrap();
        }
        writer.flush().unwrap();

        assert!(
            writer.rotated_files().len() > 3,
            "nothing should have been pruned"
        );
        fs::remove_dir_all(&dir).ok();
    }

    /// Restarting the process must continue the file rather than truncate it -
    /// otherwise every restart destroys the log of the run that just ended,
    /// which is exactly the run you want to read after a crash.
    #[test]
    fn reopening_appends_instead_of_truncating() {
        let dir = temp_dir("reopen");

        let mut first = ActivityWriter::new(config_in(&dir)).unwrap();
        first.write_record(&record(ActivityKind::Operation, "before_restart", "x -> y")).unwrap();
        first.flush().unwrap();
        drop(first);

        let mut second = ActivityWriter::new(config_in(&dir)).unwrap();
        second.write_record(&record(ActivityKind::Operation, "after_restart", "x -> y")).unwrap();
        second.flush().unwrap();

        let text = fs::read_to_string(second.current_path()).unwrap();
        assert!(text.contains("before_restart"), "the previous run was truncated away");
        assert!(text.contains("after_restart"));
        assert_eq!(
            text.matches("# micro_sp activity log").count(),
            1,
            "re-opening an existing file must not write a second header"
        );

        fs::remove_dir_all(&dir).ok();
    }

    /// A single record larger than the whole budget must still be written
    /// once, not send the writer into a rotate-forever loop.
    #[test]
    fn an_oversized_record_still_makes_progress() {
        let dir = temp_dir("oversized");
        let config = ActivityLogConfig {
            max_bytes: 64,
            ..config_in(&dir)
        };
        let mut writer = ActivityWriter::new(config).unwrap();

        let huge = record(ActivityKind::Variable, "big", &"x".repeat(4096));
        writer.write_record(&huge).unwrap();
        writer.write_record(&huge).unwrap();
        writer.flush().unwrap();

        let mut all = fs::read_to_string(writer.current_path()).unwrap();
        for path in writer.rotated_files() {
            all.push_str(&fs::read_to_string(path).unwrap());
        }
        let written = all
            .lines()
            .filter(|line| line.contains("| VAR") && line.contains("big"))
            .count();
        assert_eq!(written, 2, "both oversized records were written exactly once");

        fs::remove_dir_all(&dir).ok();
    }

    /// The directory is created on demand - a fresh deployment has no `logs/`.
    #[test]
    fn a_missing_directory_is_created() {
        let dir = temp_dir("mkdir").join("nested").join("deeper");
        assert!(!dir.exists());

        let mut writer = ActivityWriter::new(config_in(&dir)).unwrap();
        writer.write_record(&record(ActivityKind::Sop, "s", "a -> b")).unwrap();
        writer.flush().unwrap();

        assert!(dir.join("micro_sp.log").exists());
        fs::remove_dir_all(dir.parent().unwrap().parent().unwrap()).ok();
    }

    // -- filtering --------------------------------------------------------

    /// The per-tick elapsed counters are the reason a naive "log every diff"
    /// would be useless: they change on every tick of every active operation
    /// and would crowd out everything worth reading.
    #[test]
    fn the_per_tick_elapsed_counters_are_filtered_out_by_default() {
        let config = ActivityLogConfig::default();

        assert!(!config.logs_variable("op_move_elapsed_executing_ms"));
        assert!(!config.logs_variable("op_move_elapsed_disabled_ms"));

        // Everything a human actually wants survives the filter.
        assert!(config.logs_variable("op_move"));
        assert!(config.logs_variable("op_move_information"));
        assert!(config.logs_variable("robot_pose"));
        assert!(config.logs_variable("sp_plan_state"));
    }

    /// The filter is configurable, including "log absolutely everything".
    #[test]
    fn the_skip_list_is_configurable() {
        let noisy = ActivityLogConfig {
            skip_suffixes: vec!["_information".to_string()],
            ..Default::default()
        };
        assert!(!noisy.logs_variable("op_move_information"));
        // Replacing the list replaces it wholesale, so the defaults no longer
        // apply - that is what makes "log everything" expressible.
        assert!(noisy.logs_variable("op_move_elapsed_executing_ms"));

        let everything = ActivityLogConfig {
            skip_suffixes: vec![],
            ..Default::default()
        };
        assert!(everything.logs_variable("op_move_elapsed_executing_ms"));
    }

    // -- the state diff ---------------------------------------------------

    /// `log_state_diff` is fed the runner's own delta, so it has to pair each
    /// changed variable with its previous value and mark genuinely new ones as
    /// new rather than inventing an old value.
    #[test]
    fn a_diff_pairs_each_change_with_its_previous_value() {
        let old = state_of(&[("robot_pose", "home".to_spvalue()), ("count", 1.to_spvalue())]);
        let delta = state_of(&[
            ("robot_pose", "at_b".to_spvalue()),
            ("count", 2.to_spvalue()),
            ("freshly_added", "new_value".to_spvalue()),
        ]);

        // Render the same way the emission path does, without needing a global.
        let mut lines: Vec<String> = delta
            .state
            .iter()
            .map(|(name, assignment)| {
                let previous = old.state.get(name).map(|a| &a.val);
                match previous {
                    Some(p) => format!(
                        "{name}: {} -> {}",
                        render_value(p, 120),
                        render_value(&assignment.val, 120)
                    ),
                    None => format!("{name}: (new) -> {}", render_value(&assignment.val, 120)),
                }
            })
            .collect();
        lines.sort();

        assert_eq!(
            lines,
            vec![
                "count: 1 -> 2".to_string(),
                "freshly_added: (new) -> new_value".to_string(),
                "robot_pose: home -> at_b".to_string(),
            ]
        );
    }

    /// Emission must be free when nothing is installed - this is what lets the
    /// calls sit on the runners' hot paths unconditionally.
    #[test]
    fn every_emission_helper_is_a_no_op_when_no_log_is_installed() {
        // Nothing here may panic, and with no log installed nothing is queued.
        let before = dropped_count();

        log_operation("t", "op_a", "Initial", "Executing", "Starting");
        log_transition("t", "t_a", "t_a_abc123");
        log_sop("t", "sop_a", "Initial", "Executing", "");
        log_variable("t", "v", None, &"x".to_spvalue());
        log_state_diff("t", &State::new(), &state_of(&[("v", "x".to_spvalue())]));

        if !is_enabled() {
            assert!(!flush(), "flush reports that there is nothing to flush");
            assert_eq!(
                dropped_count(),
                before,
                "a disabled log discards events without counting them as drops"
            );
        }
    }

    /// An empty delta - the common case, since the runners only write when
    /// something changed - must not even look at the config.
    #[test]
    fn an_empty_diff_emits_nothing() {
        log_state_diff("t", &state_of(&[("a", 1.to_spvalue())]), &State::new());
        // Reaching here without a panic is the assertion; combined with the
        // early return in `log_state_diff` this is the idle-system path.
    }

    // -- config from the environment --------------------------------------

    /// Being off by default is a correctness property for a library: importing
    /// `micro_sp` must not start writing files into somebody's working
    /// directory.
    #[test]
    #[serial_test::serial(micro_sp_activity_log_env)]
    fn the_log_is_off_unless_the_environment_asks_for_it() {
        let _guard = EnvGuard::clear(&[
            "MICRO_SP_ACTIVITY_LOG",
            "MICRO_SP_ACTIVITY_LOG_DIR",
            "MICRO_SP_ACTIVITY_LOG_MAX_MB",
            "MICRO_SP_ACTIVITY_LOG_MAX_FILES",
            "MICRO_SP_ACTIVITY_LOG_SKIP",
        ]);
        assert!(ActivityLogConfig::from_env().is_none());
    }

    /// Either the flag or a directory turns it on; naming a directory is a
    /// clear enough statement of intent on its own.
    #[test]
    #[serial_test::serial(micro_sp_activity_log_env)]
    fn either_the_flag_or_a_directory_enables_it() {
        let _guard = EnvGuard::clear(&[
            "MICRO_SP_ACTIVITY_LOG",
            "MICRO_SP_ACTIVITY_LOG_DIR",
            "MICRO_SP_ACTIVITY_LOG_MAX_MB",
            "MICRO_SP_ACTIVITY_LOG_MAX_FILES",
            "MICRO_SP_ACTIVITY_LOG_SKIP",
        ]);

        unsafe { std::env::set_var("MICRO_SP_ACTIVITY_LOG", "true") };
        assert!(ActivityLogConfig::from_env().is_some());
        unsafe { std::env::set_var("MICRO_SP_ACTIVITY_LOG", "0") };
        assert!(ActivityLogConfig::from_env().is_none(), "an explicit 0 is off");

        unsafe { std::env::remove_var("MICRO_SP_ACTIVITY_LOG") };
        unsafe { std::env::set_var("MICRO_SP_ACTIVITY_LOG_DIR", "/tmp/somewhere") };
        let config = ActivityLogConfig::from_env().expect("a directory alone enables it");
        assert_eq!(config.dir, PathBuf::from("/tmp/somewhere"));
    }

    /// The tunables a deployment actually reaches for.
    #[test]
    #[serial_test::serial(micro_sp_activity_log_env)]
    fn size_retention_and_skip_list_come_from_the_environment() {
        let _guard = EnvGuard::clear(&[
            "MICRO_SP_ACTIVITY_LOG",
            "MICRO_SP_ACTIVITY_LOG_DIR",
            "MICRO_SP_ACTIVITY_LOG_MAX_MB",
            "MICRO_SP_ACTIVITY_LOG_MAX_FILES",
            "MICRO_SP_ACTIVITY_LOG_SKIP",
        ]);

        unsafe { std::env::set_var("MICRO_SP_ACTIVITY_LOG", "1") };
        unsafe { std::env::set_var("MICRO_SP_ACTIVITY_LOG_MAX_MB", "20") };
        unsafe { std::env::set_var("MICRO_SP_ACTIVITY_LOG_MAX_FILES", "3") };
        unsafe { std::env::set_var("MICRO_SP_ACTIVITY_LOG_SKIP", "_information, _counter") };

        let config = ActivityLogConfig::from_env().unwrap();
        assert_eq!(config.max_bytes, 20 * 1024 * 1024);
        assert_eq!(config.max_files, 3);
        assert!(!config.logs_variable("op_a_information"));
        assert!(!config.logs_variable("op_a_counter"), "whitespace is trimmed");
        assert!(config.logs_variable("op_a_elapsed_executing_ms"));
    }

    /// Garbage in the environment must fall back to the defaults rather than
    /// taking the process down or, worse, rotating on every line.
    #[test]
    #[serial_test::serial(micro_sp_activity_log_env)]
    fn nonsense_tunables_fall_back_to_the_defaults() {
        let _guard = EnvGuard::clear(&[
            "MICRO_SP_ACTIVITY_LOG",
            "MICRO_SP_ACTIVITY_LOG_DIR",
            "MICRO_SP_ACTIVITY_LOG_MAX_MB",
            "MICRO_SP_ACTIVITY_LOG_MAX_FILES",
            "MICRO_SP_ACTIVITY_LOG_SKIP",
        ]);

        unsafe { std::env::set_var("MICRO_SP_ACTIVITY_LOG", "yes") };
        unsafe { std::env::set_var("MICRO_SP_ACTIVITY_LOG_MAX_MB", "not-a-number") };
        let config = ActivityLogConfig::from_env().unwrap();
        assert_eq!(config.max_bytes, DEFAULT_MAX_BYTES);

        // Zero would mean "rotate before every record"; that is a foot-gun, not
        // a request.
        unsafe { std::env::set_var("MICRO_SP_ACTIVITY_LOG_MAX_MB", "0") };
        let config = ActivityLogConfig::from_env().unwrap();
        assert_eq!(config.max_bytes, DEFAULT_MAX_BYTES);
    }

    // -- the installed log, end to end ------------------------------------

    /// Everything above tests the pieces. This tests the assembled thing: the
    /// process-wide install, the background thread, the queue, the flush
    /// handshake and the filter, by running them for real and reading the file
    /// back off disk.
    ///
    /// It has to happen in a child process. `init` installs into a `OnceLock`,
    /// so the first test in a binary to call it would win and every other test
    /// would silently exercise nothing - and the whole point here is to check
    /// the path that only the *first* caller takes. `info_logger` re-executes
    /// itself for the same reason; this follows that pattern.
    #[test]
    fn an_installed_log_writes_every_kind_of_event_to_disk() {
        const CHILD_DIR: &str = "MICRO_SP_ACTIVITY_LOG_CHILD_DIR";

        if let Ok(dir) = std::env::var(CHILD_DIR) {
            // ---- child: install for real and emit one of each kind ----
            let config = ActivityLogConfig {
                dir: PathBuf::from(&dir),
                ..Default::default()
            };
            assert!(init(config), "the first install in a process must succeed");
            assert!(is_enabled());
            assert!(!init(ActivityLogConfig::default()), "installing twice is refused");

            log_operation(
                "sp_operation_runner",
                "op_move_to_b_Xtc2ckM0IB",
                "Initial",
                "Executing",
                "Starting",
            );
            log_transition("sp_auto_transition_runner", "t_open_gripper", "t_open_gripper_aB3");
            log_sop("sp_sop_runner", "sop_assembly_7fKd", "Initial", "Executing", "");

            let old = state_of(&[
                ("robot_pose", "home".to_spvalue()),
                ("op_move_elapsed_executing_ms", 100.to_spvalue()),
            ]);
            let delta = state_of(&[
                ("robot_pose", "at_b".to_spvalue()),
                ("gripper_closed", true.to_spvalue()),
                // Must be filtered out: it changes on every tick.
                ("op_move_elapsed_executing_ms", 200.to_spvalue()),
            ]);
            log_state_diff("sp_sop_runner", &old, &delta);

            assert!(flush(), "flush must confirm the events reached the file");
            return;
        }

        // ---- parent ----
        let dir = temp_dir("installed");
        fs::create_dir_all(&dir).unwrap();

        let exe = std::env::current_exe().expect("test binary path");
        let output = std::process::Command::new(exe)
            .arg("utils::activity_log::tests::an_installed_log_writes_every_kind_of_event_to_disk")
            .arg("--exact")
            .arg("--nocapture")
            .env(CHILD_DIR, &dir)
            .output()
            .expect("failed to spawn the child test process");
        assert!(
            output.status.success(),
            "child failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );

        let text = fs::read_to_string(dir.join("micro_sp.log"))
            .expect("the child should have created micro_sp.log");

        // Each kind made it through the queue, the thread and the formatter.
        let line_with = |needle: &str| {
            text.lines()
                .find(|l| l.contains(needle))
                .unwrap_or_else(|| panic!("no line mentioning {needle} in:\n{text}"))
                .to_string()
        };

        let op = line_with("op_move_to_b_Xtc2ckM0IB");
        assert!(op.contains("| OP    |"), "{op}");
        assert!(op.contains("Initial -> Executing"), "{op}");
        assert!(op.contains("(Starting)"), "{op}");
        assert!(op.contains("sp_operation_runner"), "source is the runner: {op}");

        let transition = line_with("t_open_gripper");
        assert!(transition.contains("| TRANS |"), "{transition}");
        assert!(
            transition.contains("t_open_gripper_aB3"),
            "the per-firing id ties it to the variable changes: {transition}"
        );

        let sop = line_with("sop_assembly_7fKd");
        assert!(sop.contains("| SOP   |"), "{sop}");
        assert!(sop.contains("Initial -> Executing"), "{sop}");

        // A changed variable carries both sides; a brand new one says so.
        let pose = line_with("robot_pose");
        assert!(pose.contains("| VAR   |"), "{pose}");
        assert!(pose.contains("home -> at_b"), "{pose}");
        let gripper = line_with("gripper_closed");
        assert!(gripper.contains("(new) -> true"), "{gripper}");

        // The filter really applies on the live path, not just in the config.
        assert!(
            !text.contains("op_move_elapsed_executing_ms"),
            "the per-tick counter must not reach the file:\n{text}"
        );

        // Ordering is preserved: the queue is FIFO and the writer appends.
        let pos = |needle: &str| text.find(needle).unwrap();
        assert!(
            pos("op_move_to_b_Xtc2ckM0IB") < pos("t_open_gripper")
                && pos("t_open_gripper") < pos("sop_assembly_7fKd"),
            "events must land in the order they were emitted:\n{text}"
        );

        fs::remove_dir_all(&dir).ok();
    }

    // -- the writer thread, driven directly -------------------------------
    //
    // `writer_loop` is a plain synchronous function over a channel, which is
    // what makes the background thread testable without installing anything
    // process-wide: build a writer, feed it a `Receiver`, and read the file
    // back. Everything in this section drives it that way.

    /// Collect every log line in `dir`, active file and rotated files alike.
    fn all_text_in(dir: &Path) -> String {
        let mut out = String::new();
        let mut paths: Vec<PathBuf> = fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();
        paths.sort();
        for path in paths {
            out.push_str(&fs::read_to_string(&path).unwrap());
        }
        out
    }

    /// The loop's contract in one test: everything queued reaches the file, in
    /// the order it was sent, and the loop *terminates* once the last sender is
    /// gone. The termination half matters as much as the writing half - a loop
    /// that outlived its senders would keep a thread and an open file forever.
    #[test]
    fn the_writer_loop_writes_queued_records_in_order_and_then_exits() {
        let dir = temp_dir("loop_order");
        let writer = ActivityWriter::new(config_in(&dir)).unwrap();
        let path = writer.current_path();

        let (tx, rx) = sync_channel::<Msg>(16);
        for name in ["op_first", "op_second", "op_third"] {
            tx.send(Msg::Record(Box::new(record(
                ActivityKind::Operation,
                name,
                "Initial -> Executing",
            ))))
            .unwrap();
        }
        // Dropping the last sender is the only shutdown signal there is.
        drop(tx);

        // If this returned, the loop noticed the disconnect. If it did not, the
        // test would hang rather than fail, which is the honest outcome for a
        // loop that never exits.
        writer_loop(writer, rx);

        let text = fs::read_to_string(&path).unwrap();
        let first = text.find("op_first").expect("op_first was never written");
        let second = text.find("op_second").expect("op_second was never written");
        let third = text.find("op_third").expect("op_third was never written");
        assert!(first < second && second < third, "FIFO order:\n{text}");
        // The loop flushes before returning, so nothing is left in the
        // `BufWriter` when the thread ends.
        assert!(text.ends_with('\n'));

        fs::remove_dir_all(&dir).ok();
    }

    /// The batching path: one blocking `recv` followed by a greedy `try_recv`
    /// drain. With the whole burst already queued before the loop starts, the
    /// first `recv` takes one record and the inner drain has to pick up the
    /// other ninety-nine - if the drain were wrong, the loop would still write
    /// them, but this pins that none are lost or reordered by the batching.
    #[test]
    fn a_whole_burst_queued_at_once_is_drained_in_one_batch() {
        let dir = temp_dir("loop_batch");
        let writer = ActivityWriter::new(config_in(&dir)).unwrap();
        let path = writer.current_path();

        let (tx, rx) = sync_channel::<Msg>(256);
        for i in 0..100 {
            tx.send(Msg::Record(Box::new(record(
                ActivityKind::Variable,
                &format!("burst_{i:03}"),
                "a -> b",
            ))))
            .unwrap();
        }
        drop(tx);
        writer_loop(writer, rx);

        let text = fs::read_to_string(&path).unwrap();
        for i in 0..100 {
            assert!(
                text.contains(&format!("burst_{i:03} ")),
                "burst_{i:03} was dropped by the batch drain"
            );
        }
        let mut previous = 0;
        for i in 0..100 {
            let at = text.find(&format!("burst_{i:03} ")).unwrap();
            assert!(at >= previous, "batching must not reorder records");
            previous = at;
        }

        fs::remove_dir_all(&dir).ok();
    }

    /// The flush handshake is what makes [`flush`] meaningful: when the ack
    /// arrives, everything sent *before* the flush is already on disk. This
    /// reads the file while the writer thread is still alive and still holds the
    /// channel open, so a passing assertion can only come from the handshake and
    /// not from the loop's final flush on shutdown.
    #[test]
    fn a_flush_message_is_acked_only_after_the_queue_reached_the_file() {
        let dir = temp_dir("loop_flush");
        let writer = ActivityWriter::new(config_in(&dir)).unwrap();
        let path = writer.current_path();

        let (tx, rx) = sync_channel::<Msg>(16);
        let thread = std::thread::spawn(move || writer_loop(writer, rx));

        tx.send(Msg::Record(Box::new(record(
            ActivityKind::Sop,
            "sop_before_flush",
            "Initial -> Executing",
        ))))
        .unwrap();
        let (ack_tx, ack_rx) = sync_channel::<()>(1);
        tx.send(Msg::Flush(ack_tx)).unwrap();
        ack_rx.recv().expect("the writer must answer the handshake");

        let text = fs::read_to_string(&path).unwrap();
        assert!(
            text.contains("sop_before_flush"),
            "the ack promised the file was current:\n{text}"
        );

        drop(tx);
        thread.join().unwrap();
        fs::remove_dir_all(&dir).ok();
    }

    /// A gap in the file must never be silent: once events have been dropped,
    /// the writer records how many as soon as it catches up. And it must report
    /// each gap exactly once - a notice repeated on every subsequent batch would
    /// itself become the noise it is warning about.
    #[test]
    #[serial_test::serial(micro_sp_activity_log_global)]
    fn a_dropped_event_gap_is_reported_once_and_not_repeated() {
        let dir = temp_dir("loop_dropped");
        let writer = ActivityWriter::new(config_in(&dir)).unwrap();
        let path = writer.current_path();

        // Stand in for a full queue: `emit` bumps exactly this counter when
        // `try_send` fails, and the writer reads it as an absolute total.
        DROPPED.fetch_add(3, Ordering::Relaxed);

        let (tx, rx) = sync_channel::<Msg>(16);
        let thread = std::thread::spawn(move || writer_loop(writer, rx));

        let flush = |tx: &SyncSender<Msg>| {
            let (ack_tx, ack_rx) = sync_channel::<()>(1);
            tx.send(Msg::Flush(ack_tx)).unwrap();
            ack_rx.recv().unwrap();
        };

        tx.send(Msg::Record(Box::new(record(
            ActivityKind::Operation,
            "op_after_the_gap",
            "x -> y",
        ))))
        .unwrap();
        flush(&tx);

        let after_first = fs::read_to_string(&path).unwrap();
        assert_eq!(
            after_first.matches("events dropped (queue full)").count(),
            1,
            "the gap must be recorded next to the events around it:\n{after_first}"
        );

        // A second batch with no new drops must stay quiet.
        tx.send(Msg::Record(Box::new(record(
            ActivityKind::Operation,
            "op_later_still",
            "x -> y",
        ))))
        .unwrap();
        flush(&tx);

        let after_second = fs::read_to_string(&path).unwrap();
        assert!(after_second.contains("op_later_still"), "{after_second}");
        assert_eq!(
            after_second.matches("events dropped (queue full)").count(),
            1,
            "the same gap must not be re-reported on every later batch:\n{after_second}"
        );

        drop(tx);
        thread.join().unwrap();
        fs::remove_dir_all(&dir).ok();
    }

    /// Rotation has to work from inside the writer thread too, not only when a
    /// test drives the writer by hand - a long-running process rotates while the
    /// loop is draining, and nothing may be lost across the boundary.
    #[test]
    fn the_writer_loop_rotates_mid_stream_without_losing_records() {
        let dir = temp_dir("loop_rotate");
        let config = ActivityLogConfig {
            max_bytes: 900,
            max_files: 0,
            ..config_in(&dir)
        };
        let writer = ActivityWriter::new(config).unwrap();
        let active = writer.current_path();

        let (tx, rx) = sync_channel::<Msg>(512);
        for i in 0..120 {
            tx.send(Msg::Record(Box::new(record(
                ActivityKind::Variable,
                &format!("rot_{i:03}"),
                "a -> b",
            ))))
            .unwrap();
        }
        drop(tx);
        writer_loop(writer, rx);

        let files: Vec<PathBuf> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();
        assert!(
            files.len() > 1,
            "120 records over a 900 byte budget must have rotated"
        );
        assert!(
            fs::metadata(&active).unwrap().len() <= 900,
            "the active file is over its budget"
        );

        let text = all_text_in(&dir);
        for i in 0..120 {
            assert!(
                text.contains(&format!("rot_{i:03} ")),
                "rot_{i:03} vanished across a rotation inside the writer thread"
            );
        }

        fs::remove_dir_all(&dir).ok();
    }

    // -- the installed log, in this process -------------------------------

    /// The emission helpers in one go, against a real installed log.
    ///
    /// This is the only test in the binary that calls [`init`] in-process: the
    /// handle is a `OnceLock`, so whoever calls it first wins for the whole
    /// binary. Everything downstream of that install is checked here - the
    /// filter recorded by `install_filter`, the `try_send` in `emit`, the
    /// per-kind `detail` shapes, and the flush handshake - by reading the bytes
    /// the writer thread actually put on disk.
    #[test]
    #[serial_test::serial(micro_sp_activity_log_global)]
    fn the_installed_log_renders_every_emission_helper_onto_disk() {
        let dir = temp_dir("inprocess");
        let config = ActivityLogConfig {
            // A short value limit, so `filter_settings` demonstrably feeds the
            // emission path rather than the default being used by accident.
            max_value_len: 12,
            skip_suffixes: vec!["_elapsed_executing_ms".to_string(), "_secret".to_string()],
            ..config_in(&dir)
        };
        assert!(init(config), "the first install in this process must win");
        assert!(is_enabled(), "emission sites now have work to do");
        assert!(
            !init(ActivityLogConfig::default()),
            "a second install is refused, so the first one's directory stands"
        );

        let dropped_before = dropped_count();

        // An operation and a SOP: the note is parenthesised when there is one
        // and omitted entirely when there is not.
        log_operation("runner_a", "op_with_note", "Initial", "Executing", "Starting");
        log_operation("runner_a", "op_without_note", "Executing", "Completed", "");
        log_sop("runner_b", "sop_with_note", "Initial", "Executing", "Step 1/3");
        log_sop("runner_b", "sop_without_note", "Executing", "Completed", "");
        log_transition("runner_c", "t_open", "t_open_aB3");
        log_variable("runner_d", "changed_var", Some(&"home".to_spvalue()), &"at_b".to_spvalue());
        log_variable("runner_d", "brand_new_var", None, &7.to_spvalue());
        log_variable(
            "runner_d",
            "direct_long_var",
            None,
            &"0123456789abcdefghij".to_spvalue(),
        );

        let old = state_of(&[("diff_pose", "home".to_spvalue())]);
        let delta = state_of(&[
            ("diff_pose", "at_b".to_spvalue()),
            ("diff_added", true.to_spvalue()),
            ("diff_long", "0123456789abcdefghij".to_spvalue()),
            // Both of these match the installed skip list and must not appear.
            ("op_x_elapsed_executing_ms", 200.to_spvalue()),
            ("thing_secret", "hunter2".to_spvalue()),
        ]);
        log_state_diff("runner_e", &old, &delta);

        assert!(flush(), "flush must confirm the queue reached the file");
        assert_eq!(
            dropped_count(),
            dropped_before,
            "an 8192-deep queue must not drop a dozen events"
        );

        let text = fs::read_to_string(dir.join("micro_sp.log")).unwrap();
        let line = |needle: &str| {
            text.lines()
                .find(|l| l.contains(needle))
                .unwrap_or_else(|| panic!("no line mentioning {needle} in:\n{text}"))
                .to_string()
        };

        let with_note = line("op_with_note");
        assert!(with_note.contains("| OP    |"), "{with_note}");
        assert!(with_note.contains("Initial -> Executing  (Starting)"), "{with_note}");
        let without_note = line("op_without_note");
        assert!(
            without_note.ends_with("Executing -> Completed"),
            "an empty note leaves no empty parentheses behind: {without_note}"
        );

        assert!(line("sop_with_note").contains("Initial -> Executing  (Step 1/3)"));
        assert!(line("sop_without_note").ends_with("Executing -> Completed"));

        let transition = line("t_open ");
        assert!(transition.contains("| TRANS |"), "{transition}");
        assert!(
            transition.contains("taken as 't_open_aB3'"),
            "the per-firing id ties the transition to its variable changes: {transition}"
        );

        assert!(line("changed_var").contains("home -> at_b"));
        assert!(
            line("brand_new_var").contains("(new) -> 7"),
            "a variable with no previous value is marked new, not given a fake old one"
        );
        assert!(
            line("direct_long_var").contains("(new) -> 0123456789abcdefghij"),
            "a direct log_variable call renders at the default limit, not the \
             installed one: {}",
            line("direct_long_var")
        );

        assert!(line("diff_pose").contains("home -> at_b"));
        assert!(line("diff_added").contains("(new) -> true"));
        assert!(
            line("diff_long").contains("(new) -> 0123456789a…"),
            "the diff path renders through the installed max_value_len: {}",
            line("diff_long")
        );
        assert!(
            !text.contains("op_x_elapsed_executing_ms") && !text.contains("thing_secret"),
            "the installed skip list must filter the live diff:\n{text}"
        );

        // The writer thread outlives this test and keeps the file open, so the
        // directory is left in place rather than pulled out from under it.
    }

    /// Restores the environment when it drops, so these tests cannot leak into
    /// the rest of the binary (env vars are process-global).
    struct EnvGuard {
        saved: Vec<(String, Option<String>)>,
    }

    impl EnvGuard {
        fn clear(keys: &[&str]) -> Self {
            let saved = keys
                .iter()
                .map(|k| (k.to_string(), std::env::var(k).ok()))
                .collect();
            for key in keys {
                unsafe { std::env::remove_var(key) };
            }
            Self { saved }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in &self.saved {
                match value {
                    Some(v) => unsafe { std::env::set_var(key, v) },
                    None => unsafe { std::env::remove_var(key) },
                }
            }
        }
    }
}
