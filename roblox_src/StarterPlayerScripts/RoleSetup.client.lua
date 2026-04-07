-- RoleSetup.client.lua (LocalScript — StarterPlayerScripts)
-- Runs on the client when the game starts.
-- Waits for the server to assign a role, then:
--   1. Stores the role locally for other client scripts to read
--   2. Initialises role-specific perception (Blind overlay, tile distortion list, etc.)
--   3. Signals other client scripts that setup is complete

local Players           = game:GetService("Players")
local ReplicatedStorage = game:GetService("ReplicatedStorage")

local RoleData = require(ReplicatedStorage.Shared.RoleData)
local Events   = ReplicatedStorage.RemoteEvents

local player = Players.LocalPlayer

-- ─── SHARED STATE (read by other LocalScripts via _G) ─────────────────────────
-- _G is a global table shared between all LocalScripts for the same player.
-- We store the role here so PerceptionRenderer and UIController can read it.
-- This is acceptable for an MVP; in a larger project you'd use a ModuleScript instead.

_G.PlayerRole         = nil     -- "Blind" | "Delayed" | "Hallucinating"
_G.DistortedTiles     = {}      -- list of {x, z} tables (used by Hallucinating renderer)
_G.RoleSetupComplete  = false   -- set to true after setup finishes

-- ─── ROLE ASSIGNMENT ─────────────────────────────────────────────────────────

Events.AssignRole.OnClientEvent:Connect(function(data)
	local role          = data.role
	local distortedTiles = data.distortedTiles or {}

	-- Store globally so other scripts can read them
	_G.PlayerRole        = role
	_G.DistortedTiles    = distortedTiles

	print("RoleSetup: assigned role = " .. role)

	-- Show a brief role introduction message on screen
	local screenGui = player.PlayerGui:FindFirstChild("RoleIntroGui")
	if screenGui == nil then
		screenGui = Instance.new("ScreenGui")
		screenGui.Name              = "RoleIntroGui"
		screenGui.ResetOnSpawn      = false
		screenGui.IgnoreGuiInset    = true
		screenGui.Parent            = player.PlayerGui
	end

	local frame = Instance.new("Frame")
	frame.Size              = UDim2.new(1, 0, 1, 0)
	frame.Position          = UDim2.new(0, 0, 0, 0)
	frame.BackgroundColor3  = Color3.fromRGB(0, 0, 0)
	frame.BackgroundTransparency = 0
	frame.Parent            = screenGui

	local label = Instance.new("TextLabel")
	label.Size              = UDim2.new(0.8, 0, 0.4, 0)
	label.Position          = UDim2.new(0.1, 0, 0.3, 0)
	label.BackgroundTransparency = 1
	label.Text              = "YOU ARE: " .. string.upper(role) .. "\n\n" .. RoleData.Descriptions[role]
	label.TextColor3        = RoleData.UIColor[role]
	label.Font              = Enum.Font.GothamBold
	label.TextSize          = 24
	label.TextWrapped       = true
	label.Parent            = frame

	-- Fade out the intro screen after 4 seconds
	task.delay(4, function()
		-- Tween the transparency to 1 (invisible) then destroy
		for step = 1, 20 do
			frame.BackgroundTransparency = step / 20
			task.wait(0.05)
		end
		screenGui:Destroy()
	end)

	-- Signal other scripts that role setup is complete
	_G.RoleSetupComplete = true

	-- For the Blind role: apply full-screen darkness overlay immediately
	-- (PerceptionRenderer will take over after this)
	if role == "Blind" then
		-- A full black overlay is created here as early as possible so the player
		-- never sees the 3D world. PerceptionRenderer manages it after setup.
		local darkGui = Instance.new("ScreenGui")
		darkGui.Name           = "BlindOverlay"
		darkGui.ResetOnSpawn   = false
		darkGui.IgnoreGuiInset = true
		darkGui.Parent         = player.PlayerGui

		local darkFrame = Instance.new("Frame")
		darkFrame.Name                    = "DarkFrame"
		darkFrame.Size                    = UDim2.new(1, 0, 1, 0)
		darkFrame.Position                = UDim2.new(0, 0, 0, 0)
		darkFrame.BackgroundColor3        = Color3.fromRGB(0, 0, 0)
		darkFrame.BackgroundTransparency  = 0
		darkFrame.ZIndex                  = 10  -- render above most UI
		darkFrame.Parent                  = darkGui
	end

	-- For the Hallucinating role: apply tile distortion to the stage geometry
	if role == "Hallucinating" then
		-- Wait a moment for the stage geometry to be visible
		task.delay(2.5, function()
			applyTileDistortion(distortedTiles)
		end)
	end
end)

-- ─── TILE DISTORTION (Hallucinating role only) ────────────────────────────────

-- Swaps the visual material of distorted tiles so they look like the wrong type.
-- Real walkability is not changed — this is cosmetic only.
-- distortedTiles is a list of { x = number, z = number } grid coordinates.

-- Tile size in studs (must match how you build your stage grid in Studio)
local TILE_SIZE = 4

-- These materials are used to visually "swap" floor and wall tiles
local FLOOR_MATERIAL_REAL     = Enum.Material.SmoothPlastic  -- what normal floors look like
local WALL_MATERIAL_REAL      = Enum.Material.Concrete        -- what normal walls look like
local FLOOR_MATERIAL_DISTORTED = Enum.Material.Concrete       -- distorted floor looks like wall
local WALL_MATERIAL_DISTORTED  = Enum.Material.SmoothPlastic  -- distorted wall looks like floor

local DISTORTED_FLOOR_COLOR = BrickColor.new("Medium stone grey")
local DISTORTED_WALL_COLOR  = BrickColor.new("Light grey")

function applyTileDistortion(tileList)
	if tileList == nil or #tileList == 0 then return end

	-- Build a lookup set for quick membership checks: "x_z" → true
	local distortSet = {}
	for _, tile in ipairs(tileList) do
		distortSet[tile.x .. "_" .. tile.z] = true
	end

	-- Walk all FloorTile and WallTile tagged parts
	-- Parts must be named "FloorTile_X_Z" or "WallTile_X_Z" in Studio
	local CollectionService = game:GetService("CollectionService")

	for _, part in ipairs(CollectionService:GetTagged("FloorTile")) do
		-- Extract x, z from name format "FloorTile_X_Z"
		local x, z = string.match(part.Name, "FloorTile_(%d+)_(%d+)")
		if x and z and distortSet[x .. "_" .. z] then
			part.Material  = FLOOR_MATERIAL_DISTORTED
			part.BrickColor = DISTORTED_FLOOR_COLOR
		end
	end

	for _, part in ipairs(CollectionService:GetTagged("WallTile")) do
		local x, z = string.match(part.Name, "WallTile_(%d+)_(%d+)")
		if x and z and distortSet[x .. "_" .. z] then
			part.Material  = WALL_MATERIAL_DISTORTED
			part.BrickColor = DISTORTED_WALL_COLOR
		end
	end
end
