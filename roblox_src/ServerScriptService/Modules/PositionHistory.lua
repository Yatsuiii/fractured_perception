-- PositionHistory.lua (ModuleScript — server only)
-- Maintains a circular buffer of entity positions over time.
-- Used by PerceptionBroadcaster to supply stale positions to the Delayed role.
--
-- How it works:
--   1. PerceptionBroadcaster calls record() for each character every ~0.1 seconds.
--   2. When building a Delayed player's PerceptionUpdate, call getDelayed(entity, 3.0)
--      to retrieve where the entity was 3 seconds ago.

local PositionHistory = {}

-- Max entries stored per entity (10 seconds at ~20 samples/sec = 200 entries)
local MAX_ENTRIES = 200

-- Internal storage: { [entity] = { {time=t, pos=Vector3}, ... } }
-- "entity" can be a Player object or an NPC Model reference — anything works as a key.
local _history = {}

-- ─── RECORD ──────────────────────────────────────────────────────────────────

-- Record the current position of an entity right now.
-- Call this from a Heartbeat/loop in PerceptionBroadcaster.
function PositionHistory.record(entity, position)
	if _history[entity] == nil then
		_history[entity] = {}
	end

	local buffer = _history[entity]

	-- Append the new sample
	table.insert(buffer, {
		time = os.clock(),
		pos  = position,
	})

	-- Trim to maximum size (remove oldest entries from the front)
	while #buffer > MAX_ENTRIES do
		table.remove(buffer, 1)
	end
end

-- ─── QUERY ───────────────────────────────────────────────────────────────────

-- Returns the position of entity at (now - delaySeconds) seconds ago.
-- If the history does not go back that far, returns the oldest known position.
-- Returns nil if no history exists for this entity.
function PositionHistory.getDelayed(entity, delaySeconds)
	local buffer = _history[entity]
	if buffer == nil or #buffer == 0 then
		return nil
	end

	local targetTime = os.clock() - delaySeconds

	-- Walk backwards through the buffer to find the most recent entry
	-- that is still older than (now - delaySeconds)
	for i = #buffer, 1, -1 do
		if buffer[i].time <= targetTime then
			return buffer[i].pos
		end
	end

	-- All entries are newer than the target — return the oldest we have
	return buffer[1].pos
end

-- ─── CLEANUP ─────────────────────────────────────────────────────────────────

-- Call this when a player leaves or an NPC is removed to free memory
function PositionHistory.clear(entity)
	_history[entity] = nil
end

return PositionHistory
