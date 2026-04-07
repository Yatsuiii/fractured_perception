-- GameState.lua (ModuleScript — server only)
-- Single source of truth for the current session.
-- Only server scripts should read or write this table.
-- Clients receive snapshots via RemoteEvents, never access this directly.

local GameState = {}

-- ─── SESSION STATE ────────────────────────────────────────────────────────────

-- Which stage is currently active (1 or 2 for the MVP)
GameState.currentStage = 1

-- How many encounters have been resolved in the current stage
GameState.encountersResolved = 0

-- Whether the exit gate is passable
GameState.gateOpen = false

-- Overall game phase
-- "WaitingForPlayers" | "Playing" | "StageTransition" | "GameOver"
GameState.phase = "WaitingForPlayers"

-- ─── ENCOUNTER STATES ────────────────────────────────────────────────────────

-- Tracks which encounters have been resolved.
-- Populated by GameManager when the stage loads.
-- Example entry: GameState.encounterStates["Cracked Sequence"] = { resolved = false }
GameState.encounterStates = {}

-- ─── HELPERS ─────────────────────────────────────────────────────────────────

-- Mark an encounter as resolved. Returns true if it was newly resolved, false if already done.
function GameState.resolveEncounter(encounterName)
	local entry = GameState.encounterStates[encounterName]
	if entry == nil then
		warn("GameState.resolveEncounter: unknown encounter '" .. encounterName .. "'")
		return false
	end
	if entry.resolved then
		return false  -- already resolved, don't double-count
	end
	entry.resolved = true
	GameState.encountersResolved = GameState.encountersResolved + 1
	return true
end

-- Reset for a new stage (called by GameManager.advanceStage)
function GameState.resetForStage(stageNumber, encounterNames)
	GameState.currentStage        = stageNumber
	GameState.encountersResolved  = 0
	GameState.gateOpen            = false
	GameState.encounterStates     = {}

	-- Populate encounter states table from the list of names for this stage
	for _, name in ipairs(encounterNames) do
		GameState.encounterStates[name] = { resolved = false }
	end
end

return GameState
