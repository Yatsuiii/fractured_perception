-- ThresholdWatcher.server.lua (Script — ServerScriptService)
-- Monitors T/C/I/B hidden stats for each player after every action.
-- When a stat crosses a Tier 1 threshold for the first time, fires the effect once.
--
-- Wired into the action pipeline:
--   • Called by EncounterManager after encounter resolve (via a BindableEvent or polling)
--   • Polling approach used here for simplicity: checks every 0.5 seconds

local Players           = game:GetService("Players")
local ReplicatedStorage = game:GetService("ReplicatedStorage")
local ServerScriptService = game:GetService("ServerScriptService")

local ThresholdData  = require(ReplicatedStorage.Shared.ThresholdData)
local PlayerRegistry = require(ServerScriptService.Modules.PlayerRegistry)
local GameState      = require(ServerScriptService.Modules.GameState)
local EncounterData  = require(ReplicatedStorage.Shared.EncounterData)

local Events = ReplicatedStorage.RemoteEvents

-- ─── EFFECT HANDLERS ─────────────────────────────────────────────────────────

-- TruthTier1: raise all NPC trust by a small bonus
local function effectTruthTier1(player)
	local entry = PlayerRegistry.get(player)
	if entry == nil then return end
	for npcName, _ in pairs(entry.npcTrust) do
		PlayerRegistry.adjustTrust(player, npcName, ThresholdData.Effects.TruthTier1NpcTrustBonus)
	end
	print("ThresholdWatcher: TruthTier1 fired for " .. player.Name)
end

-- ChaosTier1: spawn a Phantom Signal encounter near the player
local function effectChaosTier1(player)
	local character = player.Character
	if character == nil then return end
	local root = character:FindFirstChild("HumanoidRootPart")
	if root == nil then return end

	-- Place a Part in Workspace that acts like an encounter
	local radius   = ThresholdData.Effects.ChaosTier1SpawnRadius
	local offsetX  = math.random(-radius, radius)
	local offsetZ  = math.random(-radius, radius)
	local spawnPos = root.Position + Vector3.new(offsetX, 0, offsetZ)

	local phantom = Instance.new("Part")
	phantom.Name         = "Encounter_PhantomSignal_" .. player.UserId
	phantom.Size         = Vector3.new(2, 2, 2)
	phantom.Position     = spawnPos
	phantom.Anchored     = true
	phantom.CanCollide   = false
	phantom.Transparency = 0.4
	phantom.BrickColor   = BrickColor.new("Bright violet")
	phantom.Parent       = workspace

	-- Add a ProximityPrompt so players can interact with it
	local prompt = Instance.new("ProximityPrompt")
	prompt.ActionText   = "Investigate"
	prompt.HoldDuration = 0
	prompt.Parent       = phantom

	-- Wire the prompt: when triggered, treat it as a Phantom Signal encounter
	prompt.Triggered:Connect(function(triggeringPlayer)
		local role = PlayerRegistry.getRole(triggeringPlayer)
		if role == nil then return end

		local perceptionText = EncounterData["Phantom Signal"].perception[role]
		Events.OpenDialogue:FireClient(triggeringPlayer, {
			npcName     = "Phantom Signal",
			lines       = { perceptionText, "[Share with your team. This was never real.]" },
			trustLevel  = 0,
			isEncounter = true,
		})

		-- Apply illusion reward and then remove the phantom
		PlayerRegistry.addStat(triggeringPlayer, "illusion", EncounterData["Phantom Signal"].rewards.illusion)

		-- Remove the phantom after a short delay
		task.delay(3, function()
			if phantom and phantom.Parent then
				phantom:Destroy()
			end
		end)
	end)

	print("ThresholdWatcher: ChaosTier1 spawned phantom for " .. player.Name)
end

-- IllusionTier1: add extra delay to the Delayed role (for this player and any Delayed teammates)
local function effectIllusionTier1(player)
	-- Find the player with the Delayed role and increase their delay penalty
	for p, data in PlayerRegistry.all() do
		if data.role == "Delayed" then
			local entry = PlayerRegistry.get(p)
			if entry then
				entry.delayPenalty = entry.delayPenalty + ThresholdData.Effects.IllusionTier1DelayPenalty
				print("ThresholdWatcher: IllusionTier1 — Delayed delay now " ..
					(3.0 + entry.delayPenalty) .. "s")
			end
		end
	end
end

-- BalanceTier1: halve future encounter rewards for this player
local function effectBalanceTier1(player)
	local entry = PlayerRegistry.get(player)
	if entry == nil then return end
	entry.rewardMultiplier = entry.rewardMultiplier * ThresholdData.Effects.BalanceTier1RewardMultiplier
	print("ThresholdWatcher: BalanceTier1 — reward multiplier now " .. entry.rewardMultiplier)
end

-- ─── THRESHOLD CHECK ─────────────────────────────────────────────────────────

-- Check all Tier 1 thresholds for one player.
-- Each threshold fires at most once (tracked via firedThresholds).
local function checkThresholdsForPlayer(player)
	local entry = PlayerRegistry.get(player)
	if entry == nil then return end

	-- Table mapping threshold name → { statName, statValue, effectFn }
	local checks = {
		{ name = "TruthTier1",    stat = entry.truth,    threshold = ThresholdData.Tier1.Truth,    effect = effectTruthTier1    },
		{ name = "ChaosTier1",    stat = entry.chaos,    threshold = ThresholdData.Tier1.Chaos,    effect = effectChaosTier1    },
		{ name = "IllusionTier1", stat = entry.illusion, threshold = ThresholdData.Tier1.Illusion, effect = effectIllusionTier1 },
		{ name = "BalanceTier1",  stat = entry.balance,  threshold = ThresholdData.Tier1.Balance,  effect = effectBalanceTier1  },
	}

	for _, check in ipairs(checks) do
		-- Skip if this threshold has already fired for this player
		if PlayerRegistry.hasThresholdFired(player, check.name) then
			continue
		end

		-- Fire if stat has reached or exceeded the threshold
		if check.stat >= check.threshold then
			-- Mark as fired so it never fires again
			PlayerRegistry.markThresholdFired(player, check.name)

			-- Run the effect
			check.effect(player)

			-- Notify the player with a flavour message
			local notif = ThresholdData.Notifications[check.name]
			if notif then
				Events.ThresholdCrossed:FireClient(player, {
					name        = notif.name,
					description = notif.description,
				})
			end
		end
	end
end

-- ─── POLLING LOOP ────────────────────────────────────────────────────────────

-- Check thresholds every 0.5 seconds for all players.
-- Polling is simple and reliable; for a more event-driven approach you could
-- call checkThresholdsForPlayer() directly from EncounterManager after rewards are applied.

task.spawn(function()
	while true do
		task.wait(0.5)

		if GameState.phase ~= "Playing" then continue end

		for _, player in ipairs(Players:GetPlayers()) do
			checkThresholdsForPlayer(player)
		end
	end
end)
