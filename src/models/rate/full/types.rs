use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Rates {
    pub id: Option<i32>,
    pub osu_hash: Option<String>,
    pub centirate: i32,
    pub drain_time: i32,
    pub total_time: i32,
    pub bpm: f32,
    pub rating: Vec<Rating>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Rating {
    pub id: Option<i32>,
    pub rates_id: Option<i32>,
    pub rating: f64,
    pub rating_type: String,
    pub mode_rating: ModeRating,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
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

#[derive(Debug, Clone, Serialize, ToSchema)]
pub enum ModeRating {
    Mania(ManiaRating),
    Std,
    Ctb,
    Taiko,
}
