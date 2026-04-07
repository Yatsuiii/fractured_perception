-- UIController.client.lua (LocalScript — StarterPlayerScripts)
-- Builds and manages all ScreenGui panels for every role.
-- Panels update in response to PerceptionUpdate events from the server.
--
-- Panels:
--   All roles:   Team log (bottom strip), Hidden state bars (T/C/I/B), Threshold notification
--   Blind:       Sound cue panel (right side)
--   Delayed:     "X.Xs behind reality" label + NPC trust bars (right side)
--   Hallucinating: Stability meter + ghost count (right side)

local Players           = game:GetService("Players")
local ReplicatedStorage = game:GetService("ReplicatedStorage")

local RoleData = require(ReplicatedStorage.Shared.RoleData)
local Events   = ReplicatedStorage.RemoteEvents
local player   = Players.LocalPlayer

-- ─── GUI CREATION HELPERS ────────────────────────────────────────────────────

-- Creates a ScreenGui and attaches it to PlayerGui. Returns the ScreenGui.
local function makeScreenGui(name)
	local existing = player.PlayerGui:FindFirstChild(name)
	if existing then existing:Destroy() end

	local gui               = Instance.new("ScreenGui")
	gui.Name                = name
	gui.ResetOnSpawn        = false
	gui.IgnoreGuiInset      = true
	gui.Parent              = player.PlayerGui
	return gui
end

-- Creates a Frame with a dark semi-transparent background
local function makePanel(parent, position, size, name)
	local frame                      = Instance.new("Frame")
	frame.Name                       = name or "Panel"
	frame.Position                   = position
	frame.Size                       = size
	frame.BackgroundColor3           = Color3.fromRGB(10, 10, 15)
	frame.BackgroundTransparency     = 0.25
	frame.BorderSizePixel            = 0
	frame.Parent                     = parent
	return frame
end

-- Creates a TextLabel inside a parent
local function makeLabel(parent, text, size, position, textColor, fontSize)
	local label                      = Instance.new("TextLabel")
	label.Size                       = size
	label.Position                   = position
	label.BackgroundTransparency     = 1
	label.Text                       = text
	label.TextColor3                 = textColor or Color3.fromRGB(220, 220, 220)
	label.Font                       = Enum.Font.Gotham
	label.TextSize                   = fontSize or 13
	label.TextXAlignment             = Enum.TextXAlignment.Left
	label.TextWrapped                = true
	label.Parent                     = parent
	return label
end

-- Creates a thin progress bar (for trust/stat bars)
-- Returns the background frame and the fill frame
local function makeBar(parent, position, size, fillColor)
	local bg           = Instance.new("Frame")
	bg.Position        = position
	bg.Size            = size
	bg.BackgroundColor3 = Color3.fromRGB(40, 40, 40)
	bg.BorderSizePixel = 0
	bg.Parent          = parent

	local fill         = Instance.new("Frame")
	fill.Name          = "Fill"
	fill.Size          = UDim2.new(0, 0, 1, 0)  -- starts at 0 width
	fill.BackgroundColor3 = fillColor
	fill.BorderSizePixel  = 0
	fill.Parent        = bg

	return bg, fill
end

-- ─── WAIT FOR ROLE ───────────────────────────────────────────────────────────

local mainGui      = nil
local rolePanel    = nil      -- right-side panel, content differs per role
local teamLogFrame = nil      -- bottom team log
local statBars     = {}       -- T/C/I/B bar fill references { T=fill, C=fill, I=fill, B=fill }

-- Wait for role assignment then build the UI
local function buildUI(role)
	local accentColor = RoleData.UIColor[role]

	mainGui = makeScreenGui("FracturedPerceptionUI")

	-- ── BOTTOM TEAM LOG ──────────────────────────────────────────────────────
	-- Scrolling list of team events (encounter resolved, gate opened, etc.)
	local logPanel = makePanel(mainGui,
		UDim2.new(0, 10, 0.88, 0),
		UDim2.new(0.6, 0, 0.1, 0),
		"TeamLogPanel"
	)

	local logTitle = makeLabel(logPanel, "TEAM LOG", UDim2.new(1, -10, 0, 18), UDim2.new(0, 5, 0, 2),
		accentColor, 11)

	teamLogFrame = Instance.new("ScrollingFrame")
	teamLogFrame.Name                  = "LogScroll"
	teamLogFrame.Size                  = UDim2.new(1, -10, 1, -22)
	teamLogFrame.Position              = UDim2.new(0, 5, 0, 22)
	teamLogFrame.BackgroundTransparency = 1
	teamLogFrame.ScrollBarThickness    = 4
	teamLogFrame.CanvasSize            = UDim2.new(0, 0, 0, 0)
	teamLogFrame.AutomaticCanvasSize   = Enum.AutomaticSize.Y
	teamLogFrame.Parent                = logPanel

	local logLayout = Instance.new("UIListLayout")
	logLayout.SortOrder  = Enum.SortOrder.LayoutOrder
	logLayout.Padding    = UDim.new(0, 2)
	logLayout.Parent     = teamLogFrame

	-- ── T/C/I/B STAT BARS ───────────────────────────────────────────────────
	-- Displayed in the bottom-right corner for all roles
	local statPanel = makePanel(mainGui,
		UDim2.new(0.72, 0, 0.88, 0),
		UDim2.new(0.27, 0, 0.1, 0),
		"StatPanel"
	)

	makeLabel(statPanel, "HIDDEN STATE", UDim2.new(1, -10, 0, 16), UDim2.new(0, 5, 0, 2),
		accentColor, 10)

	local statDefs = {
		{ key = "T", label = "TRUTH",    color = Color3.fromRGB(100, 200, 255) },
		{ key = "C", label = "CHAOS",    color = Color3.fromRGB(255, 80,  80)  },
		{ key = "I", label = "ILLUSION", color = Color3.fromRGB(200, 100, 255) },
		{ key = "B", label = "BALANCE",  color = Color3.fromRGB(80,  220, 120) },
	}

	for i, def in ipairs(statDefs) do
		local yOffset = 18 + (i - 1) * 16
		makeLabel(statPanel, def.label, UDim2.new(0, 55, 0, 13),
			UDim2.new(0, 5, 0, yOffset), Color3.fromRGB(180, 180, 180), 10)

		local _, fill = makeBar(statPanel,
			UDim2.new(0, 65, 0, yOffset + 3),
			UDim2.new(1, -70, 0, 8),
			def.color
		)
		statBars[def.key] = fill
	end

	-- ── RIGHT-SIDE ROLE PANEL ────────────────────────────────────────────────
	-- Contents differ per role; built below
	rolePanel = makePanel(mainGui,
		UDim2.new(0.78, 0, 0, 5),
		UDim2.new(0.21, 0, 0.82, 0),
		"RolePanel"
	)

	makeLabel(rolePanel, string.upper(role), UDim2.new(1, -10, 0, 18),
		UDim2.new(0, 5, 0, 4), accentColor, 13)

	if role == "Blind" then
		buildBlindPanel(rolePanel)
	elseif role == "Delayed" then
		buildDelayedPanel(rolePanel)
	elseif role == "Hallucinating" then
		buildHallucinatingPanel(rolePanel)
	end
end

-- ─── BLIND PANEL ─────────────────────────────────────────────────────────────

local soundCueList = nil  -- ScrollingFrame reference, filled per update

function buildBlindPanel(parent)
	makeLabel(parent, "SOUNDS NEARBY", UDim2.new(1, -10, 0, 14),
		UDim2.new(0, 5, 0, 26), Color3.fromRGB(160, 160, 160), 11)

	soundCueList = Instance.new("ScrollingFrame")
	soundCueList.Name                  = "SoundCues"
	soundCueList.Size                  = UDim2.new(1, -10, 1, -46)
	soundCueList.Position              = UDim2.new(0, 5, 0, 46)
	soundCueList.BackgroundTransparency = 1
	soundCueList.ScrollBarThickness    = 3
	soundCueList.CanvasSize            = UDim2.new(0, 0, 0, 0)
	soundCueList.AutomaticCanvasSize   = Enum.AutomaticSize.Y
	soundCueList.Parent                = parent

	local layout = Instance.new("UIListLayout")
	layout.SortOrder = Enum.SortOrder.LayoutOrder
	layout.Padding   = UDim.new(0, 3)
	layout.Parent    = soundCueList
end

-- Rebuild the sound cue list from a PerceptionUpdate packet
local colorMap = {
	Yellow = Color3.fromRGB(255, 220, 80),
	Gray   = Color3.fromRGB(140, 140, 140),
	Red    = Color3.fromRGB(255, 80,  80),
	Green  = Color3.fromRGB(80,  220, 120),
	White  = Color3.fromRGB(220, 220, 220),
}

local function updateBlindPanel(data)
	if soundCueList == nil then return end

	-- Clear old cues
	for _, child in ipairs(soundCueList:GetChildren()) do
		if child:IsA("TextLabel") then child:Destroy() end
	end

	-- If no cues, show silence message
	if #data.soundCues == 0 then
		local silenceLabel = Instance.new("TextLabel")
		silenceLabel.Size                 = UDim2.new(1, 0, 0, 16)
		silenceLabel.BackgroundTransparency = 1
		silenceLabel.Text                 = "... silence ..."
		silenceLabel.TextColor3           = Color3.fromRGB(80, 80, 80)
		silenceLabel.Font                 = Enum.Font.GothamItalic
		silenceLabel.TextSize             = 12
		silenceLabel.TextXAlignment       = Enum.TextXAlignment.Left
		silenceLabel.LayoutOrder          = 1
		silenceLabel.Parent               = soundCueList
		return
	end

	-- Add a label for each sound cue
	for i, cue in ipairs(data.soundCues) do
		local label = Instance.new("TextLabel")
		label.Size                 = UDim2.new(1, 0, 0, 16)
		label.BackgroundTransparency = 1
		label.Text                 = cue.label
		label.TextColor3           = colorMap[cue.color] or colorMap.White
		label.Font                 = Enum.Font.Gotham
		label.TextSize             = 12
		label.TextXAlignment       = Enum.TextXAlignment.Left
		label.LayoutOrder          = i
		label.Parent               = soundCueList
	end
end

-- ─── DELAYED PANEL ───────────────────────────────────────────────────────────

local delayLabel     = nil  -- "Viewing X.Xs behind"
local trustListFrame = nil  -- list of NPC trust bars

function buildDelayedPanel(parent)
	delayLabel = makeLabel(parent, "Viewing 3.0s behind reality",
		UDim2.new(1, -10, 0, 30), UDim2.new(0, 5, 0, 26),
		Color3.fromRGB(255, 200, 80), 11)

	makeLabel(parent, "NPC TRUST", UDim2.new(1, -10, 0, 14),
		UDim2.new(0, 5, 0, 62), Color3.fromRGB(160, 160, 160), 11)

	trustListFrame = Instance.new("Frame")
	trustListFrame.Name   = "TrustList"
	trustListFrame.Size   = UDim2.new(1, -10, 1, -82)
	trustListFrame.Position = UDim2.new(0, 5, 0, 82)
	trustListFrame.BackgroundTransparency = 1
	trustListFrame.Parent = parent

	local layout = Instance.new("UIListLayout")
	layout.SortOrder = Enum.SortOrder.LayoutOrder
	layout.Padding   = UDim.new(0, 6)
	layout.Parent    = trustListFrame
end

local function updateDelayedPanel(data)
	if delayLabel then
		delayLabel.Text = string.format("Viewing %.1fs behind reality", data.delaySeconds or 3.0)
	end

	if trustListFrame == nil or data.npcTrust == nil then return end

	-- Rebuild trust bars
	for _, child in ipairs(trustListFrame:GetChildren()) do
		if not child:IsA("UIListLayout") then child:Destroy() end
	end

	for i, npcEntry in ipairs(data.npcTrust) do
		local trust = npcEntry.trust

		-- Trust color: green if high, yellow if mid, red if low
		local trustColor
		if trust >= 0.7 then
			trustColor = Color3.fromRGB(80, 220, 120)
		elseif trust >= 0.35 then
			trustColor = Color3.fromRGB(255, 220, 80)
		else
			trustColor = Color3.fromRGB(255, 80, 80)
		end

		local row = Instance.new("Frame")
		row.Name   = "TrustRow_" .. i
		row.Size   = UDim2.new(1, 0, 0, 28)
		row.BackgroundTransparency = 1
		row.LayoutOrder = i
		row.Parent = trustListFrame

		makeLabel(row, npcEntry.name, UDim2.new(1, 0, 0, 13), UDim2.new(0, 0, 0, 0),
			Color3.fromRGB(200, 200, 200), 11)

		local _, fill = makeBar(row, UDim2.new(0, 0, 0, 16), UDim2.new(1, 0, 0, 8), trustColor)
		fill.Size = UDim2.new(trust, 0, 1, 0)  -- trust is 0.0–1.0 → 0%–100% width
	end
end

-- ─── HALLUCINATING PANEL ─────────────────────────────────────────────────────

local stabilityLabel = nil
local ghostCountLabel = nil

function buildHallucinatingPanel(parent)
	makeLabel(parent, "STABILITY", UDim2.new(1, -10, 0, 14),
		UDim2.new(0, 5, 0, 26), Color3.fromRGB(160, 160, 160), 11)

	stabilityLabel = makeLabel(parent, "100%", UDim2.new(1, -10, 0, 20),
		UDim2.new(0, 5, 0, 42), Color3.fromRGB(80, 220, 120), 16)

	ghostCountLabel = makeLabel(parent, "Ghosts seen: 0", UDim2.new(1, -10, 0, 14),
		UDim2.new(0, 5, 0, 68), Color3.fromRGB(180, 100, 255), 11)

	makeLabel(parent, "[reality is uncertain]", UDim2.new(1, -10, 0, 30),
		UDim2.new(0, 5, 0, 90), Color3.fromRGB(100, 60, 140), 10)
end

local function updateHallucinatingPanel(data)
	-- Stability: percentage of nearby tiles that are NOT distorted
	-- For MVP, we compute this client-side based on how many tiles in _G.DistortedTiles
	-- are within a local radius. Here we use the server-sent value if available.
	local stabilityPct = data.stabilityPercent

	if stabilityPct == nil then
		-- Compute locally: fewer distorted tiles in list → higher stability
		local total     = 80 * 30  -- map size
		local distorted = _G.DistortedTiles and #_G.DistortedTiles or 0
		stabilityPct    = math.floor(100 - (distorted / total * 100))
	end

	if stabilityLabel then
		stabilityLabel.Text = tostring(stabilityPct) .. "%"
		if stabilityPct > 70 then
			stabilityLabel.TextColor3 = Color3.fromRGB(80, 220, 120)   -- green
		elseif stabilityPct > 40 then
			stabilityLabel.TextColor3 = Color3.fromRGB(255, 220, 80)   -- yellow
		else
			stabilityLabel.TextColor3 = Color3.fromRGB(255, 80, 80)    -- red
		end
	end

	if ghostCountLabel and data.entityPositions then
		ghostCountLabel.Text = "Ghosts seen: " .. #data.entityPositions
	end
end

-- ─── TEAM LOG ────────────────────────────────────────────────────────────────

local logEntry = 0

local function addTeamLogEntry(message)
	if teamLogFrame == nil then return end

	logEntry = logEntry + 1

	local label = Instance.new("TextLabel")
	label.Size                  = UDim2.new(1, 0, 0, 14)
	label.BackgroundTransparency = 1
	label.Text                  = message
	label.TextColor3            = Color3.fromRGB(180, 180, 180)
	label.Font                  = Enum.Font.Gotham
	label.TextSize              = 11
	label.TextXAlignment        = Enum.TextXAlignment.Left
	label.LayoutOrder           = logEntry
	label.Parent                = teamLogFrame

	-- Keep log to last 20 entries
	local children = teamLogFrame:GetChildren()
	local labels = {}
	for _, c in ipairs(children) do
		if c:IsA("TextLabel") then table.insert(labels, c) end
	end
	if #labels > 20 then
		table.remove(labels, 1):Destroy()
	end
end

-- ─── HIDDEN STATE BARS ────────────────────────────────────────────────────────

local function updateStatBars(hiddenState)
	if hiddenState == nil then return end
	-- Each stat ranges 0–99.9; we cap display at 10 for the bar fill
	local MAX_DISPLAY = 10.0
	for key, fill in pairs(statBars) do
		local value = hiddenState[key] or 0
		fill.Size = UDim2.new(math.clamp(value / MAX_DISPLAY, 0, 1), 0, 1, 0)
	end
end

-- ─── THRESHOLD NOTIFICATION ───────────────────────────────────────────────────

-- Shows a brief pop-up when a threshold fires for this player
Events.ThresholdCrossed.OnClientEvent:Connect(function(data)
	if mainGui == nil then return end

	local notifFrame = Instance.new("Frame")
	notifFrame.Size                  = UDim2.new(0, 350, 0, 70)
	notifFrame.Position              = UDim2.new(0.5, -175, 0.15, 0)
	notifFrame.BackgroundColor3      = Color3.fromRGB(20, 10, 30)
	notifFrame.BackgroundTransparency = 0.1
	notifFrame.BorderSizePixel       = 0
	notifFrame.ZIndex                = 20
	notifFrame.Parent                = mainGui

	local title = Instance.new("TextLabel")
	title.Size              = UDim2.new(1, -10, 0, 22)
	title.Position          = UDim2.new(0, 5, 0, 5)
	title.BackgroundTransparency = 1
	title.Text              = "⚡ " .. (data.name or "Threshold")
	title.TextColor3        = Color3.fromRGB(255, 220, 80)
	title.Font              = Enum.Font.GothamBold
	title.TextSize          = 15
	title.TextXAlignment    = Enum.TextXAlignment.Left
	title.ZIndex            = 21
	title.Parent            = notifFrame

	local desc = Instance.new("TextLabel")
	desc.Size               = UDim2.new(1, -10, 0, 35)
	desc.Position           = UDim2.new(0, 5, 0, 28)
	desc.BackgroundTransparency = 1
	desc.Text               = data.description or ""
	desc.TextColor3         = Color3.fromRGB(200, 200, 200)
	desc.Font               = Enum.Font.GothamItalic
	desc.TextSize           = 12
	desc.TextWrapped        = true
	desc.TextXAlignment     = Enum.TextXAlignment.Left
	desc.ZIndex             = 21
	desc.Parent             = notifFrame

	-- Auto-dismiss after 5 seconds
	task.delay(5, function()
		if notifFrame and notifFrame.Parent then
			notifFrame:Destroy()
		end
	end)
end)

-- ─── EVENT WIRING ─────────────────────────────────────────────────────────────

-- Update UI on each perception packet from server
Events.PerceptionUpdate.OnClientEvent:Connect(function(data)
	local role = _G.PlayerRole
	if role == nil or mainGui == nil then return end

	-- Update hidden state bars (all roles)
	updateStatBars(data.hiddenState)

	-- Role-specific panel updates
	if role == "Blind" then
		updateBlindPanel(data)
	elseif role == "Delayed" then
		updateDelayedPanel(data)
	elseif role == "Hallucinating" then
		updateHallucinatingPanel(data)
	end
end)

-- Add entries to team log
Events.TeamLog.OnClientEvent:Connect(function(data)
	addTeamLogEntry(data.message or "")
end)

-- Gate opened: log it
Events.GateOpened.OnClientEvent:Connect(function(data)
	addTeamLogEntry("Gate is open — find the exit! (Stage " .. (data.stageNumber or "?") .. ")")
end)

-- Encounter resolved: log it
Events.EncounterResolved.OnClientEvent:Connect(function(data)
	addTeamLogEntry(data.resolvedBy .. " resolved: " .. data.encounterName)
end)

-- ─── STARTUP ─────────────────────────────────────────────────────────────────

-- Wait for role assignment then build the UI
Events.AssignRole.OnClientEvent:Connect(function(data)
	task.wait(4.2)  -- wait for intro screen to finish displaying
	buildUI(data.role)
end)
