use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct BatchChecksumsRequestDto {
    pub checksums: Vec<String>,
}
