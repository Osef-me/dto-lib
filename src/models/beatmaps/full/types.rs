use serde::Serialize;
use utoipa::ToSchema;
use crate::models::rate;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Beatmapset {
    pub id: Option<i32>,
    pub osu_id: Option<i32>,
    pub artist: String,
    pub artist_unicode: Option<String>,
    pub title: String,
    pub title_unicode: Option<String>,
    pub creator: String,
    pub source: Option<String>,
    pub tags: Option<String>,
    pub has_video: bool,
    pub has_storyboard: bool,
    pub is_explicit: bool,
    pub is_featured: bool,
    pub cover_url: Option<String>,
    pub preview_url: Option<String>,
    pub osu_file_url: Option<String>,
    pub beatmaps: Vec<Beatmap>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Beatmap {
    pub id: Option<i32>,
    pub osu_id: Option<i32>,
    pub beatmapset_id: Option<i32>,
    pub difficulty: String,
    pub count_circles: i32,
    pub count_sliders: i32,
    pub count_spinners: i32,
    pub max_combo: i32,
    pub cs: f64,
    pub ar: f64,
    pub od: f64,
    pub hp: f64,
    pub mode: i32,
    pub status: String,
    pub main_pattern: serde_json::Value,
    pub rates: Vec<rate::Rates>,
}