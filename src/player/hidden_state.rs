/// Silently tracked per player throughout every session.
/// Accumulated from every choice, interaction, and moment of trust or doubt.
#[derive(Debug, Clone, Default)]
pub struct HiddenState {
    /// Chose honesty, verified before acting, trusted carefully.
    pub truth: f32,
    /// Acted on incomplete info, communication broke down.
    pub chaos: f32,
    /// Accepted false signals as real, built certainty on lies.
    pub illusion: f32,
    /// Held contradictions, never fully resolved.
    pub balance: f32,
}

impl HiddenState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_truth(&mut self, v: f32)   { self.truth   = (self.truth   + v).clamp(0.0, 99.9); }
    pub fn add_chaos(&mut self, v: f32)   { self.chaos   = (self.chaos   + v).clamp(0.0, 99.9); }
    pub fn add_illusion(&mut self, v: f32){ self.illusion = (self.illusion + v).clamp(0.0, 99.9); }
    pub fn add_balance(&mut self, v: f32) { self.balance  = (self.balance  + v).clamp(0.0, 99.9); }

    pub fn dominant(&self) -> &'static str {
        let vals = [
            ("Truth",   self.truth),
            ("Chaos",   self.chaos),
            ("Illusion",self.illusion),
            ("Balance", self.balance),
        ];
        vals.iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, _)| *name)
            .unwrap_or("Balance")
    }

    /// Normalised 0.0–1.0 bar value for display (caps at 10.0 for the bar).
    pub fn bar(&self, v: f32) -> f32 {
        (v / 10.0).clamp(0.0, 1.0)
    }
}
