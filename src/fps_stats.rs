pub struct FPSStats {
    /// The half life (in seconds) of samples
    half_life: f32,
    /// mean
    mean: f32,
    /// variance
    variance: f32,
}

impl FPSStats {
    pub fn new(half_life: f32) -> Self {
        Self {
            half_life,
            mean: 1.0 / 60.0,
            variance: 0.0,
        }
    }

    pub fn update(&mut self, frame_time: f32) {
        let alpha: f32 = 2.0_f32.powf(-frame_time / self.half_life);
        self.mean = alpha * self.mean + (1.0 - alpha) * frame_time;
        self.variance = alpha * self.variance + (1.0 - alpha) * (self.mean - frame_time).powi(2);
    }

    pub fn mean(&self) -> f32 {
        self.mean
    }

    pub fn variance(&self) -> f32 {
        self.variance
    }

    /// Standard deviation
    pub fn std(&self) -> f32 {
        self.variance.sqrt()
    }
}
