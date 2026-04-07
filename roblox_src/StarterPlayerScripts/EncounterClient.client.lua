-- EncounterClient.client.lua (LocalScript — StarterPlayerScripts)
-- Detects when a player presses a ProximityPrompt on an encounter Part and
-- fires EncounterInteract to the server.
--
-- All encounter Parts in Workspace must:
--   1. Have a name matching "Encounter_<EncounterNameWithUnderscores>"
--   2. Contain a ProximityPrompt with ActionText = "Investigate"
--
-- This script finds those prompts and wires them up.

local Players           = game:GetService("Players")
local ReplicatedStorage = game:GetService("ReplicatedStorage")
local CollectionService = game:GetService("CollectionService")

local Events = ReplicatedStorage.RemoteEvents
local player = Players.LocalPlayer

-- Converts a Part name like "Encounter_CrackedSequence" to "Cracked Sequence"
-- by removing the "Encounter_" prefix and inserting spaces before capitals
local function partNameToEncounterName(partName)
	-- Remove prefix
	local withoutPrefix = string.gsub(partName, "^Encounter_", "")
	-- Insert space before each capital letter that follows a lowercase letter
	local withSpaces    = string.gsub(withoutPrefix, "(%l)(%u)", "%1 %2")
	return withSpaces
end

-- Wire a single ProximityPrompt inside an encounter Part
local function wireEncounterPrompt(part)
	local prompt = part:FindFirstChildOfClass("ProximityPrompt")
	if prompt == nil then
		-- No prompt in Studio yet — skip silently
		return
	end

	local encounterName = partNameToEncounterName(part.Name)

	prompt.Triggered:Connect(function()
		-- Fire to server — server validates and responds with perception text
		Events.EncounterInteract:FireServer({ encounterName = encounterName })
	end)
end

-- Wire all encounter Parts currently in workspace
local function wireAllEncounterParts()
	-- Find all Parts whose name starts with "Encounter_"
	for _, descendant in ipairs(workspace:GetDescendants()) do
		if descendant:IsA("BasePart") and string.sub(descendant.Name, 1, 9) == "Encounter" then
			wireEncounterPrompt(descendant)
		end
	end
end

-- Also wire any encounter Parts added later (e.g. Phantom Signal spawned by Chaos Tier 1)
workspace.DescendantAdded:Connect(function(descendant)
	if descendant:IsA("BasePart") and string.sub(descendant.Name, 1, 9) == "Encounter" then
		-- Small wait to ensure the ProximityPrompt has been added as well
		task.wait(0.2)
		wireEncounterPrompt(descendant)
	end
end)

-- Wire the gate Parts to fire StageAdvance when triggered
local function wireGateParts()
	local gateNames = { "Gate_Stage1", "Gate_Stage2" }
	for _, gateName in ipairs(gateNames) do
		local gatePart = workspace:FindFirstChild(gateName, true)
		if gatePart then
			local prompt = gatePart:FindFirstChildOfClass("ProximityPrompt")
			if prompt then
				prompt.Triggered:Connect(function()
					-- Tell the server this player is stepping through the gate
					Events.StageAdvance:FireServer({ stageNumber = tonumber(string.sub(gateName, -1)) })
				end)
			end
		end
	end
end

-- Run wiring after a short delay to ensure workspace is loaded
task.delay(1, function()
	wireAllEncounterParts()
	wireGateParts()
end)
