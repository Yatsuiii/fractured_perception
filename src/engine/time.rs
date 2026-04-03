use std::time::Instant;

pub const FIXED_TIME_STEP: f32 = 1.0 / 60.0;

pub struct Time {
    last_frame: Instant,
    pub delta: f32,
    pub elapsed: f32,
    accumulator: f32,
}

impl Time {
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            delta: 0.0,
            elapsed: 0.0,
            accumulator: 0.0,
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        self.delta = now.duration_since(self.last_frame).as_secs_f32();
        self.elapsed += self.delta;
        self.accumulator += self.delta;
        self.last_frame = now;
    }

    pub fn consume_fixed_step(&mut self) -> bool {
        if self.accumulator >= FIXED_TIME_STEP {
            self.accumulator -= FIXED_TIME_STEP;
            true
        } else {
            false
        }
    }
}
