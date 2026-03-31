use std::io::{self, Write};

use crossterm::{
    cursor,
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};

use crate::{
    dialogue::DialogueSession,
    perception::{CellColor, EntityColor, PanelColor, PlayerView},
    stage::StageTheme,
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
            GameState::Dialogue        => self.draw_playing(view)?,
            GameState::StageTransition => {}, // drawn separately via draw_stage_transition
            GameState::GameOver        => self.draw_game_over()?,
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
            cursor::MoveTo(2, 10), SetForegroundColor(Color::DarkBlue),
            Print("  [2]  THE DELAYED"),
            cursor::MoveTo(2, 11), Print("       You see the past."),
            cursor::MoveTo(2, 12), Print("       Reality has moved on."),
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

    /// Draws the stage transition screen — shown between stages.
    pub fn draw_stage_transition(&mut self, theme: StageTheme) -> Result<(), RenderError> {
        let stage_num = format!("STAGE {}", theme.stage_number());
        let name = theme.name();
        let desc = theme.description();

        queue!(
            self.stdout,
            cursor::MoveTo(0, 0),
            terminal::Clear(ClearType::All),
        )?;

        queue!(
            self.stdout,
            cursor::MoveTo(30, 8),
            SetForegroundColor(Color::DarkGrey),
            Print(&stage_num),
            cursor::MoveTo(2, 11),
            SetForegroundColor(Color::White),
            Print("╔══════════════════════════════════════════════════════════════╗"),
            cursor::MoveTo(2, 12),
        )?;

        // Center the name inside the box.
        let pad = (60_i32 - name.len() as i32) / 2;
        let padded_name = format!("║{:>w$}{}{:<r$}║", "", name, "", w = pad as usize, r = (60 - pad as usize - name.len()));
        queue!(
            self.stdout,
            Print(&padded_name),
            cursor::MoveTo(2, 13),
            SetForegroundColor(Color::White),
            Print("╚══════════════════════════════════════════════════════════════╝"),
        )?;

        // Description.
        let desc_pad = (64_i32 - desc.len() as i32) / 2;
        queue!(
            self.stdout,
            cursor::MoveTo(desc_pad.max(2) as u16, 16),
            SetForegroundColor(Color::DarkGrey),
            Print(desc),
        )?;

        queue!(
            self.stdout,
            cursor::MoveTo(22, 22),
            SetForegroundColor(Color::DarkGrey),
            Print("[Enter] Begin          [Q] Quit"),
            ResetColor,
        )?;

        self.stdout.flush()?;
        Ok(())
    }

    /// Draws a dialogue box overlay at the bottom of the map area.
    /// Shows the NPC name, current line of dialogue, and navigation hints.
    pub fn draw_dialogue_overlay(&mut self, session: &DialogueSession) -> Result<(), RenderError> {
        // Dialogue box dimensions and position.
        let box_width: u16 = 60;
        let box_left: u16 = 2;
        let box_top: u16 = 26; // near the bottom of the 34-row map

        // Build the content lines.
        let npc_label = format!("  {} says:", session.npc_name);

        let dialogue_text = match session.current() {
            Some(line) => line.text,
            None => "...",
        };

        // Word-wrap the dialogue text to fit inside the box.
        let inner_width = (box_width - 4) as usize;
        let wrapped = word_wrap(dialogue_text, inner_width);

        let is_last = session.current_line + 1 >= session.lines.len();
        let hint = if is_last {
            "[E/Enter] Close    [Esc] Leave"
        } else {
            "[E/Enter] Continue    [Esc] Leave"
        };

        // Calculate box height: top border + name + blank + wrapped lines + blank + hint + bottom border.
        // Draw the box border and content.
        let top_border = format!("┌{}┐", "─".repeat(box_width as usize - 2));
        let bot_border = format!("└{}┘", "─".repeat(box_width as usize - 2));
        let empty_line = format!("│{}│", " ".repeat(box_width as usize - 2));

        // Top border.
        queue!(
            self.stdout,
            cursor::MoveTo(box_left, box_top),
            SetForegroundColor(Color::White),
            Print(&top_border),
        )?;

        // NPC name line.
        let name_padded = format!("│{:<width$}│", npc_label, width = box_width as usize - 2);
        queue!(
            self.stdout,
            cursor::MoveTo(box_left, box_top + 1),
            SetForegroundColor(Color::Green),
            Print(&name_padded),
        )?;

        // Blank separator.
        queue!(
            self.stdout,
            cursor::MoveTo(box_left, box_top + 2),
            SetForegroundColor(Color::White),
            Print(&empty_line),
        )?;

        // Dialogue text lines.
        for (i, text_line) in wrapped.iter().enumerate() {
            let padded = format!("│  {:<width$}│", text_line, width = box_width as usize - 4);
            queue!(
                self.stdout,
                cursor::MoveTo(box_left, box_top + 3 + i as u16),
                SetForegroundColor(Color::White),
                Print(&padded),
            )?;
        }

        // Hint line.
        let hint_row = box_top + 3 + wrapped.len() as u16;
        let hint_padded = format!("│  {:<width$}│", hint, width = box_width as usize - 4);
        queue!(
            self.stdout,
            cursor::MoveTo(box_left, hint_row),
            SetForegroundColor(Color::DarkGrey),
            Print(&hint_padded),
        )?;

        // Bottom border.
        queue!(
            self.stdout,
            cursor::MoveTo(box_left, hint_row + 1),
            SetForegroundColor(Color::White),
            Print(&bot_border),
            ResetColor,
        )?;

        // Extra flush for the overlay.
        self.stdout.flush()?;
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
        Role::Delayed       => Color::DarkBlue,
        Role::Hallucinating => Color::DarkYellow,
    }
}

/// Wraps text at word boundaries to fit within the given width.
fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() > max_width {
            lines.push(current);
            current = word.to_string();
        } else {
            current.push(' ');
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::from("..."));
    }

    lines
}
