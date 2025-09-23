use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Current position in the queue and total size.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(title = "PendingStatus", description = "Status of a pending beatmap in the queue")]
pub struct PendingStatusDto {
    /// Beatmap position in the queue (1 = first).
    #[schema(example = 3, minimum = 1)]
    pub position: i64,

    /// Total number of items in the queue.
    #[schema(example = 42, minimum = 0)]
    pub total: i64,
}


