use std::time::{Duration, Instant};

pub struct FpsCounter {
    count: u32,
    last_reset: Instant,
}

impl Default for FpsCounter {
    fn default() -> Self {
        FpsCounter {
            count: 0,
            last_reset: Instant::now(),
        }
    }
}

#[derive(Debug)]
pub struct Stats {
    fps: u32,
    time_per_frame: Duration,
}

impl FpsCounter {
    pub fn on_frame(&mut self) -> Option<Stats> {
        self.count += 1;

        if self.last_reset.elapsed() >= Duration::SECOND {
            let stats = Stats {
                fps: self.count,
                time_per_frame: Duration::from_nanos((Duration::SECOND.as_nanos() / self.count as u128) as u64),
            };

            self.count = 0;
            self.last_reset = Instant::now();

            Some(stats)
        } else {
            None
        }
    }
}
