//! progress.rs - Defines the ProgressReporter trait for decoupled
//! progress feedback. The library reports progress; the CLI decides how to display it.
//! This keeps the core library terminal-agnostic (ready for future TUI).

/// ProgressReporter is called periodically during encrypt/decrypt operations.
/// Implementors decide how to display progress (terminal bar, TUI widget, log, etc.).
pub trait ProgressReporter {
    /// Called after each chunk is processed.
    /// * `bytes_done` - Total bytes processed so far.
    /// * `total_bytes` - Total bytes expected (0 if unknown).
    fn report(&self, bytes_done: u64, total_bytes: u64);

    /// Called when the operation completes successfully.
    fn finish(&self);
}

/// NoopReporter does nothing. Used when no progress display is desired (e.g., piping).
pub struct NoopReporter;

impl ProgressReporter for NoopReporter {
    /// Does nothing. Silent operation.
    fn report(&self, _bytes_done: u64, _total_bytes: u64) {}

    /// Does nothing. Silent operation.
    fn finish(&self) {}
}
