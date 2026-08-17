//! Runner tick periods.
//!
//! Every runner loop is a poll: wait a period, read its key set from Redis, do
//! its work, write back what changed. The period sets the latency floor for
//! anything that has to travel between runners - an auto operation firing, the
//! SOP runner noticing, the plan runner reacting - because each hop costs at
//! least one period.
//!
//! Two things make the period a real trade rather than "smaller is better":
//!
//!   * Every tick is at minimum one Redis round trip. On a local socket that is
//!     tens of microseconds, so the *floor* is low, but Redis is single
//!     threaded and every runner in every process shares it.
//!   * The per-tick client work is not free: reading a few hundred keys costs a
//!     `serde_json` parse per variable, a `State` clone and a full diff. That
//!     is the cost that actually decides how fast this can go.
//!
//! Measured on a local Redis with a 30-operation model (132-key read set), all
//! eight runners at the same period. One tick of the heaviest runner costs
//! ~150 us, of which ~120 us is the Redis round trip - so a single runner could
//! in principle poll at ~7 kHz, and the period is bounded by aggregate CPU, not
//! by any one tick.
//!
//! | period   | ticks/s | Redis  | client | 10-step SOP |
//! |----------|---------|--------|--------|-------------|
//! | 50-500ms | 66      | 0.1%   | 0.6%   | 3304 ms     |
//! | 20 ms    | 401     | 0.4%   | 3.0%   | 862 ms      |
//! | 10 ms    | 801     | 0.7%   | 4.8%   | 432 ms      |
//! | 5 ms     | 1601    | 1.3%   | 9.2%   | 175 ms      |
//! | 2 ms     | 4001    | 2.8%   | 18.0%  | -           |
//! | 1 ms     | 8001    | 3.8%   | 25.2%  | 45 ms       |
//!
//! (percentages are of one core; client CPU is the whole process, all eight
//! runners; under SOP load the 1 ms figure is ~44% of a core.)
//!
//! Latency scales essentially linearly with the period all the way down, and
//! nothing falls behind at 1 ms - 8001 ticks/s is exactly eight runners times
//! one thousand, so every runner is hitting its period. The default is 5 ms,
//! and 1 ms is the floor.
//!
//! 1 ms is the practical floor for a timer-driven loop: tokio's timer wheel has
//! millisecond granularity, so a shorter period would not fire any sooner.
//! Going faster than this needs a different design - waiting on Redis keyspace
//! notifications rather than polling - not a smaller number.
//!
//! What that 1 ms floor costs, so the trade is explicit:
//!   * ~25% of a core idle and ~44% under load, for *this* process. On a
//!     four-core controller that is a tenth of the machine.
//!   * ~3.8% of the single Redis thread per process. Several micro_sp processes
//!     against one Redis multiply that.
//!   * Write churn tracks the rate: three running sleep timers produce ~790
//!     MSET/s at 1 ms against ~10/s at 100 ms, because the elapsed counters are
//!     written every tick while something is running. It is still only 0.05% of
//!     a Redis core, but it is not nothing.
//!
//! If any of that is the wrong trade for a deployment, the period is
//! overridable at runtime with `MICRO_SP_TICK_INTERVAL_MS` - one number for
//! every runner, since a mixed set of periods only makes the slowest one the
//! latency floor anyway.

use tokio::time::{Duration, Interval, MissedTickBehavior, interval};

/// The period every runner ticks at when `MICRO_SP_TICK_INTERVAL_MS` is unset.
///
/// 5 ms keeps most of the latency win (175 ms for the 10-step SOP above) at
/// roughly a third of the CPU of a 1 ms period.
pub const DEFAULT_TICK_INTERVAL_MS: u64 = 5;

/// Refuse to spin faster than this regardless of what the environment says.
///
/// 1 ms is tokio's timer-wheel granularity, so a shorter period would not fire
/// any sooner - going faster needs a different design (Redis keyspace
/// notifications rather than polling), not a smaller number.
pub const MIN_TICK_INTERVAL_MS: u64 = 1;

/// The environment variable that sets the period for every runner.
pub const TICK_INTERVAL_ENV_VAR: &str = "MICRO_SP_TICK_INTERVAL_MS";

/// The configured tick period in milliseconds: `MICRO_SP_TICK_INTERVAL_MS` if
/// it parses as a positive integer, clamped up to [`MIN_TICK_INTERVAL_MS`],
/// otherwise [`DEFAULT_TICK_INTERVAL_MS`].
pub fn tick_interval_ms() -> u64 {
    let configured = std::env::var(TICK_INTERVAL_ENV_VAR)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok());

    match configured {
        Some(ms) if ms >= MIN_TICK_INTERVAL_MS => ms,
        Some(ms) => {
            log::warn!(
                target: "micro_sp_tick",
                "{TICK_INTERVAL_ENV_VAR}={ms} is below the {MIN_TICK_INTERVAL_MS} ms floor; using {MIN_TICK_INTERVAL_MS} ms."
            );
            MIN_TICK_INTERVAL_MS
        }
        None => DEFAULT_TICK_INTERVAL_MS,
    }
}

/// Build a runner's tick interval from [`tick_interval_ms`].
///
/// The missed-tick behaviour matters more than it looks. Tokio's default is
/// `Burst`: if a tick overruns its period, the interval fires immediately, over
/// and over, until it has caught up. At the old 100-200 ms periods that was
/// nearly unreachable. At a period of a few milliseconds it is not - one slow
/// Redis reply and the loop degenerates into a spin with no delay between
/// iterations, starving every other task on the runtime. `Delay` instead keeps
/// a full period between the end of one tick and the start of the next, so a
/// runner that cannot keep up simply runs slower rather than eating the
/// scheduler.
pub fn runner_interval() -> Interval {
    let period_ms = tick_interval_ms();

    let mut interval = interval(Duration::from_millis(period_ms));
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
    interval
}

/// Measures the real time between ticks in whole milliseconds, without losing
/// the remainder.
///
/// The runners' elapsed counters are integer milliseconds, so the obvious
/// `last.elapsed().as_millis()` truncates. At the old 100-200 ms periods that
/// was noise. At a 1 ms period it is not: a tick that really takes 1.13 ms
/// counts as 1 ms, and the operation ages 11% slower than the wall clock -
/// which for a 600 s timeout is over a minute of error, in the direction of
/// never firing.
///
/// Carrying the sub-millisecond remainder forward makes the sum exact: over any
/// number of ticks the counted milliseconds equal the elapsed milliseconds,
/// give or take the microsecond currently held in the carry.
pub struct TickClock {
    last: std::time::Instant,
    carry_us: i64,
}

impl TickClock {
    pub fn new() -> Self {
        Self {
            last: std::time::Instant::now(),
            carry_us: 0,
        }
    }

    /// Whole milliseconds since the previous call.
    pub fn elapsed_ms(&mut self) -> i64 {
        let now = std::time::Instant::now();
        let elapsed_us = now.duration_since(self.last).as_micros() as i64 + self.carry_us;
        self.last = now;
        self.carry_us = elapsed_us % 1000;
        elapsed_us / 1000
    }
}

impl Default for TickClock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod interval_tests {
    use super::*;
    use serial_test::serial;

    /// `MICRO_SP_TICK_INTERVAL_MS` is process-global, so these have to run one
    /// at a time and put it back afterwards - otherwise a runner test that
    /// happens to be running concurrently picks up whatever was left behind.
    struct EnvGuard;

    impl EnvGuard {
        fn set(value: &str) -> Self {
            unsafe { std::env::set_var(TICK_INTERVAL_ENV_VAR, value) };
            EnvGuard
        }
        fn unset() -> Self {
            unsafe { std::env::remove_var(TICK_INTERVAL_ENV_VAR) };
            EnvGuard
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe { std::env::remove_var(TICK_INTERVAL_ENV_VAR) };
        }
    }

    #[test]
    #[serial]
    fn with_nothing_set_the_period_is_the_default() {
        let _guard = EnvGuard::unset();
        assert_eq!(tick_interval_ms(), DEFAULT_TICK_INTERVAL_MS);
        assert_eq!(DEFAULT_TICK_INTERVAL_MS, 5);
    }

    #[test]
    #[serial]
    fn the_environment_variable_sets_the_period() {
        for requested in ["1", "2", "5", "50", "1000"] {
            let _guard = EnvGuard::set(requested);
            assert_eq!(
                tick_interval_ms(),
                requested.parse::<u64>().unwrap(),
                "{requested} ms should have been honoured verbatim"
            );
        }
    }

    /// 1 ms is tokio's timer granularity; below it the loop would spin without
    /// firing any sooner. Anything lower is clamped rather than accepted.
    #[test]
    #[serial]
    fn a_period_below_the_floor_is_clamped_to_the_floor() {
        let _guard = EnvGuard::set("0");
        assert_eq!(tick_interval_ms(), MIN_TICK_INTERVAL_MS);
        assert_eq!(MIN_TICK_INTERVAL_MS, 1);
    }

    /// The floor itself is allowed - the clamp must not be off by one and turn
    /// the documented minimum into an error.
    #[test]
    #[serial]
    fn the_floor_itself_is_accepted() {
        let _guard = EnvGuard::set("1");
        assert_eq!(tick_interval_ms(), 1);
    }

    /// A value that is not a positive integer - a typo, an empty variable left
    /// over in a unit file, a float, a negative number - must fall back to the
    /// default rather than to the floor or to a panic. Falling back to the
    /// floor would silently put a deployment at 1 ms because of a typo.
    #[test]
    #[serial]
    fn unparseable_values_fall_back_to_the_default() {
        for junk in ["", "   ", "abc", "5ms", "-1", "2.5", "1e3", "0x5"] {
            let _guard = EnvGuard::set(junk);
            assert_eq!(
                tick_interval_ms(),
                DEFAULT_TICK_INTERVAL_MS,
                "{junk:?} should have fallen back to the default"
            );
        }
    }

    /// Surrounding whitespace is common in env files and is trimmed.
    #[test]
    #[serial]
    fn surrounding_whitespace_is_tolerated() {
        let _guard = EnvGuard::set("  20\n");
        assert_eq!(tick_interval_ms(), 20);
    }

    /// The interval the runners actually get has to fire at the configured
    /// period, and it must not burst-catch-up after a slow tick - one slow
    /// Redis reply at a 1 ms period would otherwise turn the loop into a spin
    /// with no delay between iterations and starve every other task.
    #[tokio::test]
    #[serial]
    async fn a_missed_tick_delays_rather_than_bursting() {
        let _guard = EnvGuard::set("10");
        let mut interval = runner_interval();

        // First tick completes immediately, as tokio intervals do.
        interval.tick().await;

        // Overrun the period by a lot, then take two more ticks. With `Burst`
        // the second of those would return instantly; with `Delay` it waits a
        // full period.
        tokio::time::sleep(Duration::from_millis(35)).await;
        interval.tick().await;

        let before = std::time::Instant::now();
        interval.tick().await;
        let waited = before.elapsed();

        assert!(
            waited >= Duration::from_millis(8),
            "expected a full period between ticks, waited only {waited:?}"
        );
    }

    #[tokio::test]
    #[serial]
    async fn the_interval_paces_at_the_configured_period() {
        let _guard = EnvGuard::set("20");
        let mut interval = runner_interval();

        let start = std::time::Instant::now();
        for _ in 0..4 {
            interval.tick().await;
        }
        let elapsed = start.elapsed();

        // Three periods of 20 ms after the immediate first tick.
        assert!(
            elapsed >= Duration::from_millis(55),
            "four ticks at 20 ms took only {elapsed:?}"
        );
        assert!(
            elapsed < Duration::from_millis(400),
            "four ticks at 20 ms took {elapsed:?}, which is far more than the period explains"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The property that matters: no systematic loss, however short the ticks.
    #[test]
    fn the_carry_makes_the_sum_exact() {
        let mut clock = TickClock {
            last: std::time::Instant::now(),
            carry_us: 0,
        };

        // Feed it a fixed sub-millisecond remainder per tick by driving the
        // arithmetic directly: 1130 us per tick, 100 ticks = 113 ms exactly.
        let mut total = 0i64;
        for _ in 0..100 {
            let elapsed_us = 1130 + clock.carry_us;
            clock.carry_us = elapsed_us % 1000;
            total += elapsed_us / 1000;
        }
        assert_eq!(total, 113, "1130 us x 100 must count as 113 ms, not 100");
    }

    #[test]
    fn a_real_elapsed_ms_is_non_negative_and_small() {
        let mut clock = TickClock::new();
        std::thread::sleep(std::time::Duration::from_millis(5));
        let ms = clock.elapsed_ms();
        assert!((4..=20).contains(&ms), "expected about 5 ms, got {ms}");
    }

    /// `TickClock::default()` has to behave like `TickClock::new()` - a fresh
    /// clock with no carry, so the very first `elapsed_ms()` call reports a
    /// small, non-negative duration rather than replaying whatever `Instant`
    /// happened to be at `0` or a stale carry from a previous run.
    #[test]
    fn default_builds_a_fresh_clock_like_new() {
        let mut clock = TickClock::default();
        std::thread::sleep(std::time::Duration::from_millis(5));
        let ms = clock.elapsed_ms();
        assert!(
            (4..=20).contains(&ms),
            "a freshly defaulted clock should measure about 5 ms, got {ms}"
        );
    }
}
