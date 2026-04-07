-- PlayerRegistry.lua (ModuleScript — server only)
-- Stores per-player data: role, hidden state (T/C/I/B), NPC trust, fired thresholds.
-- EncounterManager and ThresholdWatcher read/write entries here.

local StageData = require(game:GetService("ReplicatedStorage").Shared.StageData)

local PlayerRegistry = {}

-- Internal table keyed by Player object
local _data = {}

-- ─── INITIALISE / CLEANUP ────────────────────────────────────────────────────

-- Call this from GameManager when a player is assigned their role.
-- stageNumber is used to seed NPC trust defaults from StageData.
function PlayerRegistry.init(player, role, stageNumber)
	local stageDef = StageData[stageNumber]

	-- Build the default trust values from this stage's NPC definitions
	local npcTrust = {}
	if stageDef then
		for _, npcDef in ipairs(stageDef.npcs) do
			npcTrust[npcDef.name] = npcDef.baseTrust
		end
	end

	_data[player] = {
		role = role,

		-- Hidden state stats (T/C/I/B), all start at 0
		truth    = 0.0,
		chaos    = 0.0,
		illusion = 0.0,
		balance  = 0.0,

		-- Per-NPC trust: { ["The Watcher"] = 0.6, ... }
		npcTrust = npcTrust,

		-- Thresholds that have already fired for this player
		-- Example: { ChaosTier1 = true }
		firedThresholds = {},

		-- If BalanceTier1 fired, future encounter rewards are multiplied by this
		rewardMultiplier = 1.0,

		-- Extra delay seconds added by IllusionTier1 (0 normally, 1.5 after threshold)
		delayPenalty = 0.0,
	}
end

-- Remove data when player leaves
function PlayerRegistry.remove(player)
	_data[player] = nil
end

-- ─── GETTERS ─────────────────────────────────────────────────────────────────

function PlayerRegistry.get(player)
	return _data[player]
end

function PlayerRegistry.getRole(player)
	local entry = _data[player]
	return entry and entry.role or nil
end

function PlayerRegistry.getTrust(player, npcName)
	local entry = _data[player]
	if entry == nil then return 0.4 end
	local trust = entry.npcTrust[npcName]
	return trust ~= nil and trust or 0.4  -- default to 0.4 if no entry yet
end

-- Returns the trust tier string based on the current trust value
-- "Low" = < 0.35, "Mid" = 0.35–0.7, "High" = >= 0.7
function PlayerRegistry.getTrustTier(player, npcName)
	local trust = PlayerRegistry.getTrust(player, npcName)
	if trust >= 0.7 then
		return "High"
	elseif trust >= 0.35 then
		return "Mid"
	else
		return "Low"
	end
end

-- ─── SETTERS ─────────────────────────────────────────────────────────────────

-- Add delta to a stat, clamped between 0 and 99.9
function PlayerRegistry.addStat(player, statName, amount)
	local entry = _data[player]
	if entry == nil then return end
	entry[statName] = math.clamp(entry[statName] + amount, 0, 99.9)
end

-- Adjust NPC trust for a player, clamped between 0 and 1
function PlayerRegistry.adjustTrust(player, npcName, delta)
	local entry = _data[player]
	if entry == nil then return end
	local current = entry.npcTrust[npcName] or 0.4
	entry.npcTrust[npcName] = math.clamp(current + delta, 0.0, 1.0)
end

-- Mark a threshold as fired so it cannot fire again for this player
function PlayerRegistry.markThresholdFired(player, thresholdName)
	local entry = _data[player]
	if entry == nil then return end
	entry.firedThresholds[thresholdName] = true
end

-- Returns true if this threshold has already fired for this player
function PlayerRegistry.hasThresholdFired(player, thresholdName)
	local entry = _data[player]
	if entry == nil then return true end  -- treat unknown players as already fired
	return entry.firedThresholds[thresholdName] == true
end

-- ─── ALL PLAYERS ITERATION ───────────────────────────────────────────────────

-- Iterate over all registered players: for player, data in PlayerRegistry.all() do ... end
function PlayerRegistry.all()
	return pairs(_data)
end

return PlayerRegistry
