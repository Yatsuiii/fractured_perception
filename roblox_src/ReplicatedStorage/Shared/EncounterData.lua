-- EncounterData.lua (ModuleScript)
-- All encounter definitions for both stages.
-- Each encounter has: which stage it belongs to, its type, the Part name in Workspace,
-- what each role perceives when they investigate it, and T/C/I/B rewards on resolve.

local EncounterData = {}

-- encounter kind constants
EncounterData.Kind = {
	Puzzle   = "Puzzle",
	Enemy    = "Enemy",
	Obstacle = "Obstacle",
}

-- ─── STAGE 1: THE SHATTERED HALLS ───────────────────────────────────────────

EncounterData["Cracked Sequence"] = {
	kind          = "Puzzle",
	stage         = 1,
	-- Name of the Part in Workspace > Stage1 > Encounters that marks this encounter
	worldPartName = "Encounter_CrackedSequence",
	-- What each role reads when they press the ProximityPrompt
	perception = {
		Blind         = "A rhythmic tapping echoes from the tiles. The pattern is incomplete. Count the beats and tell the others.",
		Delayed       = "Tiles glow in sequence — but you see the pattern from seconds ago. It has already changed. Describe the old order.",
		Hallucinating = "Two sequences pulse at once. Only one leads forward. They mirror each other almost perfectly — almost.",
	},
	-- T/C/I/B stat rewards when this encounter is resolved
	rewards = { truth = 0.3, chaos = 0.0, illusion = 0.1, balance = 0.1 },
}

EncounterData["Collapsed Archway"] = {
	kind          = "Obstacle",
	stage         = 1,
	worldPartName = "Encounter_CollapsedArchway",
	perception = {
		Blind         = "You hear a gap in the air ahead — wind cutting through stone. There is a way through, but you cannot see it.",
		Delayed       = "You see the archway intact, but your team walked through it seconds ago. It is already fallen for them.",
		Hallucinating = "The archway collapses and rebuilds in a loop. One version is solid, one is not. Ask your team which they see.",
	},
	rewards = { truth = 0.1, chaos = 0.0, illusion = 0.2, balance = 0.2 },
}

EncounterData["Fragment"] = {
	kind          = "Enemy",
	stage         = 1,
	worldPartName = "Encounter_Fragment",
	perception = {
		Blind         = "You hear slow scraping — something dragging itself across the floor. It is close. Which direction is it moving?",
		Delayed       = "You see a figure frozen mid-step. In reality, it has already moved. Tell your team where it was heading.",
		Hallucinating = "Three copies of the same creature face you. Only one is real. The others vanish when you focus on them.",
	},
	rewards = { truth = 0.0, chaos = 0.3, illusion = 0.1, balance = 0.1 },
}

EncounterData["Echo Lock"] = {
	kind          = "Puzzle",
	stage         = 1,
	worldPartName = "Encounter_EchoLock",
	perception = {
		Blind         = "A voice repeats a word you cannot quite make out. It is asking something. Listen carefully and speak it back.",
		Delayed       = "You see a symbol carved into the lock — but it faded seconds ago. Describe the shape before it disappears.",
		Hallucinating = "The lock shows two symbols at once, overlapping. One is the answer. One is a trap. Ask the others what they see.",
	},
	rewards = { truth = 0.2, chaos = 0.0, illusion = 0.1, balance = 0.2 },
}

-- ─── STAGE 2: THE DROWNED ARCHIVE ───────────────────────────────────────────

EncounterData["Ink Current"] = {
	kind          = "Puzzle",
	stage         = 2,
	worldPartName = "Encounter_InkCurrent",
	perception = {
		Blind         = "Water whispers near your feet. The current pulls left. Something is written in the flow — read it aloud.",
		Delayed       = "Faded ink bleeds across a page you saw seconds ago. The words are already dissolving. What did they say?",
		Hallucinating = "The message is inverted — mirror-written. Hold it against what your team sees. One of you has the real version.",
	},
	rewards = { truth = 0.2, chaos = 0.0, illusion = 0.2, balance = 0.1 },
}

EncounterData["Drowned Scribe"] = {
	kind          = "Enemy",
	stage         = 2,
	worldPartName = "Encounter_DrownedScribe",
	perception = {
		Blind         = "Waterlogged footsteps approach from behind. They stop. Then start again. It knows you are here.",
		Delayed       = "A hunched figure stands at the desk — but your team already passed it. Where did it go after you lost sight?",
		Hallucinating = "The scribe splits into two at the moment it turns. One version writes. One version watches. Which one is moving?",
	},
	rewards = { truth = 0.0, chaos = 0.3, illusion = 0.1, balance = 0.1 },
}

EncounterData["Flooded Stacks"] = {
	kind          = "Obstacle",
	stage         = 2,
	worldPartName = "Encounter_FloodedStacks",
	perception = {
		Blind         = "Water rising — you can hear the level climbing the shelves. The safe path changes. Your team must guide you.",
		Delayed       = "You see the water at a low mark, but seconds have passed. The real level is higher. Warn your team of the old path.",
		Hallucinating = "The shelves flicker between flooded and dry. One state is real. Walk with caution and trust your team's reports.",
	},
	rewards = { truth = 0.1, chaos = 0.1, illusion = 0.2, balance = 0.1 },
}

EncounterData["Phantom Archive"] = {
	kind          = "Puzzle",
	stage         = 2,
	worldPartName = "Encounter_PhantomArchive",
	perception = {
		Blind         = "You hear pages turning — but no one is there. The sound forms a rhythm. Count with it. Three beats, then silence.",
		Delayed       = "A book lies open to a page you saw seconds ago. The content has changed. Tell the team what it said before.",
		Hallucinating = "Two books occupy the same shelf. One is real. One is a memory of a book. Only your team can tell them apart.",
	},
	rewards = { truth = 0.2, chaos = 0.0, illusion = 0.1, balance = 0.2 },
}

-- ─── SPECIAL: PHANTOM SIGNAL (spawned by Chaos Tier 1 threshold) ────────────
-- This encounter is not placed in stages — it is spawned dynamically near a player.

EncounterData["Phantom Signal"] = {
	kind          = "Puzzle",
	stage         = nil,  -- not tied to a stage; spawned dynamically
	worldPartName = nil,  -- created at runtime, not pre-placed
	perception = {
		Blind         = "You hear... nothing. A sound that was never there. It was not real.",
		Delayed       = "The signal was here — seconds ago. Or was it? There is no record of it arriving.",
		Hallucinating = "It shimmers and splits. Which one is real? Neither. Both. The question dissolves.",
	},
	rewards = { truth = 0.0, chaos = 0.0, illusion = 0.2, balance = 0.0 },
}

return EncounterData
