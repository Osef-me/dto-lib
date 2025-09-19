use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct RatingFilter {
    pub rating_type: Option<String>,
    pub rating_min: Option<f64>,
    pub rating_max: Option<f64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PatternFilter {
    pub pattern_type: Option<String>,
    pub pattern_min: Option<f64>,
    pub pattern_max: Option<f64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BeatmapFilter {
    pub search_term: Option<String>,
    pub total_time_min: Option<i32>,
    pub total_time_max: Option<i32>,
    pub bpm_min: Option<f64>,
    pub bpm_max: Option<f64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Filters {
    pub rating: Option<RatingFilter>,
    pub pattern: Option<PatternFilter>,
    pub beatmap: Option<BeatmapFilter>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}
