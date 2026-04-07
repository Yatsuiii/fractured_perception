-- RoleData.lua (ModuleScript)
-- Defines the three roles available in Fractured Perception.
-- Other scripts require() this to check role names and glyph colors.

local RoleData = {}

-- The three role names. Used everywhere as string keys.
RoleData.Roles = {
	"Blind",
	"Delayed",
	"Hallucinating",
}

-- Short descriptions shown during role assignment screen.
RoleData.Descriptions = {
	Blind         = "You cannot see — but you can hear everything.",
	Delayed       = "You see the truth, but always three seconds too late.",
	Hallucinating = "The world shifts around you. Trust nothing at face value.",
}

-- BrickColor used for each role's UI accent
RoleData.UIColor = {
	Blind         = Color3.fromRGB(100, 200, 255),  -- light blue
	Delayed       = Color3.fromRGB(255, 220, 80),   -- amber
	Hallucinating = Color3.fromRGB(200, 100, 255),  -- purple
}

return RoleData
