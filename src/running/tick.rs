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
//! one thousand, so every runner is hitting its period. That is why the
//! defaults are 1 ms.
//!
//! 1 ms is also the practical floor for a timer-driven loop: tokio's timer
//! wheel has millisecond granularity, so a shorter period would not fire any
//! sooner. Going faster than this needs a different design - waiting on Redis
//! keyspace notifications rather than polling - not a smaller number.
//!
//! What 1 ms costs, so the trade is explicit:
//!   * ~25% of a core idle and ~44% under load, for *this* process. On a
//!     four-core controller that is a tenth of the machine.
//!   * ~3.8% of the single Redis thread per process. Several micro_sp processes
//!     against one Redis multiply that.
//!   * Write churn tracks the rate: three running sleep timers produce ~790
//!     MSET/s at 1 ms against ~10/s at 100 ms, because the elapsed counters are
//!     written every tick while something is running. It is still only 0.05% of
//!     a Redis core, but it is not nothing.
//!
//! If any of that is the wrong trade for a deployment, every runner's period is
//! overridable at runtime - 5 ms keeps most of the latency win (175 ms for the
//! same SOP) at a third of the CPU.

use tokio::time::{Duration, Interval, MissedTickBehavior, interval};

/// Refuse to spin faster than this regardless of what the environment says.
///
/// At a period this short the loop is effectively "read Redis as fast as the
/// socket allows"; anything lower buys no latency but does burn a core.
pub const MIN_TICK_INTERVAL_MS: u64 = 1;

/// Build a runner's tick interval, honouring `env_var` if it is set to a
/// positive integer number of milliseconds.
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
pub fn runner_interval(env_var: &str, default_ms: u64) -> Interval {
    let configured = std::env::var(env_var)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok());

    let period_ms = match configured {
        Some(ms) if ms >= MIN_TICK_INTERVAL_MS => ms,
        Some(ms) => {
            log::warn!(
                target: "micro_sp_tick",
                "{env_var}={ms} is below the {MIN_TICK_INTERVAL_MS} ms floor; using {MIN_TICK_INTERVAL_MS} ms."
            );
            MIN_TICK_INTERVAL_MS
        }
        None => default_ms,
    };

    if configured.is_some() {
        log::info!(target: "micro_sp_tick", "{env_var} set: ticking every {period_ms} ms.");
    }

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
}
