/// Threshold Events — world mutations triggered when T/C/I/B stats
/// cross specific values.
///
/// Each threshold fires exactly once per playthrough. Effects are either
/// one-shot (applied immediately) or ongoing (checked each engine tick).
///
/// Tier 1 triggers at 2.0, Tier 2 at 5.0.

use std::collections::HashSet;

use crate::player::hidden_state::HiddenState;

// ---------------------------------------------------------------------------
// Threshold identifiers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Threshold {
    TruthTier1,
    TruthTier2,
    ChaosTier1,
    ChaosTier2,
    IllusionTier1,
    IllusionTier2,
    BalanceTier1,
    BalanceTier2,
}

const TIER_1: f32 = 2.0;
const TIER_2: f32 = 5.0;

impl Threshold {
    pub fn name(self) -> &'static str {
        match self {
            Threshold::TruthTier1    => "Clarity Pulse",
            Threshold::TruthTier2    => "Piercing Sight",
            Threshold::ChaosTier1    => "False Signal",
            Threshold::ChaosTier2    => "Eroding Trust",
            Threshold::IllusionTier1 => "Temporal Drift",
            Threshold::IllusionTier2 => "Fracture",
            Threshold::BalanceTier1  => "Still Point",
            Threshold::BalanceTier2  => "Equilibrium",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Threshold::TruthTier1    => "NPC trust boosted",
            Threshold::TruthTier2    => "Vision expands beyond the veil",
            Threshold::ChaosTier1    => "A false encounter materialises",
            Threshold::ChaosTier2    => "Trust erodes over time",
            Threshold::IllusionTier1 => "The Delayed falls further behind",
            Threshold::IllusionTier2 => "Reality distorts further",
            Threshold::BalanceTier1  => "Extremes are dampened",
            Threshold::BalanceTier2  => "Stats converge toward equilibrium",
        }
    }

    /// True if this threshold's effect is checked every tick (ongoing).
    /// False if the effect fires once and is done.
    pub fn is_ongoing(self) -> bool {
        matches!(self, Threshold::ChaosTier2 | Threshold::BalanceTier1)
    }
}

// ---------------------------------------------------------------------------
// Tracker
// ---------------------------------------------------------------------------

pub struct ThresholdTracker {
    fired: HashSet<Threshold>,
}

impl ThresholdTracker {
    pub fn new() -> Self {
        Self { fired: HashSet::new() }
    }

    /// Check the current hidden state and return any newly crossed thresholds.
    pub fn check(&mut self, state: &HiddenState) -> Vec<Threshold> {
        let candidates = [
            (Threshold::TruthTier1,    state.truth,    TIER_1),
            (Threshold::TruthTier2,    state.truth,    TIER_2),
            (Threshold::ChaosTier1,    state.chaos,    TIER_1),
            (Threshold::ChaosTier2,    state.chaos,    TIER_2),
            (Threshold::IllusionTier1, state.illusion,  TIER_1),
            (Threshold::IllusionTier2, state.illusion,  TIER_2),
            (Threshold::BalanceTier1,  state.balance,   TIER_1),
            (Threshold::BalanceTier2,  state.balance,   TIER_2),
        ];

        let mut newly_crossed = Vec::new();
        for (threshold, value, required) in candidates {
            if value >= required && self.fired.insert(threshold) {
                newly_crossed.push(threshold);
            }
        }
        newly_crossed
    }

    /// Whether this threshold has already fired.
    pub fn is_active(&self, threshold: Threshold) -> bool {
        self.fired.contains(&threshold)
    }

    /// Current FOV bonus from truth thresholds.
    pub fn fov_bonus(&self) -> f32 {
        if self.is_active(Threshold::TruthTier2) { 3.0 } else { 0.0 }
    }

    /// Current delay addition from illusion thresholds.
    pub fn delay_penalty(&self) -> f32 {
        if self.is_active(Threshold::IllusionTier1) { 1.5 } else { 0.0 }
    }

    /// Multiplier for encounter stat rewards (Balance T1 halves them).
    pub fn stat_gain_multiplier(&self) -> f32 {
        if self.is_active(Threshold::BalanceTier1) { 0.5 } else { 1.0 }
    }

    /// Whether distortion percentage should be doubled (Illusion T2).
    pub fn double_distortion(&self) -> bool {
        self.is_active(Threshold::IllusionTier2)
    }

    pub fn clear(&mut self) {
        self.fired.clear();
    }
}
