use std::time::{Duration, Instant};

pub struct FpsCounter {
    frames: u32,
    since: Instant,
}

impl Default for FpsCounter {
    fn default() -> Self {
        FpsCounter {
            frames: 0,
            since: Instant::now(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Stats {
    fps: u32,
    time_per_frame: Duration,
}

impl FpsCounter {
    pub fn on_frame(&mut self) -> Option<Stats> {
        self.frames += 1;

        if self.since.elapsed() >= Duration::SECOND {
            let stats = Stats {
                fps: self.frames,
                time_per_frame: Duration::from_nanos((Duration::SECOND.as_nanos() / self.frames as u128) as u64),
            };

            self.frames = 0;
            self.since = Instant::now();

            Some(stats)
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct Delta {
    last_frame: Option<Instant>
}

impl Delta {
    pub fn on_frame(&mut self) -> Option<Duration> {
        let now = Instant::now();

        let out = match &self.last_frame {
            Some(last_frame) => Some(now - *last_frame),
            None => None,
        };

        self.last_frame = Some(now);

        out
    }
}
