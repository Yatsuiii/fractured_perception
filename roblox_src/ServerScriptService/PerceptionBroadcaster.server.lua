-- PerceptionBroadcaster.server.lua (Script — ServerScriptService)
-- Runs every 0.5 seconds and sends each player a PerceptionUpdate packet.
-- The packet contains only what that player's role needs to render their view.
--
-- Blind   → sound cues (direction + intensity labels, no positions)
-- Delayed → stale entity positions (3 seconds behind, or more if IllusionTier1 fired)
-- Hallucinating → current entity positions (ghost offset computed client-side)

local Players           = game:GetService("Players")
local RunService        = game:GetService("RunService")
local ReplicatedStorage = game:GetService("ReplicatedStorage")
local ServerScriptService = game:GetService("ServerScriptService")

local PlayerRegistry  = require(ServerScriptService.Modules.PlayerRegistry)
local PositionHistory = require(ServerScriptService.Modules.PositionHistory)
local GameState       = require(ServerScriptService.Modules.GameState)
local StageData       = require(ReplicatedStorage.Shared.StageData)
local EncounterData   = require(ReplicatedStorage.Shared.EncounterData)

local Events = ReplicatedStorage.RemoteEvents

-- How often to sample positions for the history buffer (seconds)
local RECORD_INTERVAL    = 0.1

-- How often to broadcast PerceptionUpdate packets to all clients (seconds)
local BROADCAST_INTERVAL = 0.5

-- Base delay for the Delayed role (seconds)
local BASE_DELAY = 3.0

-- Sound intensity thresholds (in studs)
local SOUND_CLOSE = 15
local SOUND_NEAR  = 30
local SOUND_FAR   = 50   -- beyond this, not reported

-- ─── DIRECTION UTILITY ────────────────────────────────────────────────────────

-- Returns a compass direction string from one position to another
-- Example: getDirection(Vector3.new(0,0,0), Vector3.new(10,0,5)) → "SE"
local function getDirection(fromPos, toPos)
	local dx    = toPos.X - fromPos.X
	local dz    = toPos.Z - fromPos.Z
	local angle = math.deg(math.atan2(dz, dx))

	-- Normalize to 0–360
	if angle < 0 then angle = angle + 360 end

	if     angle < 22.5  or angle >= 337.5 then return "E"
	elseif angle < 67.5  then return "SE"
	elseif angle < 112.5 then return "S"
	elseif angle < 157.5 then return "SW"
	elseif angle < 202.5 then return "W"
	elseif angle < 247.5 then return "NW"
	elseif angle < 292.5 then return "N"
	else                       return "NE"
	end
end

-- Returns an intensity label based on distance in studs
local function getIntensity(distance)
	if distance <= SOUND_CLOSE then
		return "close"
	elseif distance <= SOUND_NEAR then
		return "near"
	elseif distance <= SOUND_FAR then
		return "far"
	else
		return nil  -- too far away — don't include
	end
end

-- ─── POSITION RECORDING LOOP ─────────────────────────────────────────────────

-- Records positions for all characters and NPCs every RECORD_INTERVAL seconds.
-- This feeds PositionHistory so the Delayed role can query stale positions.

task.spawn(function()
	while true do
		task.wait(RECORD_INTERVAL)

		-- Record each player character's position
		for _, player in ipairs(Players:GetPlayers()) do
			local character = player.Character
			if character then
				local root = character:FindFirstChild("HumanoidRootPart")
				if root then
					PositionHistory.record(player, root.Position)
				end
			end
		end

		-- Record each NPC's position (look them up by known names in workspace)
		local npcNames = { "NPC_Watcher", "NPC_Echo", "NPC_Archivist" }
		for _, npcPartName in ipairs(npcNames) do
			local model = workspace:FindFirstChild(npcPartName, true)
			if model then
				local root = model:FindFirstChild("HumanoidRootPart")
				if root then
					PositionHistory.record(npcPartName, root.Position)
				end
			end
		end
	end
end)

-- ─── BROADCAST LOOP ──────────────────────────────────────────────────────────

task.spawn(function()
	while true do
		task.wait(BROADCAST_INTERVAL)

		if GameState.phase ~= "Playing" then continue end

		-- Build entity position lists for Delayed / Hallucinating roles
		-- We collect real positions here; Delayed queries history for the stale version

		-- Current positions of all players
		local playerPositions = {}
		for _, p in ipairs(Players:GetPlayers()) do
			local character = p.Character
			if character then
				local root = character:FindFirstChild("HumanoidRootPart")
				if root then
					table.insert(playerPositions, {
						id    = tostring(p.UserId),
						label = PlayerRegistry.getRole(p) or p.Name,
						pos   = { x = root.Position.X, y = root.Position.Y, z = root.Position.Z },
					})
				end
			end
		end

		-- Current positions of active NPCs
		local npcPositions = {}
		local npcNames = { "NPC_Watcher", "NPC_Echo", "NPC_Archivist" }
		for _, npcPartName in ipairs(npcNames) do
			local model = workspace:FindFirstChild(npcPartName, true)
			if model then
				local root = model:FindFirstChild("HumanoidRootPart")
				if root then
					-- Derive NPC display name (or use Attribute set in Studio)
					local npcName = model:GetAttribute("NpcName") or string.gsub(model.Name, "NPC_", "The ")
					table.insert(npcPositions, {
						id    = npcPartName,
						label = npcName,
						pos   = { x = root.Position.X, y = root.Position.Y, z = root.Position.Z },
					})
				end
			end
		end

		-- Active encounter positions (for sound cues + Hallucinating ghost encounters)
		local stageDef        = StageData[GameState.currentStage]
		local encounterPositions = {}
		if stageDef then
			for _, encounterName in ipairs(stageDef.encounters) do
				local stateEntry = GameState.encounterStates[encounterName]
				if stateEntry and not stateEntry.resolved then
					local encounter = EncounterData[encounterName]
					if encounter and encounter.worldPartName then
						local part = workspace:FindFirstChild(encounter.worldPartName, true)
						if part then
							table.insert(encounterPositions, {
								name  = encounterName,
								kind  = encounter.kind,
								pos   = part.Position,
							})
						end
					end
				end
			end
		end

		-- Send perception update to each player based on their role
		for _, player in ipairs(Players:GetPlayers()) do
			local entry = PlayerRegistry.get(player)
			if entry == nil then continue end

			local character = player.Character
			if character == nil then continue end
			local myRoot = character:FindFirstChild("HumanoidRootPart")
			if myRoot == nil then continue end

			local myPos = myRoot.Position
			local role  = entry.role

			-- ── BLIND PACKET ────────────────────────────────────────────────
			if role == "Blind" then
				local soundCues = {}

				-- Other players (footsteps)
				for _, p in ipairs(Players:GetPlayers()) do
					if p == player then continue end
					local c = p.Character
					if c == nil then continue end
					local r = c:FindFirstChild("HumanoidRootPart")
					if r == nil then continue end

					local dist      = (myPos - r.Position).Magnitude
					local intensity = getIntensity(dist)
					if intensity then
						local dir = getDirection(myPos, r.Position)
						table.insert(soundCues, {
							label = dir .. " Footsteps [" .. intensity .. "]",
							color = "Gray",
						})
					end
				end

				-- NPCs (named sound)
				for _, npc in ipairs(npcPositions) do
					local npcPos    = Vector3.new(npc.pos.x, npc.pos.y, npc.pos.z)
					local dist      = (myPos - npcPos).Magnitude
					local intensity = getIntensity(dist)
					if intensity then
						local dir = getDirection(myPos, npcPos)
						table.insert(soundCues, {
							label = dir .. " " .. npc.label .. " [" .. intensity .. "]",
							color = "Yellow",
						})
					end
				end

				-- Encounters (ambient sound by kind)
				local kindSound = {
					Puzzle   = "rhythmic hum",
					Enemy    = "low growl",
					Obstacle = "shifting stone",
				}
				for _, enc in ipairs(encounterPositions) do
					local dist      = (myPos - enc.pos).Magnitude
					local intensity = getIntensity(dist)
					if intensity then
						local dir   = getDirection(myPos, enc.pos)
						local sound = kindSound[enc.kind] or "faint noise"
						table.insert(soundCues, {
							label = dir .. " " .. sound .. " [" .. intensity .. "]",
							color = "Yellow",
						})
					end
				end

				-- Hidden state bars for UI panel
				Events.PerceptionUpdate:FireClient(player, {
					role      = "Blind",
					soundCues = soundCues,
					hiddenState = {
						T = entry.truth,
						C = entry.chaos,
						I = entry.illusion,
						B = entry.balance,
					},
				})

			-- ── DELAYED PACKET ───────────────────────────────────────────────
			elseif role == "Delayed" then
				local delaySeconds = BASE_DELAY + entry.delayPenalty

				-- Collect stale positions for other players
				local entityPositions = {}
				for _, p in ipairs(Players:GetPlayers()) do
					if p == player then continue end
					local stalePos = PositionHistory.getDelayed(p, delaySeconds)
					if stalePos then
						table.insert(entityPositions, {
							id    = tostring(p.UserId),
							label = PlayerRegistry.getRole(p) or p.Name,
							pos   = { x = stalePos.X, y = stalePos.Y, z = stalePos.Z },
						})
					end
				end

				-- Collect stale positions for NPCs
				local npcNames = { "NPC_Watcher", "NPC_Echo", "NPC_Archivist" }
				for _, npcPartName in ipairs(npcNames) do
					local stalePos = PositionHistory.getDelayed(npcPartName, delaySeconds)
					if stalePos then
						local npcName = string.gsub(npcPartName, "NPC_", "The ")
						table.insert(entityPositions, {
							id    = npcPartName,
							label = npcName,
							pos   = { x = stalePos.X, y = stalePos.Y, z = stalePos.Z },
						})
					end
				end

				-- NPC trust table for the side panel
				local npcTrustList = {}
				for npcName, trust in pairs(entry.npcTrust) do
					table.insert(npcTrustList, { name = npcName, trust = trust })
				end

				Events.PerceptionUpdate:FireClient(player, {
					role             = "Delayed",
					entityPositions  = entityPositions,
					delaySeconds     = delaySeconds,
					npcTrust         = npcTrustList,
					hiddenState = {
						T = entry.truth,
						C = entry.chaos,
						I = entry.illusion,
						B = entry.balance,
					},
				})

			-- ── HALLUCINATING PACKET ─────────────────────────────────────────
			elseif role == "Hallucinating" then
				-- Send current (real) positions — ghost offset is computed client-side
				local entityPositions = {}
				for _, p in ipairs(Players:GetPlayers()) do
					if p == player then continue end
					local c = p.Character
					if c == nil then continue end
					local r = c:FindFirstChild("HumanoidRootPart")
					if r == nil then continue end

					table.insert(entityPositions, {
						id    = tostring(p.UserId),
						label = PlayerRegistry.getRole(p) or p.Name,
						pos   = { x = r.Position.X, y = r.Position.Y, z = r.Position.Z },
					})
				end

				-- Also include NPCs
				for _, npc in ipairs(npcPositions) do
					table.insert(entityPositions, npc)
				end

				-- Stability = percentage of tiles in a local radius that are NOT distorted
				-- For MVP, broadcast a static value and let the client compute it from the tile list
				Events.PerceptionUpdate:FireClient(player, {
					role            = "Hallucinating",
					entityPositions = entityPositions,
					hiddenState = {
						T = entry.truth,
						C = entry.chaos,
						I = entry.illusion,
						B = entry.balance,
					},
				})
			end
		end
	end
end)
