/// Timeline operations: home, mentions, and user timelines
pub mod commands;
pub mod models;

pub use commands::{HttpTimelineClient, TimelineClient, TimelineCommand, TimelineError};
pub use models::{TimelineArgs, TimelineKind, TimelineMeta, TimelineResult};

// Mock client is only available in tests
#[cfg(test)]
pub use commands::MockTimelineClient;
