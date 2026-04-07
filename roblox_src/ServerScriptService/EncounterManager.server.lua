-- EncounterManager.server.lua (Script — ServerScriptService)
-- Handles all encounter interaction and resolution:
--   • Client presses ProximityPrompt → EncounterInteract fires
--   • Server looks up the player's role and returns their perception text
--   • After a second interaction (or auto-resolve), marks encounter resolved
--   • Applies T/C/I/B rewards to the player's hidden state
--   • Opens the gate when the stage clear threshold is met

local ReplicatedStorage   = game:GetService("ReplicatedStorage")
local ServerScriptService = game:GetService("ServerScriptService")

local EncounterData    = require(ReplicatedStorage.Shared.EncounterData)
local StageData        = require(ReplicatedStorage.Shared.StageData)
local GameState        = require(ServerScriptService.Modules.GameState)
local PlayerRegistry   = require(ServerScriptService.Modules.PlayerRegistry)

local Events = ReplicatedStorage.RemoteEvents

-- Tracks players who are currently "inside" an encounter (waiting for resolve confirmation)
-- Key: player, Value: encounter name they interacted with
local pendingResolve = {}

-- ─── HELPERS ─────────────────────────────────────────────────────────────────

-- Apply encounter stat rewards to the player, respecting BalanceTier1 multiplier
local function applyRewards(player, encounterName)
	local encounter = EncounterData[encounterName]
	if encounter == nil then return end

	local entry = PlayerRegistry.get(player)
	if entry == nil then return end

	local mult = entry.rewardMultiplier  -- 1.0 normally, 0.5 if BalanceTier1 fired

	PlayerRegistry.addStat(player, "truth",    encounter.rewards.truth    * mult)
	PlayerRegistry.addStat(player, "chaos",    encounter.rewards.chaos    * mult)
	PlayerRegistry.addStat(player, "illusion", encounter.rewards.illusion * mult)
	PlayerRegistry.addStat(player, "balance",  encounter.rewards.balance  * mult)
end

-- Check if the stage clear threshold has been met; open the gate if so
local function checkGateCondition()
	if GameState.gateOpen then return end  -- already open

	local stageDef = StageData[GameState.currentStage]
	if stageDef == nil then return end

	if GameState.encountersResolved >= stageDef.clearThreshold then
		GameState.gateOpen = true

		-- Make the gate Part passable (transparent + no collision) so players can walk through
		local gatePart = workspace:FindFirstChild(stageDef.gatePartName, true)
		if gatePart then
			gatePart.Transparency = 0.6
			gatePart.CanCollide   = false
		end

		-- Tell all clients the gate is now open
		Events.GateOpened:FireAllClients({ stageNumber = GameState.currentStage })
		Events.TeamLog:FireAllClients({
			message = "The gate is open. Find the exit.",
		})

		print("EncounterManager: gate opened for stage " .. GameState.currentStage)
	end
end

-- ─── FIRST INTERACTION: show perception text ─────────────────────────────────

-- Client fires EncounterInteract when the player presses the ProximityPrompt.
-- Server responds with the role-specific perception text.
Events.EncounterInteract.OnServerEvent:Connect(function(player, data)
	-- Guard: game must be in Playing phase
	if GameState.phase ~= "Playing" then return end

	local encounterName = data and data.encounterName
	if encounterName == nil then
		warn("EncounterManager: EncounterInteract received with no encounterName from " .. player.Name)
		return
	end

	-- Guard: encounter must exist and be active in the current stage
	local stateEntry = GameState.encounterStates[encounterName]
	if stateEntry == nil then
		warn("EncounterManager: encounter '" .. encounterName .. "' not in current stage")
		return
	end
	if stateEntry.resolved then
		-- Already resolved — tell client and skip
		Events.OpenDialogue:FireClient(player, {
			npcName    = encounterName,
			lines      = { "This has already been resolved." },
			trustLevel = 0,
			isEncounter = true,
		})
		return
	end

	local encounter = EncounterData[encounterName]
	if encounter == nil then
		warn("EncounterManager: no encounter data for '" .. encounterName .. "'")
		return
	end

	-- Look up this player's role to find their perception text
	local role = PlayerRegistry.getRole(player)
	if role == nil then return end

	local perceptionText = encounter.perception[role]
	if perceptionText == nil then
		perceptionText = "You sense something here, but cannot read it."
	end

	-- Store that this player is pending resolve for this encounter
	pendingResolve[player] = encounterName

	-- Send the perception text to the client as a dialogue-style popup
	Events.OpenDialogue:FireClient(player, {
		npcName     = encounterName,
		lines       = {
			perceptionText,
			-- Second line prompts the player to share with their team then confirm resolve
			"[Share this with your team. Press confirm when your team agrees.]",
		},
		trustLevel  = 0,
		isEncounter = true,  -- client uses this to show a "Resolve" button instead of "Next"
	})
end)

-- ─── SECOND INTERACTION: resolve the encounter ────────────────────────────────

-- Client fires DialogueChoice with lineIndex == -1 to signal "resolve" confirmation.
-- This happens when the player clicks the Resolve button in the encounter popup.
Events.DialogueChoice.OnServerEvent:Connect(function(player, data)
	if GameState.phase ~= "Playing" then return end

	-- lineIndex == -1 is the resolve signal (normal dialogue advances use 1, 2, 3...)
	if data.lineIndex ~= -1 then return end

	local encounterName = pendingResolve[player]
	if encounterName == nil then return end

	-- Clear the pending marker
	pendingResolve[player] = nil

	-- Attempt to resolve the encounter in GameState
	local wasNew = GameState.resolveEncounter(encounterName)
	if not wasNew then return end  -- another player already resolved it

	-- Apply stat rewards to the player who resolved it
	applyRewards(player, encounterName)

	-- Notify all clients: encounter resolved, update their world view
	Events.EncounterResolved:FireAllClients({
		encounterName = encounterName,
		resolvedBy    = PlayerRegistry.getRole(player) or player.Name,
	})

	-- Post to the shared team log
	Events.TeamLog:FireAllClients({
		message = (PlayerRegistry.getRole(player) or player.Name)
			.. " resolved: " .. encounterName,
	})

	-- Check if the gate should now open
	checkGateCondition()

	-- Trigger ThresholdWatcher to check if any new thresholds have been crossed
	-- (ThresholdWatcher listens to EncounterResolved on the server event bus implicitly —
	--  see ThresholdWatcher.server.lua for how it hooks in)
end)
