-- DialogueData.lua (ModuleScript)
-- NPC dialogue lines organized by NPC name, then role, then trust tier.
-- Trust tiers: "Low" (< 0.35), "Mid" (0.35–0.7), "High" (>= 0.7)
--
-- Each line entry:
--   text       = string shown to the player
--   trustDelta = how much trust changes after this line (positive or negative)
--   statNudge  = { truth, chaos, illusion, balance } added to hidden state

local DialogueData = {}

-- ─── THE WATCHER (Stage 1) ────────────────────────────────────────────────────

DialogueData["The Watcher"] = {

	Blind = {
		Low = {
			{ text = "You walk loudly for someone with no eyes. The others will leave you behind.", trustDelta = -0.02, statNudge = {0, 0.05, 0, 0} },
			{ text = "I have nothing for someone who does not listen.", trustDelta = -0.01, statNudge = {0, 0.03, 0, 0} },
		},
		Mid = {
			{ text = "Sound is the oldest map. You understand this more than the others.", trustDelta = 0.02, statNudge = {0.02, 0, 0, 0} },
			{ text = "There are four encounters in this hall. You have found them by sound alone.", trustDelta = 0.03, statNudge = {0.03, 0, 0, 0} },
			{ text = "The gate hums at a low frequency. Listen for it and you will find your way.", trustDelta = 0.02, statNudge = {0.02, 0, 0, 0.01} },
		},
		High = {
			{ text = "You are closer to the truth than the others. Sound does not lie.", trustDelta = 0.05, statNudge = {0.05, 0, 0, 0} },
			{ text = "The last tile is in the terminus. The echo there is... different.", trustDelta = 0.03, statNudge = {0.03, 0, 0, 0} },
			{ text = "Trust what you hear. Always.", trustDelta = 0.02, statNudge = {0, 0, 0, 0.02} },
		},
	},

	Delayed = {
		Low = {
			{ text = "You are always late. Even to this conversation.", trustDelta = -0.02, statNudge = {0, 0.03, 0.02, 0} },
			{ text = "Looking at the past will not help anyone here.", trustDelta = -0.01, statNudge = {0, 0.02, 0.03, 0} },
		},
		Mid = {
			{ text = "What you see is still true. It is simply old. That has value.", trustDelta = 0.02, statNudge = {0.02, 0, 0, 0} },
			{ text = "You can predict the pattern. The others cannot. Use that.", trustDelta = 0.03, statNudge = {0.03, 0, 0, 0} },
			{ text = "The creature you saw near the east wall — it has moved, but not far.", trustDelta = 0.02, statNudge = {0.02, 0, 0, 0.01} },
		},
		High = {
			{ text = "You understand that the past is a map. Few do.", trustDelta = 0.05, statNudge = {0.05, 0, 0, 0} },
			{ text = "The gate mechanism was triggered moments ago. Check the eastern room.", trustDelta = 0.03, statNudge = {0.03, 0, 0, 0} },
			{ text = "Keep watching. Keep reporting. You are the team's memory.", trustDelta = 0.02, statNudge = {0, 0, 0, 0.02} },
		},
	},

	Hallucinating = {
		Low = {
			{ text = "I cannot tell which version of you I am speaking to.", trustDelta = -0.02, statNudge = {0, 0, 0.05, 0} },
			{ text = "Your eyes betray you. And they betray your team as well.", trustDelta = -0.01, statNudge = {0, 0, 0.04, 0} },
		},
		Mid = {
			{ text = "What you see that is wrong can still tell you what is right.", trustDelta = 0.02, statNudge = {0.01, 0, 0, 0.02} },
			{ text = "The real path hides between the two you see. Find the overlap.", trustDelta = 0.03, statNudge = {0.02, 0, 0, 0.01} },
			{ text = "The ghost copies move differently from the real. Watch the feet.", trustDelta = 0.02, statNudge = {0.02, 0, 0, 0} },
		},
		High = {
			{ text = "You are seeing more than the others. That is not a curse.", trustDelta = 0.05, statNudge = {0.02, 0, 0, 0.03} },
			{ text = "The real gate does not shimmer. It is solid. You will know it.", trustDelta = 0.03, statNudge = {0.03, 0, 0, 0} },
			{ text = "Hold both versions and choose neither. That is wisdom.", trustDelta = 0.02, statNudge = {0, 0, 0, 0.05} },
		},
	},
}

-- ─── THE ECHO (Stages 1 & 2) ─────────────────────────────────────────────────

DialogueData["The Echo"] = {

	Blind = {
		Low = {
			{ text = "...", trustDelta = 0, statNudge = {0, 0, 0.02, 0} },
			{ text = "I repeat what I hear. And I hear nothing useful from you.", trustDelta = -0.01, statNudge = {0, 0.02, 0, 0} },
		},
		Mid = {
			{ text = "There was a sound — east, two rooms. Low hum. I heard it before you arrived.", trustDelta = 0.02, statNudge = {0.02, 0, 0, 0} },
			{ text = "The others make noise when they are afraid. Listen for changes in their rhythm.", trustDelta = 0.03, statNudge = {0.02, 0, 0, 0.01} },
		},
		High = {
			{ text = "I echo what the walls remember. They remember a path north.", trustDelta = 0.04, statNudge = {0.04, 0, 0, 0} },
			{ text = "The sound shifts before the encounter activates. You will feel it first.", trustDelta = 0.03, statNudge = {0.03, 0, 0, 0} },
		},
	},

	Delayed = {
		Low = {
			{ text = "You see echoes of the past. I am an echo too. We are the same.", trustDelta = 0.01, statNudge = {0, 0, 0.03, 0} },
			{ text = "But you are late. Again.", trustDelta = -0.01, statNudge = {0, 0.02, 0, 0} },
		},
		Mid = {
			{ text = "Seconds ago, something passed here. You would have seen it.", trustDelta = 0.02, statNudge = {0.02, 0, 0, 0} },
			{ text = "I repeat what others say. What they said before was: go left.", trustDelta = 0.03, statNudge = {0.02, 0, 0, 0.01} },
		},
		High = {
			{ text = "You and I both carry the past. But I carry it more honestly.", trustDelta = 0.04, statNudge = {0.03, 0, 0, 0.01} },
			{ text = "Three seconds back, the encounter to the west was just activated.", trustDelta = 0.04, statNudge = {0.04, 0, 0, 0} },
		},
	},

	Hallucinating = {
		Low = {
			{ text = "Which version of me are you speaking to?", trustDelta = -0.01, statNudge = {0, 0, 0.04, 0} },
			{ text = "I am already two things at once. You make me three.", trustDelta = -0.01, statNudge = {0, 0, 0.03, 0} },
		},
		Mid = {
			{ text = "The ghost of me and the real me agree: go north.", trustDelta = 0.02, statNudge = {0.01, 0, 0, 0.02} },
			{ text = "I see the doubles you see. I know which is real. Ask me.", trustDelta = 0.03, statNudge = {0.02, 0, 0, 0.01} },
		},
		High = {
			{ text = "You see two of everything. I say: listen to neither. Find the third option.", trustDelta = 0.04, statNudge = {0.01, 0, 0, 0.04} },
			{ text = "The encounter to the south is real. The one to the north is an echo of it.", trustDelta = 0.04, statNudge = {0.03, 0, 0, 0.01} },
		},
	},
}

-- ─── THE ARCHIVIST (Stage 2) ─────────────────────────────────────────────────

DialogueData["The Archivist"] = {

	Blind = {
		Low = {
			{ text = "The water rises. You will not hear it until it reaches your knees.", trustDelta = -0.01, statNudge = {0, 0.03, 0, 0} },
			{ text = "This is not a place for the sightless.", trustDelta = -0.02, statNudge = {0, 0.04, 0, 0} },
		},
		Mid = {
			{ text = "The ink speaks through sound too. Dripping. Listen to the pattern.", trustDelta = 0.02, statNudge = {0.02, 0, 0, 0} },
			{ text = "Two rooms north, the shelves are still dry. I heard someone else say so moments ago.", trustDelta = 0.03, statNudge = {0.02, 0, 0, 0.01} },
		},
		High = {
			{ text = "You have mapped this place better with your ears than I have with my eyes.", trustDelta = 0.05, statNudge = {0.05, 0, 0, 0} },
			{ text = "The gate is sealed by a sound lock. Your team must produce all three tones.", trustDelta = 0.03, statNudge = {0.03, 0, 0, 0} },
		},
	},

	Delayed = {
		Low = {
			{ text = "You are looking at records. I have spent my life doing the same. Neither of us knows what is happening now.", trustDelta = 0.01, statNudge = {0, 0, 0.02, 0.01} },
			{ text = "Old maps lead into walls that have since been built.", trustDelta = -0.01, statNudge = {0, 0.02, 0.02, 0} },
		},
		Mid = {
			{ text = "What you saw seconds ago is still useful. The Scribe had not moved from the desk yet.", trustDelta = 0.02, statNudge = {0.02, 0, 0, 0} },
			{ text = "The ink on the western shelf was legible three seconds ago. Did you read it?", trustDelta = 0.03, statNudge = {0.03, 0, 0, 0} },
		},
		High = {
			{ text = "You see the archive as it was. I see it as it is. Together we see it whole.", trustDelta = 0.05, statNudge = {0.04, 0, 0, 0.01} },
			{ text = "Tell me: three seconds back, was the northern passage open?", trustDelta = 0.03, statNudge = {0.02, 0, 0, 0.01} },
		},
	},

	Hallucinating = {
		Low = {
			{ text = "You see books that were burned. You see shelves that have not been built. You are useless here.", trustDelta = -0.02, statNudge = {0, 0, 0.05, 0} },
			{ text = "Do not touch anything. You will confuse the collection.", trustDelta = -0.01, statNudge = {0, 0, 0.03, 0} },
		},
		Mid = {
			{ text = "The false shelves shimmer. Real books do not. You know this.", trustDelta = 0.02, statNudge = {0.01, 0, 0, 0.02} },
			{ text = "The Scribe you see doubled — the one writing is the real one. The watching one is a memory.", trustDelta = 0.03, statNudge = {0.02, 0, 0, 0.01} },
		},
		High = {
			{ text = "You are the only one who sees both the archive as it is and as it was. That is not madness. That is perspective.", trustDelta = 0.05, statNudge = {0.02, 0, 0, 0.04} },
			{ text = "The real exit is behind the false wall on the east. Walk through it.", trustDelta = 0.04, statNudge = {0.03, 0, 0, 0.01} },
		},
	},
}

return DialogueData
