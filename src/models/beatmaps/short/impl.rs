use crate::models::beatmaps::short::types::{Beatmap, Beatmapset, BeatmapsetShort, Rating};
use bigdecimal::ToPrimitive;
use db::models::beatmaps::beatmap::types::BeatmapRow;
use db::models::beatmaps::beatmapset::types::BeatmapsetRow;
use db::models::rating::beatmap_rating::types::BeatmapRatingRow;

impl Rating {
    pub fn from_row(row: BeatmapRatingRow) -> Self {
        Self {
            rating: row.rating.to_f64().unwrap(),
            rating_type: row.rating_type,
        }
    }
}

impl Beatmap {
    pub fn from_row(row: BeatmapRow, ratings: Vec<BeatmapRatingRow>) -> Self {
        Self {
            osu_id: row.osu_id,
            difficulty: row.difficulty,
            mode: row.mode,
            status: row.status,
            ratings: ratings
                .into_iter()
                .map(|row| Rating::from_row(row))
                .collect(),
        }
    }
}

impl Beatmapset {
    pub fn from_row(row: BeatmapsetRow) -> Self {
        Self {
            osu_id: row.osu_id,
            artist: row.artist,
            title: row.title,
            cover_url: row.cover_url,
            creator: row.creator,
        }
    }
}

impl BeatmapsetShort {
    pub fn from_row(
        row: BeatmapsetRow,
        beatmaps: Vec<BeatmapRow>,
        ratings: Vec<BeatmapRatingRow>,
    ) -> Self {
        let beatmapset = Beatmapset::from_row(row);

        let mut beatmaps_result = Vec::new();
        for beatmap_row in beatmaps {
            beatmaps_result.push(Beatmap::from_row(beatmap_row, ratings.clone()));
        }

        Self {
            beatmapset,
            beatmaps: beatmaps_result,
        }
    }
}
