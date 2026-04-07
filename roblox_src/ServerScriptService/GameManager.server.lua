-- GameManager.server.lua (Script — ServerScriptService)
-- Central server brain for Fractured Perception.
-- Responsibilities:
--   • Wait for 3 players, assign roles in join order
--   • Load stage 1, then stage 2 when players advance
--   • Listen for StageAdvance events from clients (player stepped through gate)
--   • Broadcast SyncGameState to all clients after any phase change

local Players           = game:GetService("Players")
local ReplicatedStorage = game:GetService("ReplicatedStorage")
local ServerScriptService = game:GetService("ServerScriptService")

-- Shared data modules
local StageData         = require(ReplicatedStorage.Shared.StageData)

-- Server-only modules
local GameState         = require(ServerScriptService.Modules.GameState)
local PlayerRegistry    = require(ServerScriptService.Modules.PlayerRegistry)

-- RemoteEvents folder
local Events = ReplicatedStorage.RemoteEvents

-- Role assignment order — first player gets Blind, second Delayed, third Hallucinating
local ROLE_ORDER = { "Blind", "Delayed", "Hallucinating" }

-- Tracks players who have joined (in order)
local joinedPlayers = {}

-- ─── HELPERS ─────────────────────────────────────────────────────────────────

-- Broadcast the current GameState snapshot to all clients
local function broadcastGameState()
	Events.SyncGameState:FireAllClients({
		stage              = GameState.currentStage,
		encountersResolved = GameState.encountersResolved,
		gateOpen           = GameState.gateOpen,
		phase              = GameState.phase,
	})
end

-- Find the spawn position for a stage (looks for a Part named spawnPartName in Workspace)
local function getSpawnPosition(stageDef)
	local spawnPart = workspace:FindFirstChild(stageDef.spawnPartName, true)  -- true = search descendants
	if spawnPart then
		-- Spawn slightly above the part so players don't clip into the floor
		return spawnPart.Position + Vector3.new(0, 3, 0)
	end
	-- Fallback if the part is missing — warn the developer and use origin
	warn("GameManager: spawn part '" .. stageDef.spawnPartName .. "' not found in Workspace!")
	return Vector3.new(0, 5, 0)
end

-- Teleport all players to the current stage's spawn position
local function teleportAllToStage(stageNumber)
	local stageDef     = StageData[stageNumber]
	local spawnPos     = getSpawnPosition(stageDef)

	-- Spread players slightly so they don't overlap (offset by 2 studs each)
	local offsets = {
		Vector3.new(0,  0,  0),
		Vector3.new(3,  0,  0),
		Vector3.new(-3, 0,  0),
	}

	local index = 1
	for _, player in ipairs(Players:GetPlayers()) do
		local character = player.Character
		if character then
			local rootPart = character:FindFirstChild("HumanoidRootPart")
			if rootPart then
				rootPart.CFrame = CFrame.new(spawnPos + (offsets[index] or Vector3.new(0,0,0)))
			end
		end
		index = index + 1
	end
end

-- Hide the previous stage geometry and show the current stage
local function switchStageVisibility(stageNumber)
	-- Hide all stages first
	for i = 1, 2 do
		local model = workspace:FindFirstChild("Stage" .. i)
		if model then
			model.Parent.Parent = workspace  -- ensure it exists
			for _, desc in ipairs(model:GetDescendants()) do
				if desc:IsA("BasePart") then
					desc.Transparency = (i == stageNumber) and 0 or 1
					desc.CanCollide    = (i == stageNumber)
				end
			end
		end
	end
end

-- Load a stage: reset GameState, set encounter list, show geometry, teleport players
local function loadStage(stageNumber)
	local stageDef = StageData[stageNumber]
	if stageDef == nil then
		-- No more stages — game over
		GameState.phase = "GameOver"
		broadcastGameState()
		return
	end

	GameState.phase = "StageTransition"
	broadcastGameState()

	-- Small delay so the transition screen can display
	task.wait(2)

	-- Reset server state for the new stage
	GameState.resetForStage(stageNumber, stageDef.encounters)

	-- Update NPC trust defaults in PlayerRegistry for the new stage's NPCs
	for player, _ in PlayerRegistry.all() do
		PlayerRegistry.init(player, PlayerRegistry.getRole(player), stageNumber)
	end

	-- Show stage geometry, hide previous stage
	switchStageVisibility(stageNumber)

	-- Move all players to the spawn point
	teleportAllToStage(stageNumber)

	GameState.phase = "Playing"
	broadcastGameState()
end

-- ─── ROLE ASSIGNMENT ─────────────────────────────────────────────────────────

local function tryStartGame()
	-- Only start when exactly 3 players have joined
	-- For testing with fewer players, change 3 to 1 or 2
	if #joinedPlayers < 3 then
		return
	end

	GameState.phase = "Playing"

	-- Assign roles in join order and initialise registry
	for i, player in ipairs(joinedPlayers) do
		local role = ROLE_ORDER[i]
		PlayerRegistry.init(player, role, GameState.currentStage)

		-- Precompute the distorted tile list for Hallucinating players
		-- (computed server-side so all clients agree on which tiles are distorted)
		local distortedTiles = {}
		if role == "Hallucinating" then
			distortedTiles = computeDistortedTiles()  -- defined below
		end

		-- Tell the client their role
		Events.AssignRole:FireClient(player, {
			role          = role,
			distortedTiles = distortedTiles,
		})
	end

	-- Load stage 1
	loadStage(1)
end

-- ─── DISTORTED TILE COMPUTATION (for Hallucinating role) ─────────────────────

-- Uses a deterministic hash so the same tiles distort every run.
-- Matches the logic from the Rust prototype: tile_hash(x, z, seed ^ 0xDEAD) % 100 < 18
-- Returns a flat list of {x, z} tables for each distorted tile.
local DISTORT_SEED    = 12345  -- fixed seed for the map; change to change layout
local DISTORT_PERCENT = 18     -- percentage of tiles that appear visually wrong

local function tileHash(x, z, seed)
	-- Simple deterministic hash mixing x, z, and seed.
	-- We use bit32 operations since Roblox LuaU supports them.
	local h = seed
	h = bit32.bxor(h, x * 2654435761)
	h = bit32.bxor(h, z * 2246822519)
	h = bit32.bxor(h, bit32.rshift(h, 16))
	return h % 100  -- returns 0–99
end

function computeDistortedTiles()
	-- Walk every position in the stage grid (80 wide × 30 deep matches Rust prototype)
	-- Adjust MAP_WIDTH / MAP_DEPTH to match your actual Studio map size
	local MAP_WIDTH = 80
	local MAP_DEPTH = 30
	local seed      = bit32.bxor(DISTORT_SEED, 0xDEAD)

	local distorted = {}
	for x = 0, MAP_WIDTH - 1 do
		for z = 0, MAP_DEPTH - 1 do
			if tileHash(x, z, seed) < DISTORT_PERCENT then
				table.insert(distorted, { x = x, z = z })
			end
		end
	end
	return distorted
end

-- ─── EVENT LISTENERS ─────────────────────────────────────────────────────────

-- Track players as they join
Players.PlayerAdded:Connect(function(player)
	table.insert(joinedPlayers, player)
	print("GameManager: " .. player.Name .. " joined (role slot " .. #joinedPlayers .. ")")
	tryStartGame()
end)

-- Clean up when a player leaves
Players.PlayerRemoving:Connect(function(player)
	PlayerRegistry.remove(player)
	-- Remove from join list
	for i, p in ipairs(joinedPlayers) do
		if p == player then
			table.remove(joinedPlayers, i)
			break
		end
	end
end)

-- A player stepped through the gate and requests stage advance
Events.StageAdvance.OnServerEvent:Connect(function(player, data)
	-- Only allow advance if the gate is actually open
	if not GameState.gateOpen then
		return
	end
	-- Only allow advance during Playing phase
	if GameState.phase ~= "Playing" then
		return
	end

	print("GameManager: stage advance requested by " .. player.Name)
	local nextStage = GameState.currentStage + 1
	loadStage(nextStage)
end)
