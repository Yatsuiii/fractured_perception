-- ThresholdData.lua (ModuleScript)
-- Defines T/C/I/B Tier 1 thresholds: what value triggers them and what effect fires.
-- MVP only implements Tier 1 for each stat.
-- ThresholdWatcher.server.lua checks these after every action.

local ThresholdData = {}

-- Tier 1 breakpoints — when a stat reaches this value, the effect fires once
ThresholdData.Tier1 = {
	Truth    = 2.0,
	Chaos    = 2.0,
	Illusion = 2.0,
	Balance  = 2.0,
}

-- Human-readable name and description sent to the player when a threshold fires
-- (shown as a brief notification on their screen by UIController)
ThresholdData.Notifications = {
	TruthTier1    = {
		name        = "Clarity Pulse",
		description = "Your trust with NPCs strengthens. They sense something real in you.",
	},
	ChaosTier1    = {
		name        = "False Signal",
		description = "A phantom encounter flickers into existence near you. It was never there.",
	},
	IllusionTier1 = {
		name        = "Temporal Drift",
		description = "The Delayed role falls further behind. Time is slipping.",
	},
	BalanceTier1  = {
		name        = "Still Point",
		description = "Rewards thin out. You are holding too much without resolution.",
	},
}

-- Effect parameters used by ThresholdWatcher
ThresholdData.Effects = {
	-- TruthTier1: raise all NPC trust for this player by this amount
	TruthTier1NpcTrustBonus    = 0.1,

	-- IllusionTier1: extra seconds added to the Delayed role's perception delay
	IllusionTier1DelayPenalty  = 1.5,   -- base 3.0 → becomes 4.5

	-- BalanceTier1: multiplier applied to all future encounter stat rewards
	BalanceTier1RewardMultiplier = 0.5,

	-- ChaosTier1: radius (in studs) around the player to randomly place phantom encounter
	ChaosTier1SpawnRadius      = 8,
}

return ThresholdData
