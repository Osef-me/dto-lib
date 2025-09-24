use serde::Serialize;
use utoipa::ToSchema;
use serde_json::Value;

#[derive(Serialize, Debug, Clone, ToSchema)]
pub struct Beatmapset {
    pub osu_id: Option<i32>,
    pub artist: String,
    pub title: String,
    pub creator: String,
    pub cover_url: Option<String>,
    pub total_beatmaps: i32,
    pub beatmaps: Vec<Beatmap>,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
pub struct Rating {
    pub rating: f64,
    pub rating_type: String,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
pub struct Beatmap {
    pub osu_id: Option<i32>,
    pub difficulty: String,
    pub mode: i32,
    pub status: String,
    pub main_pattern: Value,
    pub ratings: Vec<Rating>,
}
