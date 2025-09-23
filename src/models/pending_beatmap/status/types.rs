use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct PendingStatusDto {
    pub position: i64,
    pub total: i64,
}


