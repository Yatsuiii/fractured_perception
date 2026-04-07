-- NPCController.server.lua (Script — ServerScriptService)
-- Manages all NPC interactions:
--   • Player walks near an NPC and presses the ProximityPrompt
--   • Server determines the trust tier and picks the correct dialogue lines
--   • Sends dialogue to the client via OpenDialogue
--   • Applies trust deltas and T/C/I/B nudges as the player advances through lines
--   • Proximity trust gain: standing near an NPC slowly increases trust over time

local Players           = game:GetService("Players")
local ReplicatedStorage = game:GetService("ReplicatedStorage")
local ServerScriptService = game:GetService("ServerScriptService")
local RunService        = game:GetService("RunService")

local DialogueData   = require(ReplicatedStorage.Shared.DialogueData)
local PlayerRegistry = require(ServerScriptService.Modules.PlayerRegistry)
local GameState      = require(ServerScriptService.Modules.GameState)

local Events = ReplicatedStorage.RemoteEvents

-- Tracks which NPC a player is currently talking to, and their position in the dialogue
-- Key: player,  Value: { npcName, lines, currentLineIndex }
local activeSessions = {}

-- ─── FIND NPC MODEL ──────────────────────────────────────────────────────────

-- Returns the NPC Model in Workspace given its part name, or nil if not found
local function findNpcModel(npcPartName)
	return workspace:FindFirstChild(npcPartName, true)
end

-- ─── DIALOGUE SETUP ──────────────────────────────────────────────────────────

-- Called when a player presses the ProximityPrompt on an NPC
local function startDialogue(player, npcName)
	if GameState.phase ~= "Playing" then return end

	-- Do not start a second session if one is already active
	if activeSessions[player] then return end

	-- Determine trust tier for this player with this NPC
	local tier   = PlayerRegistry.getTrustTier(player, npcName)
	local role   = PlayerRegistry.getRole(player)

	if role == nil then
		warn("NPCController: player has no role assigned yet")
		return
	end

	-- Look up dialogue lines
	local npcDialogue = DialogueData[npcName]
	if npcDialogue == nil then
		warn("NPCController: no dialogue data for NPC '" .. npcName .. "'")
		return
	end

	local roleDialogue = npcDialogue[role]
	if roleDialogue == nil then
		warn("NPCController: no dialogue for role " .. role .. " with NPC " .. npcName)
		return
	end

	local tierLines = roleDialogue[tier]
	if tierLines == nil or #tierLines == 0 then
		warn("NPCController: no " .. tier .. " tier dialogue for " .. role .. " with " .. npcName)
		return
	end

	-- Store session state
	activeSessions[player] = {
		npcName          = npcName,
		lines            = tierLines,
		currentLineIndex = 1,
	}

	-- Extract just the text strings to send to the client
	local lineTexts = {}
	for _, line in ipairs(tierLines) do
		table.insert(lineTexts, line.text)
	end

	-- Send the dialogue to the client
	Events.OpenDialogue:FireClient(player, {
		npcName    = npcName,
		lines      = lineTexts,
		trustLevel = PlayerRegistry.getTrust(player, npcName),
		isEncounter = false,
	})
end

-- ─── LINE ADVANCE ─────────────────────────────────────────────────────────────

-- Client fires DialogueChoice with the line they just advanced past
Events.DialogueChoice.OnServerEvent:Connect(function(player, data)
	-- lineIndex == -1 is used for encounter resolve — skip it here
	if data.lineIndex == -1 then return end

	local session = activeSessions[player]
	if session == nil then return end

	-- Apply the trust delta and stat nudge for the line that was just read
	local lineIndex = data.lineIndex
	local line      = session.lines[lineIndex]

	if line then
		-- Apply trust change with this NPC
		PlayerRegistry.adjustTrust(player, session.npcName, line.trustDelta)

		-- Apply hidden stat nudges
		local nudge = line.statNudge
		PlayerRegistry.addStat(player, "truth",    nudge[1])
		PlayerRegistry.addStat(player, "chaos",    nudge[2])
		PlayerRegistry.addStat(player, "illusion", nudge[3])
		PlayerRegistry.addStat(player, "balance",  nudge[4])
	end

	-- If all lines have been read, end the dialogue
	if lineIndex >= #session.lines then
		activeSessions[player] = nil
		Events.CloseDialogue:FireClient(player, {})
	end
end)

-- ─── PROXIMITY PROMPT WIRING ─────────────────────────────────────────────────

-- Wire a ProximityPrompt on an NPC model to trigger dialogue
local function wireNpcPrompt(npcModel, npcName)
	-- Find the ProximityPrompt inside the model (you add this in Studio)
	local prompt = npcModel:FindFirstChildOfClass("ProximityPrompt")
	if prompt == nil then
		warn("NPCController: no ProximityPrompt found inside '" .. npcModel.Name .. "'")
		return
	end

	prompt.ActionText = "Talk"
	prompt.HoldDuration = 0  -- instant trigger

	prompt.Triggered:Connect(function(player)
		startDialogue(player, npcName)
	end)
end

-- Wire all NPC prompts found in the current stage
-- Called by a BindableEvent from GameManager when a stage loads
-- For the MVP, we wire all NPCs at game start since stages are pre-built in Studio
local function wireAllNpcs()
	-- Stage 1 NPCs
	local watcher = workspace:FindFirstChild("NPC_Watcher", true)
	if watcher then wireNpcPrompt(watcher, "The Watcher") end

	local echo1 = workspace:FindFirstChild("NPC_Echo", true)
	if echo1 then wireNpcPrompt(echo1, "The Echo") end

	-- Stage 2 NPCs
	local archivist = workspace:FindFirstChild("NPC_Archivist", true)
	if archivist then wireNpcPrompt(archivist, "The Archivist") end
end

-- Wire NPCs as soon as the game is ready
-- (A short wait ensures Studio has finished loading all descendants)
task.delay(1, wireAllNpcs)

-- ─── PROXIMITY TRUST GAIN ─────────────────────────────────────────────────────

-- Every second, players standing within 10 studs of an NPC gain +0.01 trust with that NPC
-- This runs at a low frequency to avoid performance cost

local NPC_TRUST_RADIUS   = 10   -- studs
local TRUST_GAIN_RATE    = 0.01 -- per second
local TRUST_CHECK_PERIOD = 1.0  -- seconds between checks

task.spawn(function()
	while true do
		task.wait(TRUST_CHECK_PERIOD)

		if GameState.phase ~= "Playing" then continue end

		-- Find all NPC models that are active in the scene
		-- We look for Models with an Attribute "NpcName" set in Studio
		-- OR we can use a list of known NPC part names
		local npcModels = {}
		for _, npcPartName in ipairs({ "NPC_Watcher", "NPC_Echo", "NPC_Archivist" }) do
			local model = workspace:FindFirstChild(npcPartName, true)
			if model then
				-- Get the NPC name from an Attribute set in Studio, or derive from part name
				local npcName = model:GetAttribute("NpcName")
				if npcName == nil then
					-- Fallback: derive name from part name (e.g. "NPC_Watcher" → "The Watcher")
					npcName = string.gsub(model.Name, "NPC_", "The ")
				end
				table.insert(npcModels, { model = model, name = npcName })
			end
		end

		-- Check each player's distance to each NPC
		for _, player in ipairs(Players:GetPlayers()) do
			local character = player.Character
			if character == nil then continue end
			local rootPart = character:FindFirstChild("HumanoidRootPart")
			if rootPart == nil then continue end

			for _, npc in ipairs(npcModels) do
				local npcRoot = npc.model:FindFirstChild("HumanoidRootPart")
				if npcRoot == nil then continue end

				local distance = (rootPart.Position - npcRoot.Position).Magnitude
				if distance <= NPC_TRUST_RADIUS then
					PlayerRegistry.adjustTrust(player, npc.name, TRUST_GAIN_RATE)
				end
			end
		end
	end
end)
