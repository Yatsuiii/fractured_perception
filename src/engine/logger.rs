use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    time::Instant,
};

/// Writes a timestamped session log to `logs/session_<unix>.log` on disk.
/// One file is created per run; entries are flushed incrementally.
/// If the log file cannot be created (disk full, read-only fs), the logger
/// degrades silently — all writes become no-ops.
pub struct SessionLogger {
    writer: Option<BufWriter<File>>,
    start:  Instant,
}

impl SessionLogger {
    /// Creates the `logs/` directory if absent and opens a new log file.
    /// Returns a disabled logger on any I/O failure instead of panicking.
    pub fn new() -> Self {
        let writer = Self::try_open();
        let mut logger = Self { writer, start: Instant::now() };
        logger.write_line("SESSION START");
        logger
    }

    /// Writes a single timestamped event line.
    /// Leading/trailing whitespace is stripped so display-indent doesn't leak into the file.
    pub fn log(&mut self, text: &str) {
        self.write_line(text.trim());
    }

    /// Writes the SESSION END summary and flushes all buffered bytes.
    pub fn finish(&mut self, puzzles_solved: usize, total: usize) {
        let msg = format!("SESSION END — {}/{} puzzles solved", puzzles_solved, total);
        self.write_line(&msg);
        if let Some(w) = self.writer.as_mut() {
            let _ = w.flush();
        }
    }

    // --- private ---

    fn try_open() -> Option<BufWriter<File>> {
        fs::create_dir_all("logs").ok()?;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let path = format!("logs/session_{}.log", ts);
        let file = File::create(&path).ok()?;
        Some(BufWriter::new(file))
    }

    fn write_line(&mut self, text: &str) {
        if let Some(w) = self.writer.as_mut() {
            let elapsed = self.start.elapsed().as_secs_f32();
            let mins    = (elapsed / 60.0) as u32;
            let secs    = elapsed % 60.0;
            let _ = writeln!(w, "[{:02}:{:06.3}] {}", mins, secs, text);
        }
    }
}
