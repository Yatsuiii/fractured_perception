-- StageData.lua (ModuleScript)
-- Defines both playable stages: which encounters are active, where players spawn,
-- how many encounters must be resolved to open the gate, and which NPCs are present.
-- Workspace Part names here must match what you build in Studio exactly.

local StageData = {}

StageData[1] = {
	name        = "The Shattered Halls",
	description = "Reality is cracked but recognizable. Learn to communicate.",

	-- How many encounters must be resolved before the exit gate opens
	clearThreshold = 3,

	-- Name of the Model folder in Workspace that contains this stage's geometry
	modelName = "Stage1",

	-- Where all three players spawn when this stage loads (Vector3 set at Studio time)
	-- These will be used to teleport players — match this to a SpawnLocation Part's position
	spawnPartName = "Spawn_Stage1",

	-- Name of the Part that acts as the exit gate trigger
	gatePartName = "Gate_Stage1",

	-- Encounters active in this stage (must be keys in EncounterData)
	encounters = {
		"Cracked Sequence",
		"Collapsed Archway",
		"Fragment",
		"Echo Lock",
	},

	-- NPCs placed in this stage
	npcs = {
		{
			name      = "The Watcher",
			partName  = "NPC_Watcher",   -- Model name in Workspace > Stage1 > NPCs
			baseTrust = 0.6,              -- starting trust level (0.0 to 1.0)
		},
		{
			name      = "The Echo",
			partName  = "NPC_Echo",
			baseTrust = 0.4,
		},
	},
}

StageData[2] = {
	name        = "The Drowned Archive",
	description = "Sound warps through flooded ruins. Ink bleeds across walls.",

	clearThreshold = 3,

	modelName    = "Stage2",
	spawnPartName = "Spawn_Stage2",
	gatePartName = "Gate_Stage2",

	encounters = {
		"Ink Current",
		"Drowned Scribe",
		"Flooded Stacks",
		"Phantom Archive",
	},

	npcs = {
		{
			name      = "The Archivist",
			partName  = "NPC_Archivist",
			baseTrust = 0.3,
		},
		{
			name      = "The Echo",
			partName  = "NPC_Echo",
			baseTrust = 0.5,
		},
	},
}

return StageData
