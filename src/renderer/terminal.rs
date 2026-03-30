use std::io::{self, Write};

use crossterm::{
    cursor,
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};

use crate::{
    perception::{CellColor, EntityColor, PanelColor, PlayerView},
    state::GameState,
};

use super::{RenderError, Renderer};

// --- Layout constants ---
const PANEL_SEP_COL: u16   = 81; // '│' separator column
const PANEL_START_COL: u16 = 83; // side panel text start
const MAP_ROW_OFFSET: u16  = 1;  // row 0 = HUD, map starts at row 1
const HUD_WIDTH: usize     = 110; // total HUD separator width in columns
const ROLE_COL: u16        = 24; // column where the active role name appears
const KEYBIND_COL: u16     = 85; // column where the keybind hint begins

pub struct TerminalRenderer {
    stdout:   io::Stdout,
    hud_line: String, // pre-built once — avoids per-frame heap allocation
}

impl TerminalRenderer {
    pub fn new() -> Self {
        Self {
            stdout:   io::stdout(),
            hud_line: "─".repeat(HUD_WIDTH),
        }
    }
}

impl Renderer for TerminalRenderer {
    fn init(&mut self) -> Result<(), RenderError> {
        terminal::enable_raw_mode()?;
        execute!(self.stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
        Ok(())
    }

    fn clear(&mut self) -> Result<(), RenderError> {
        execute!(self.stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        Ok(())
    }

    /// All draw helpers use `queue!` (no flush). One flush at the end of this
    /// function pushes the complete frame at once, eliminating mid-render stutter.
    fn draw_view(&mut self, state: &GameState, view: &PlayerView) -> Result<(), RenderError> {
        match state {
            GameState::MainMenu   => self.draw_main_menu()?,
            GameState::RoleSelect => self.draw_role_select()?,
            GameState::Playing    => self.draw_playing(view)?,
            GameState::Paused     => { self.draw_playing(view)?; self.draw_pause_overlay()?; }
            GameState::GameOver   => self.draw_game_over()?,
        }
        self.stdout.flush()?;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), RenderError> {
        execute!(self.stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
}

impl TerminalRenderer {
    // --- Static screens ---

    fn draw_main_menu(&mut self) -> Result<(), RenderError> {
        queue!(
            self.stdout,
            cursor::MoveTo(2, 2), SetForegroundColor(Color::White),
            Print("╔══════════════════════════╗"),
            cursor::MoveTo(2, 3), Print("║   FRACTURED PERCEPTION   ║"),
            cursor::MoveTo(2, 4), Print("╚══════════════════════════╝"),
            ResetColor,
            cursor::MoveTo(2, 6), Print("  [Enter]  Begin"),
            cursor::MoveTo(2, 7), Print("  [Space]  Cycle player view"),
            cursor::MoveTo(2, 8), Print("  [Q]      Quit"),
        )?;
        Ok(())
    }

    fn draw_role_select(&mut self) -> Result<(), RenderError> {
        queue!(
            self.stdout,
            cursor::MoveTo(2, 2), SetForegroundColor(Color::White),
            Print("╔══════════════════════════════╗"),
            cursor::MoveTo(2, 3), Print("║       CHOOSE YOUR ROLE       ║"),
            cursor::MoveTo(2, 4), Print("╚══════════════════════════════╝"),
            ResetColor,
            cursor::MoveTo(2, 6), SetForegroundColor(Color::DarkGrey),
            Print("  [1]  THE BLIND"),
            cursor::MoveTo(2, 7), Print("       Navigate by sound alone."),
            cursor::MoveTo(2, 8), Print("       You hear what others cannot."),
            ResetColor,
            cursor::MoveTo(2, 10), SetForegroundColor(Color::Cyan),
            Print("  [2]  THE VISUAL ANALYST"),
            cursor::MoveTo(2, 11), Print("       You see everything."),
            cursor::MoveTo(2, 12), Print("       Half of it is a lie."),
            ResetColor,
            cursor::MoveTo(2, 14), SetForegroundColor(Color::DarkYellow),
            Print("  [3]  THE HALLUCINATING"),
            cursor::MoveTo(2, 15), Print("       Reality bends around you."),
            cursor::MoveTo(2, 16), Print("       Trust nothing you see."),
            ResetColor,
            cursor::MoveTo(2, 18), SetForegroundColor(Color::DarkGrey),
            Print("  [Esc] Back    [Q] Quit"),
            ResetColor,
        )?;
        Ok(())
    }

    fn draw_game_over(&mut self) -> Result<(), RenderError> {
        queue!(
            self.stdout,
            cursor::MoveTo(2, 3), SetForegroundColor(Color::Green),
            Print("╔══════════════════════════╗"),
            cursor::MoveTo(2, 4), Print("║   ALL PUZZLES  SOLVED   ║"),
            cursor::MoveTo(2, 5), Print("╚══════════════════════════╝"),
            ResetColor,
            cursor::MoveTo(2, 7), SetForegroundColor(Color::DarkGrey),
            Print("  Reality has been restored."),
            ResetColor,
            cursor::MoveTo(2, 9), Print("  [Enter]  Play Again"),
            cursor::MoveTo(2, 10), Print("  [Q]      Quit"),
        )?;
        Ok(())
    }

    // --- Playing state ---

    fn draw_playing(&mut self, view: &PlayerView) -> Result<(), RenderError> {
        self.draw_hud(view)?;
        self.draw_map(view)?;
        self.draw_entities(view)?;
        self.draw_separator(view)?;
        self.draw_side_panel(view)?;
        Ok(())
    }

    fn draw_hud(&mut self, view: &PlayerView) -> Result<(), RenderError> {
        queue!(
            self.stdout,
            cursor::MoveTo(0, 0),
            SetForegroundColor(Color::DarkGrey), Print(&self.hud_line),
            cursor::MoveTo(2, 0),
            SetForegroundColor(Color::White),    Print("FRACTURED PERCEPTION"),
            cursor::MoveTo(ROLE_COL, 0),
            SetForegroundColor(role_color(view.role)), Print(view.role.name()),
            cursor::MoveTo(KEYBIND_COL, 0),
            SetForegroundColor(Color::DarkGrey), Print("[Space] Switch  [Esc] Pause  [Q] Quit"),
            ResetColor,
        )?;
        Ok(())
    }

    fn draw_map(&mut self, view: &PlayerView) -> Result<(), RenderError> {
        for y in 0..view.map_height {
            for x in 0..view.map_width {
                let cell = &view.cells[y * view.map_width + x];
                let (color, glyph) = cell_appearance(cell.glyph, cell.color);
                queue!(
                    self.stdout,
                    cursor::MoveTo(x as u16, y as u16 + MAP_ROW_OFFSET),
                    SetForegroundColor(color),
                    Print(glyph),
                )?;
            }
        }
        // Single reset after the entire map — not once per cell.
        queue!(self.stdout, ResetColor)?;
        Ok(())
    }

    fn draw_entities(&mut self, view: &PlayerView) -> Result<(), RenderError> {
        for e in &view.entities {
            // Entities whose position falls inside the separator / panel are skipped.
            if e.col >= PANEL_SEP_COL { continue; }
            queue!(
                self.stdout,
                cursor::MoveTo(e.col, e.row + MAP_ROW_OFFSET),
                SetForegroundColor(entity_color(e.color)),
                Print(e.glyph),
            )?;
        }
        queue!(self.stdout, ResetColor)?;
        Ok(())
    }

    fn draw_separator(&mut self, view: &PlayerView) -> Result<(), RenderError> {
        for row in MAP_ROW_OFFSET..MAP_ROW_OFFSET + view.map_height as u16 {
            queue!(
                self.stdout,
                cursor::MoveTo(PANEL_SEP_COL, row),
                SetForegroundColor(Color::DarkGrey),
                Print("│"),
            )?;
        }
        queue!(self.stdout, ResetColor)?;
        Ok(())
    }

    fn draw_side_panel(&mut self, view: &PlayerView) -> Result<(), RenderError> {
        // `.take(map_height)` clamps panel output to the visible map area cleanly.
        for (i, line) in view.panel_lines.iter().take(view.map_height).enumerate() {
            queue!(
                self.stdout,
                cursor::MoveTo(PANEL_START_COL, i as u16 + MAP_ROW_OFFSET),
                SetForegroundColor(panel_color(line.color)),
                Print(&line.text),
            )?;
        }
        queue!(self.stdout, ResetColor)?;
        Ok(())
    }

    // --- Overlays ---

    fn draw_pause_overlay(&mut self) -> Result<(), RenderError> {
        queue!(
            self.stdout,
            cursor::MoveTo(4, 4), SetForegroundColor(Color::White),
            Print("┌─────────────────┐"),
            cursor::MoveTo(4, 5), Print("│     PAUSED      │"),
            cursor::MoveTo(4, 6), Print("│  [Esc] Resume   │"),
            cursor::MoveTo(4, 7), Print("│  [Q]   Quit     │"),
            cursor::MoveTo(4, 8), Print("└─────────────────┘"),
            ResetColor,
        )?;
        Ok(())
    }
}

// --- Color mapping ---

fn cell_appearance(glyph: char, color: CellColor) -> (Color, char) {
    match color {
        CellColor::Hidden     => (Color::Reset,      ' '),
        CellColor::Memory     => (Color::DarkGrey,   glyph),
        CellColor::Floor      => (Color::DarkGrey,   glyph),
        CellColor::Wall       => (Color::White,      glyph),
        CellColor::Door       => (Color::DarkYellow, glyph),
        CellColor::Fabricated => (Color::DarkGrey,   glyph), // identical to floor — intentional
        CellColor::Distorted  => (Color::DarkYellow, glyph),
    }
}

fn entity_color(ec: EntityColor) -> Color {
    match ec {
        EntityColor::Self_      => Color::Yellow,
        EntityColor::Ally       => Color::Cyan,
        EntityColor::Npc        => Color::Green,
        EntityColor::NpcDoubt   => Color::DarkYellow,
        EntityColor::Ghost      => Color::DarkRed,
        EntityColor::AuraTrust  => Color::Green,
        EntityColor::AuraDoubt  => Color::DarkRed,
        EntityColor::AuraAlly   => Color::DarkCyan,
        EntityColor::AuraPuzzle => Color::DarkYellow,
    }
}

fn panel_color(pc: PanelColor) -> Color {
    match pc {
        PanelColor::White    => Color::White,
        PanelColor::Grey     => Color::Grey,
        PanelColor::DarkGrey => Color::DarkGrey,
        PanelColor::Yellow   => Color::Yellow,
        PanelColor::Red      => Color::Red,
        PanelColor::Green    => Color::Green,
        PanelColor::Cyan     => Color::Cyan,
    }
}

fn role_color(role: crate::player::Role) -> Color {
    use crate::player::Role;
    match role {
        Role::Blind         => Color::DarkGrey,
        Role::VisualAnalyst => Color::Cyan,
        Role::Hallucinating => Color::DarkYellow,
    }
}
