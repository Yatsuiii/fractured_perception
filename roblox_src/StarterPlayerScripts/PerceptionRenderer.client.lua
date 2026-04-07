-- PerceptionRenderer.client.lua (LocalScript — StarterPlayerScripts)
-- Applies the role-specific perception filter every time a PerceptionUpdate arrives.
--
-- Blind         → updates the sound cue panel (no world changes needed)
-- Delayed       → moves ghost proxy Parts to stale entity positions
-- Hallucinating → updates ghost double Parts at offset positions

local Players           = game:GetService("Players")
local ReplicatedStorage = game:GetService("ReplicatedStorage")

local Events = ReplicatedStorage.RemoteEvents
local player = Players.LocalPlayer

-- Wait until RoleSetup has finished before doing anything
local function waitForSetup()
	while not _G.RoleSetupComplete do
		task.wait(0.1)
	end
end

-- ─── GHOST PART FACTORY ──────────────────────────────────────────────────────

-- Creates a simple Part that represents a ghost/proxy entity in the world.
-- Used by both Delayed (stale position proxies) and Hallucinating (ghost doubles).
local function createGhostPart(id, label, color, transparency)
	local part = Instance.new("Part")
	part.Name        = "Ghost_" .. id
	part.Size        = Vector3.new(2, 5, 2)       -- rough humanoid size
	part.Anchored    = true
	part.CanCollide  = false
	part.CastShadow  = false
	part.Transparency = transparency
	part.BrickColor  = BrickColor.new("Medium stone grey")
	part.Parent      = workspace

	-- Add a Highlight to tint the ghost
	local highlight          = Instance.new("SelectionBox")
	highlight.Adornee        = part
	highlight.Color3         = color
	highlight.LineThickness  = 0.05
	highlight.SurfaceTransparency = 0.5
	highlight.Parent         = part

	-- Small label above the ghost
	local billboard           = Instance.new("BillboardGui")
	billboard.Size            = UDim2.new(0, 100, 0, 30)
	billboard.StudsOffset     = Vector3.new(0, 3, 0)
	billboard.AlwaysOnTop     = false
	billboard.Parent          = part

	local nameLabel           = Instance.new("TextLabel")
	nameLabel.Size            = UDim2.new(1, 0, 1, 0)
	nameLabel.BackgroundTransparency = 1
	nameLabel.Text            = label
	nameLabel.TextColor3      = color
	nameLabel.Font            = Enum.Font.Gotham
	nameLabel.TextSize        = 12
	nameLabel.Parent          = billboard

	return part
end

-- ─── DELAYED ROLE RENDERER ───────────────────────────────────────────────────

-- Stores ghost proxy Parts indexed by entity id string
local delayedGhosts = {}

local DELAYED_GHOST_COLOR = Color3.fromRGB(255, 200, 80)  -- amber

local function updateDelayedRenderer(data)
	-- Update or create a ghost proxy for each entity in the stale position list
	local seenIds = {}

	for _, entityData in ipairs(data.entityPositions) do
		local id  = entityData.id
		local pos = Vector3.new(entityData.pos.x, entityData.pos.y, entityData.pos.z)

		seenIds[id] = true

		if delayedGhosts[id] == nil then
			-- Create a new ghost proxy for this entity
			delayedGhosts[id] = createGhostPart(id, entityData.label, DELAYED_GHOST_COLOR, 0.4)
		end

		-- Move the ghost to the stale position
		delayedGhosts[id].Position = pos
	end

	-- Remove ghost proxies for entities no longer in the list (they left the range)
	for id, ghost in pairs(delayedGhosts) do
		if not seenIds[id] then
			ghost:Destroy()
			delayedGhosts[id] = nil
		end
	end

	-- Hide real character models for other players (so only the ghost is visible)
	for _, otherPlayer in ipairs(Players:GetPlayers()) do
		if otherPlayer == player then continue end
		local character = otherPlayer.Character
		if character == nil then continue end
		for _, part in ipairs(character:GetDescendants()) do
			if part:IsA("BasePart") then
				part.LocalTransparencyModifier = 1
			end
		end
	end
end

-- ─── HALLUCINATING ROLE RENDERER ─────────────────────────────────────────────

-- Stores ghost double Parts indexed by entity id string
local hallucinatingGhosts = {}

local GHOST_COLOR = Color3.fromRGB(180, 100, 255)  -- purple

-- Stable ghost offset per entity — same formula as the Rust prototype
-- entity id is a string; we convert the first characters to a number for modulo
local function ghostOffset(idString)
	-- Hash the id string to a stable small integer
	local hashVal = 0
	for i = 1, math.min(#idString, 8) do
		hashVal = hashVal + string.byte(idString, i)
	end
	local ox = ((hashVal % 3) - 1) * 3     -- -3, 0, or +3 studs on X
	local oz = ((hashVal % 7) < 3 and -4 or 4)  -- -4 or +4 studs on Z
	return Vector3.new(ox, 0, oz)
end

local function updateHallucinatingRenderer(data)
	local seenIds = {}

	for _, entityData in ipairs(data.entityPositions) do
		local id     = entityData.id
		local realPos = Vector3.new(entityData.pos.x, entityData.pos.y, entityData.pos.z)
		local offset  = ghostOffset(id)
		local ghostPos = realPos + offset

		seenIds[id] = true

		if hallucinatingGhosts[id] == nil then
			hallucinatingGhosts[id] = createGhostPart(id, "~" .. entityData.label, GHOST_COLOR, 0.55)
		end

		-- Move ghost double to the offset position
		hallucinatingGhosts[id].Position = ghostPos
	end

	-- Remove stale ghost doubles
	for id, ghost in pairs(hallucinatingGhosts) do
		if not seenIds[id] then
			ghost:Destroy()
			hallucinatingGhosts[id] = nil
		end
	end
end

-- ─── ENCOUNTER RESOLVED: remove ghost doubles for resolved encounters ─────────

-- When an encounter is resolved, the Hallucinating player should no longer see
-- a ghost double for it (matches the Rust prototype's "Resolved" encounter state)
Events.EncounterResolved.OnClientEvent:Connect(function(data)
	if _G.PlayerRole ~= "Hallucinating" then return end
	-- Encounter ghost id is built as "enc_" + encounterName
	local ghostId = "enc_" .. data.encounterName
	if hallucinatingGhosts[ghostId] then
		hallucinatingGhosts[ghostId]:Destroy()
		hallucinatingGhosts[ghostId] = nil
	end
end)

-- ─── MAIN UPDATE HANDLER ─────────────────────────────────────────────────────

Events.PerceptionUpdate.OnClientEvent:Connect(function(data)
	local role = _G.PlayerRole
	if role == nil then return end

	-- UIController handles all panel updates — we only handle world object updates here

	if role == "Delayed" then
		updateDelayedRenderer(data)
	elseif role == "Hallucinating" then
		updateHallucinatingRenderer(data)
	end
	-- Blind role: no world changes needed — sound panel is handled in UIController
end)

-- ─── STARTUP ─────────────────────────────────────────────────────────────────

task.spawn(waitForSetup)
