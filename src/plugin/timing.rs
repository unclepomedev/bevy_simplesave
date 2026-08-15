/// When a registered resource should be written back to disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveTiming {
    /// Save automatically whenever the resource changes.
    Auto,
    /// Only save when save_now is called explicitly.
    Manual,
}
