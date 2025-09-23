use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, Debug, Clone, ToSchema)]
pub struct ApiResponse<T> {
    pub message: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn ok(message: impl Into<String>, data: Option<T>) -> Self {
        Self {
            message: message.into(),
            status: "200".to_string(),
            data,
        }
    }

    pub fn error(status: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status: status.into(),
            data: None,
        }
    }
}

#[derive(Serialize, Debug, Clone, ToSchema)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
pub struct PaginatedResponse<T> {
    pub message: String,
    pub status: String,
    pub data: Vec<T>,
    pub pagination: Pagination,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
pub struct Empty;
