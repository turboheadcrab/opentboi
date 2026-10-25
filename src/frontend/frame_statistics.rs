//! Engine-independent frame timing, so every frontend reports comparable numbers.

use std::time::{Duration, Instant};

const MILLISECONDS_PER_SECOND: f64 = 1000.0;
const PERCENTILE_99: f64 = 0.99;

pub struct FrameStatistics {
    frontend_name: &'static str,
    process_started: Instant,
    time_to_first_frame: Option<Duration>,
    previous_frame: Option<Instant>,
    frame_durations: Vec<Duration>,
}

impl FrameStatistics {
    pub fn new(frontend_name: &'static str, process_started: Instant) -> Self {
        Self {
            frontend_name,
            process_started,
            time_to_first_frame: None,
            previous_frame: None,
            frame_durations: Vec::new(),
        }
    }

    /// Call once per finished frame (after presenting it).
    pub fn record_frame(&mut self) {
        let now = Instant::now();
        match self.previous_frame {
            Some(previous) => self.frame_durations.push(now - previous),
            None => self.time_to_first_frame = Some(now - self.process_started),
        }
        self.previous_frame = Some(now);
    }

    /// For pipelined renderers, where a frame is presented after the game loop moved on: replaces
    /// the time to first frame with when it actually reached the screen.
    #[cfg_attr(
        not(feature = "bevy-frontend"),
        expect(dead_code, reason = "only Bevy pipelines rendering")
    )]
    pub fn record_first_present(&mut self, presented_at: Instant) {
        self.time_to_first_frame = Some(presented_at - self.process_started);
    }

    pub fn has_run_for(&self, duration: Duration) -> bool {
        self.process_started.elapsed() >= duration
    }

    /// One line a benchmark script can parse.
    pub fn report(&self) -> String {
        let first_frame = self.time_to_first_frame.map_or(f64::NAN, to_milliseconds);
        let mut sorted = self.frame_durations.clone();
        sorted.sort_unstable();
        let frame_count = sorted.len();
        let Some(&slowest) = sorted.last() else {
            return format!("{}: first frame after {first_frame:.0} ms", self.frontend_name);
        };
        let total: Duration = sorted.iter().sum();
        let mean = to_milliseconds(total) / frame_count as f64;
        let percentile_index =
            crate::whole_numbers::clamped_usize(frame_count as f64 * PERCENTILE_99).min(frame_count - 1);
        format!(
            "{}: first frame after {first_frame:.0} ms; {frame_count} frames, mean {mean:.2} ms ({:.0} fps), \
             median {:.2} ms, 99th percentile {:.2} ms, slowest {:.2} ms",
            self.frontend_name,
            MILLISECONDS_PER_SECOND / mean,
            to_milliseconds(sorted[frame_count / 2]),
            to_milliseconds(sorted[percentile_index]),
            to_milliseconds(slowest),
        )
    }
}

fn to_milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * MILLISECONDS_PER_SECOND
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_summarizes_recorded_frames() {
        let mut statistics = FrameStatistics::new("test", Instant::now());
        for _ in 0..3 {
            statistics.record_frame();
        }
        let report = statistics.report();
        assert!(report.starts_with("test: first frame after"), "{report}");
        assert!(report.contains("2 frames"), "{report}");
    }
}
