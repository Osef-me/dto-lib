use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct RatingFilter {
    pub rating_type: Option<String>,
    pub rating_min: Option<f64>,
    pub rating_max: Option<f64>,
}

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct SkillsetFilter {
    pub pattern_type: Option<String>,
    pub pattern_min: Option<f64>,
    pub pattern_max: Option<f64>,
}

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct BeatmapFilter {
    pub search_term: Option<String>,
    pub total_time_min: Option<i32>,
    pub total_time_max: Option<i32>,
    pub bpm_min: Option<f64>,
    pub bpm_max: Option<f64>,
}

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct BeatmapTechnicalFilter {
    /// Overall Difficulty (OD) range
    pub od_min: Option<f64>,
    pub od_max: Option<f64>,
    /// Beatmap status (pending, ranked, qualified, loved, graveyard)
    pub status: Option<String>,
}

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct RatesFilter {
    /// Drain time in seconds
    pub drain_time_min: Option<i32>,
    pub drain_time_max: Option<i32>,
}

#[derive(Deserialize, Debug, Clone, IntoParams)]
pub struct Filters {
    pub rating: Option<RatingFilter>,
    pub skillset: Option<SkillsetFilter>,
    pub beatmap: Option<BeatmapFilter>,
    pub beatmap_technical: Option<BeatmapTechnicalFilter>,
    pub rates: Option<RatesFilter>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}
