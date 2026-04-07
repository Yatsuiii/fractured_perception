# Fractured Perception — Roblox Studio Setup Guide

This guide explains how to get the scripts running in Roblox Studio.
All `.server.lua` files go in ServerScriptService, `.client.lua` in StarterPlayerScripts,
and plain `.lua` files are ModuleScripts.

---

## Option A: Use Rojo (recommended for ongoing development)

1. Install [Rojo](https://rojo.space/) plugin in Studio and the CLI tool.
2. Run `rojo serve` from the `roblox_src/` folder.
3. Click Sync in the Studio Rojo plugin.

The `default.project.json` file (you'll need to create one) maps the folder structure to Studio locations.

---

## Option B: Manual copy-paste (simpler for beginners)

1. Open Roblox Studio and create a new baseplate.
2. For each script below, right-click the correct service in the Explorer and choose Insert Object → Script/LocalScript/ModuleScript. Then open the file from this folder and paste its contents.

### Scripts to create

#### In ServerScriptService:
| File | Script type |
|---|---|
| `ServerScriptService/GameManager.server.lua` | Script |
| `ServerScriptService/EncounterManager.server.lua` | Script |
| `ServerScriptService/NPCController.server.lua` | Script |
| `ServerScriptService/PerceptionBroadcaster.server.lua` | Script |
| `ServerScriptService/ThresholdWatcher.server.lua` | Script |

Create a Folder named `Modules` inside ServerScriptService, then add:
| File | Script type |
|---|---|
| `ServerScriptService/Modules/GameState.lua` | ModuleScript |
| `ServerScriptService/Modules/PlayerRegistry.lua` | ModuleScript |
| `ServerScriptService/Modules/PositionHistory.lua` | ModuleScript |

#### In StarterPlayerScripts:
| File | Script type |
|---|---|
| `StarterPlayerScripts/RoleSetup.client.lua` | LocalScript |
| `StarterPlayerScripts/PerceptionRenderer.client.lua` | LocalScript |
| `StarterPlayerScripts/UIController.client.lua` | LocalScript |
| `StarterPlayerScripts/EncounterClient.client.lua` | LocalScript |
| `StarterPlayerScripts/DialogueClient.client.lua` | LocalScript |

#### In ReplicatedStorage:
Create a Folder named `Shared`, then inside it add these as ModuleScripts:
| File |
|---|
| `ReplicatedStorage/Shared/RoleData.lua` |
| `ReplicatedStorage/Shared/EncounterData.lua` |
| `ReplicatedStorage/Shared/StageData.lua` |
| `ReplicatedStorage/Shared/DialogueData.lua` |
| `ReplicatedStorage/Shared/ThresholdData.lua` |

Create a Folder named `RemoteEvents` inside ReplicatedStorage.
Inside it, right-click → Insert Object → RemoteEvent and create one for each name:
- AssignRole
- SyncGameState
- PerceptionUpdate
- EncounterInteract
- EncounterResolved
- OpenDialogue
- DialogueChoice
- CloseDialogue
- GateOpened
- StageAdvance
- ThresholdCrossed
- TeamLog

---

## Workspace Setup

Build your 3D stage inside Workspace with this structure:

```
Workspace/
├── Stage1/          (Model containing all Stage 1 geometry)
│   ├── Floor/       (folder of floor tile Parts)
│   │   ├── FloorTile_0_0  (Part, named in FloorTile_X_Z format)
│   │   ├── FloorTile_1_0
│   │   └── ...
│   ├── Walls/       (folder of wall tile Parts)
│   │   ├── WallTile_0_0
│   │   └── ...
│   ├── Encounters/  (folder of encounter marker Parts)
│   │   ├── Encounter_CrackedSequence   (Part with ProximityPrompt inside)
│   │   ├── Encounter_CollapsedArchway
│   │   ├── Encounter_Fragment
│   │   └── Encounter_EchoLock
│   └── NPCs/
│       ├── NPC_Watcher   (Model with HumanoidRootPart + ProximityPrompt)
│       └── NPC_Echo
├── Stage2/          (same structure as Stage1)
├── Spawn_Stage1     (Part — player spawn location for stage 1)
├── Spawn_Stage2     (Part — player spawn location for stage 2)
├── Gate_Stage1      (Part with ProximityPrompt, ActionText = "Cross the Gate")
└── Gate_Stage2      (Part with ProximityPrompt, ActionText = "Cross the Gate")
```

### Tagging floor and wall tiles

After building the tile Parts, you need to apply CollectionService tags so the
Hallucinating distortion system can find them.

Use the **Tag Editor** plugin in Studio (search for it in the Creator Marketplace):
1. Select all floor tile Parts → add tag `FloorTile`
2. Select all wall tile Parts → add tag `WallTile`
3. Select all stage geometry Parts → add tag `StageGeometry`

Alternatively, add a Script in ServerScriptService that auto-tags on startup:
```lua
local CollectionService = game:GetService("CollectionService")
for _, part in ipairs(workspace.Stage1.Floor:GetChildren()) do
    CollectionService:AddTag(part, "FloorTile")
end
for _, part in ipairs(workspace.Stage1.Walls:GetChildren()) do
    CollectionService:AddTag(part, "WallTile")
end
-- Repeat for Stage2
```

### NPC Attribute setup

For each NPC Model, set an Attribute named `NpcName` with the display name:
- `NPC_Watcher` → NpcName = "The Watcher"
- `NPC_Echo`    → NpcName = "The Echo"
- `NPC_Archivist` → NpcName = "The Archivist"

To set attributes: select the Model in Explorer → Properties → Attributes → + button.

---

## Testing

1. In Studio: Playtest → click the dropdown arrow next to Play → choose "2 Players" or "3 Players".
2. This opens multiple test clients.
3. Player 1 = Blind, Player 2 = Delayed, Player 3 = Hallucinating.

**To test with fewer than 3 players during development:**
Open `GameManager.server.lua` and change line:
```lua
if #joinedPlayers < 3 then
```
to:
```lua
if #joinedPlayers < 1 then  -- start immediately with 1 player
```

---

## Adjusting map size for distortion

If your map is not 80×30 tiles, update these values in `GameManager.server.lua`:
```lua
local MAP_WIDTH = 80   -- change to your actual tile grid width
local MAP_DEPTH = 30   -- change to your actual tile grid depth
```

And update the tile size in `RoleSetup.client.lua` if your tiles are not 4 studs wide:
```lua
local TILE_SIZE = 4   -- studs per tile
```
