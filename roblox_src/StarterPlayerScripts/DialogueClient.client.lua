-- DialogueClient.client.lua (LocalScript — StarterPlayerScripts)
-- Shows dialogue and encounter perception popups when OpenDialogue fires.
-- Player presses Next to advance through lines.
-- Encounter popups have a Resolve button instead of Next on the final line.

local Players           = game:GetService("Players")
local ReplicatedStorage = game:GetService("ReplicatedStorage")

local Events = ReplicatedStorage.RemoteEvents
local player = Players.LocalPlayer

-- ─── UI CONSTANTS ────────────────────────────────────────────────────────────

local BG_COLOR      = Color3.fromRGB(8, 8, 14)
local BORDER_COLOR  = Color3.fromRGB(60, 60, 80)
local TEXT_COLOR    = Color3.fromRGB(220, 220, 230)
local ACCENT_COLOR  = Color3.fromRGB(255, 220, 80)
local RESOLVE_COLOR = Color3.fromRGB(80, 220, 120)

-- ─── STATE ───────────────────────────────────────────────────────────────────

local currentLines      = {}   -- list of dialogue line strings
local currentLineIndex  = 0    -- which line we are showing
local currentNpcName    = ""   -- the NPC or encounter name for this session
local isEncounterPopup  = false -- whether this is an encounter (shows Resolve button)
local dialogueGui       = nil  -- ScreenGui reference
local textLabel         = nil  -- the main text area
local speakerLabel      = nil  -- NPC/encounter name display
local nextButton        = nil  -- "Next" / "Resolve" button

-- ─── GUI BUILDER ─────────────────────────────────────────────────────────────

local function buildDialogueGui()
	-- Destroy any existing instance
	if dialogueGui and dialogueGui.Parent then
		dialogueGui:Destroy()
	end

	dialogueGui                    = Instance.new("ScreenGui")
	dialogueGui.Name               = "DialogueGui"
	dialogueGui.ResetOnSpawn       = false
	dialogueGui.IgnoreGuiInset     = true
	dialogueGui.Enabled            = false   -- hidden until a dialogue opens
	dialogueGui.Parent             = player.PlayerGui

	-- Main container frame (centered, lower portion of screen)
	local container                = Instance.new("Frame")
	container.Name                 = "Container"
	container.Size                 = UDim2.new(0.55, 0, 0.28, 0)
	container.Position             = UDim2.new(0.225, 0, 0.65, 0)
	container.BackgroundColor3     = BG_COLOR
	container.BackgroundTransparency = 0.05
	container.BorderSizePixel      = 0
	container.Parent               = dialogueGui

	-- Colored top border line
	local topBorder                = Instance.new("Frame")
	topBorder.Size                 = UDim2.new(1, 0, 0, 2)
	topBorder.BackgroundColor3     = BORDER_COLOR
	topBorder.BorderSizePixel      = 0
	topBorder.Parent               = container

	-- Speaker / encounter name label
	speakerLabel                   = Instance.new("TextLabel")
	speakerLabel.Name              = "SpeakerLabel"
	speakerLabel.Size              = UDim2.new(1, -20, 0, 22)
	speakerLabel.Position          = UDim2.new(0, 10, 0, 6)
	speakerLabel.BackgroundTransparency = 1
	speakerLabel.Text              = ""
	speakerLabel.TextColor3        = ACCENT_COLOR
	speakerLabel.Font              = Enum.Font.GothamBold
	speakerLabel.TextSize          = 14
	speakerLabel.TextXAlignment    = Enum.TextXAlignment.Left
	speakerLabel.Parent            = container

	-- Main dialogue text
	textLabel                      = Instance.new("TextLabel")
	textLabel.Name                 = "DialogueText"
	textLabel.Size                 = UDim2.new(1, -20, 1, -60)
	textLabel.Position             = UDim2.new(0, 10, 0, 32)
	textLabel.BackgroundTransparency = 1
	textLabel.Text                 = ""
	textLabel.TextColor3           = TEXT_COLOR
	textLabel.Font                 = Enum.Font.GothamItalic
	textLabel.TextSize             = 13
	textLabel.TextXAlignment       = Enum.TextXAlignment.Left
	textLabel.TextYAlignment       = Enum.TextYAlignment.Top
	textLabel.TextWrapped          = true
	textLabel.Parent               = container

	-- Next / Resolve button
	nextButton                     = Instance.new("TextButton")
	nextButton.Name                = "NextButton"
	nextButton.Size                = UDim2.new(0, 110, 0, 28)
	nextButton.Position            = UDim2.new(1, -120, 1, -34)
	nextButton.BackgroundColor3    = Color3.fromRGB(30, 30, 45)
	nextButton.BorderSizePixel     = 0
	nextButton.Text                = "Next"
	nextButton.TextColor3          = TEXT_COLOR
	nextButton.Font                = Enum.Font.GothamBold
	nextButton.TextSize            = 13
	nextButton.Parent              = container

	-- Close button (escape the dialogue early)
	local closeButton              = Instance.new("TextButton")
	closeButton.Name               = "CloseButton"
	closeButton.Size               = UDim2.new(0, 24, 0, 24)
	closeButton.Position           = UDim2.new(1, -28, 0, 4)
	closeButton.BackgroundTransparency = 1
	closeButton.Text               = "✕"
	closeButton.TextColor3         = Color3.fromRGB(120, 120, 140)
	closeButton.Font               = Enum.Font.GothamBold
	closeButton.TextSize           = 14
	closeButton.Parent             = container

	-- ── BUTTON ACTIONS ──────────────────────────────────────────────────────

	nextButton.MouseButton1Click:Connect(function()
		if #currentLines == 0 then return end

		if currentLineIndex >= #currentLines then
			-- Last line: check if this is an encounter resolve
			if isEncounterPopup then
				-- Signal server to resolve this encounter
				Events.DialogueChoice:FireServer({
					npcName   = currentNpcName,
					lineIndex = -1,   -- -1 = resolve signal
				})
			end
			closeDialogue()
		else
			-- Advance to next line
			Events.DialogueChoice:FireServer({
				npcName   = currentNpcName,
				lineIndex = currentLineIndex,  -- the line that was just read
			})
			currentLineIndex = currentLineIndex + 1
			showCurrentLine()
		end
	end)

	closeButton.MouseButton1Click:Connect(function()
		closeDialogue()
	end)
end

-- ─── SHOW CURRENT LINE ───────────────────────────────────────────────────────

function showCurrentLine()
	if textLabel == nil then return end

	local line = currentLines[currentLineIndex]
	if line == nil then return end

	textLabel.Text = line

	-- Update button text on the last line
	local isLastLine = (currentLineIndex >= #currentLines)
	if isLastLine and isEncounterPopup then
		nextButton.Text            = "Resolve"
		nextButton.BackgroundColor3 = Color3.fromRGB(20, 50, 30)
		nextButton.TextColor3      = RESOLVE_COLOR
	else
		nextButton.Text            = isLastLine and "Close" or "Next"
		nextButton.BackgroundColor3 = Color3.fromRGB(30, 30, 45)
		nextButton.TextColor3      = TEXT_COLOR
	end
end

-- ─── OPEN / CLOSE ────────────────────────────────────────────────────────────

local function openDialogue(data)
	currentNpcName     = data.npcName or ""
	currentLines       = data.lines   or {}
	currentLineIndex   = 1
	isEncounterPopup   = data.isEncounter == true

	if dialogueGui == nil then
		buildDialogueGui()
	end

	speakerLabel.Text  = currentNpcName
	dialogueGui.Enabled = true

	showCurrentLine()
end

function closeDialogue()
	if dialogueGui then
		dialogueGui.Enabled = false
	end
	currentLines     = {}
	currentLineIndex = 0
	currentNpcName   = ""
end

-- ─── SERVER EVENT LISTENERS ───────────────────────────────────────────────────

Events.OpenDialogue.OnClientEvent:Connect(function(data)
	openDialogue(data)
end)

Events.CloseDialogue.OnClientEvent:Connect(function()
	closeDialogue()
end)

-- ─── STARTUP ─────────────────────────────────────────────────────────────────

-- Build the GUI on startup (hidden until needed)
buildDialogueGui()
