pub struct Beatmapset {
    pub osu_id: Option<i32>,
    pub artist: String,
    pub title: String,
    pub creator: String,
    pub cover_url: Option<String>,
    pub total_beatmaps: i32,
    pub beatmaps: Vec<Beatmap>,
}

pub struct Rating {
    pub rating: f64,
    pub rating_type: String,
}

pub struct Beatmap {
    pub osu_id: Option<i32>,
    pub difficulty: String,
    pub mode: i32,
    pub status: String,
    pub ratings: Vec<Rating>,
}
