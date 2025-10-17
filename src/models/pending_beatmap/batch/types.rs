use serde::Deserialize;
use utoipa::ToSchema;

/// Batch import request of beatmaps by osu! checksums.
///
/// A "checksum" refers to the beatmap file hash (e.g., MD5)
/// used by osu! to identify a `.osu` file.
#[derive(Deserialize, Debug, Clone, ToSchema)]
#[schema(
    title = "BatchChecksumsRequest",
    description = "Import beatmaps using a list of osu! checksums"
)]
pub struct BatchChecksumsRequestDto {
    /// List of osu! checksums to enqueue for processing.
    ///
    /// Each item is a non-empty hexadecimal string representing
    /// the hash of a `.osu` file.
    #[schema(
        min_items = 1,
        example = json!([
            "d41d8cd98f00b204e9800998ecf8427e",
            "e2fc714c4727ee9395f324cd2e7f331f"
        ])
    )]
    pub checksums: Vec<String>,
}
