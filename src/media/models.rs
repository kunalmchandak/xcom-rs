use serde::{Deserialize, Serialize};

/// Result of a media upload operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResult {
    /// The media ID returned by the X API
    pub media_id: String,
}

impl UploadResult {
    pub fn new(media_id: impl Into<String>) -> Self {
        Self {
            media_id: media_id.into(),
        }
    }
}
