use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Beatmapset {
    pub id: i32,
    pub osu_id: Option<i32>,
    pub artist: String,
    pub artist_unicode: Option<String>,
    pub title: String,
    pub title_unicode: Option<String>,
    pub creator: String,
    pub source: Option<String>,
    pub tags: Option<Vec<String>>,
    pub has_video: bool,
    pub has_storyboard: bool,
    pub is_explicit: bool,
    pub is_featured: bool,
    pub cover_url: Option<String>,
    pub preview_url: Option<String>,
    pub osu_file_url: Option<String>,
    pub beatmaps: Vec<BeatmapInfo>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct BeatmapInfo {
    pub beatmap_osu_id: i32,
    pub name: String,
    pub count_circles: i32,
    pub count_sliders: i32,
    pub count_spinners: i32,
    pub od: f64,
    pub hp: f64,
    pub main_pattern: serde_json::Value,
    pub ratings: Vec<RatingInfo>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct RatingInfo {
    pub rating_type: String,
    pub rating_value: f64,
}


