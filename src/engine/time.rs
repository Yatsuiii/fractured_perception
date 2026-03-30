use std::time::Instant;

pub struct Time {
    last_frame: Instant,
    pub delta: f32,
    pub elapsed: f32,
}

impl Time {
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            delta: 0.0,
            elapsed: 0.0,
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        self.delta = now.duration_since(self.last_frame).as_secs_f32();
        self.elapsed += self.delta;
        self.last_frame = now;
    }
}
