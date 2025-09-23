use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
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
    pub rates: Vec<Rates>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Rates {
    pub id: Option<i32>,
    pub osu_hash: Option<String>,
    pub centirate: i32,
    pub drain_time: i32,
    pub total_time: i32,
    pub bpm: f32,
    pub rating: Vec<Rating>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Rating {
    pub id: Option<i32>,
    pub rates_id: Option<i32>,
    pub rating: f64,
    pub rating_type: String,
    pub mode_rating: ModeRating,
}

#[derive(Debug, Clone, Serialize)]
pub struct ManiaRating {
    pub id: Option<i32>,
    pub stream: f64,
    pub jumpstream: f64,
    pub handstream: f64,
    pub stamina: f64,
    pub jackspeed: f64,
    pub chordjack: f64,
    pub technical: f64,
}

#[derive(Debug, Clone, Serialize)]
pub enum ModeRating {
    Mania(ManiaRating),
    Std,
    Ctb,
    Taiko,
}
