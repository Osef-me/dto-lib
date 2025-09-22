use crate::models::beatmaps::short::types::{Beatmap, Beatmapset, Rating};
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
    pub fn from_row(
        row: BeatmapsetRow,
        beatmaps: Vec<BeatmapRow>,
        ratings: Vec<BeatmapRatingRow>,
        rating_type: Option<&str>,
    ) -> Self {
        let total_beatmaps = beatmaps.len() as i32;
        let rating_type = rating_type.unwrap_or("osu");
        
        // Créer tous les beatmaps d'abord
        let mut beatmaps_result = Vec::new();
        for beatmap_row in beatmaps {
            beatmaps_result.push(Beatmap::from_row(beatmap_row, ratings.clone()));
        }
        
        // Trier par rating croissant
        beatmaps_result.sort_by(|a, b| {
            let a_rating = a.ratings
                .iter()
                .filter(|r| r.rating_type == rating_type)
                .map(|r| r.rating)
                .fold(f64::INFINITY, f64::min);
            
            let b_rating = b.ratings
                .iter()
                .filter(|r| r.rating_type == rating_type)
                .map(|r| r.rating)
                .fold(f64::INFINITY, f64::min);
            
            a_rating.partial_cmp(&b_rating).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Prendre les 5 premiers (plus bas rating)
        if beatmaps_result.len() > 5 {
            beatmaps_result.truncate(5);
        }

        Self {
            osu_id: row.osu_id,
            artist: row.artist,
            title: row.title,
            cover_url: row.cover_url,
            creator: row.creator,
            total_beatmaps,
            beatmaps: beatmaps_result,
        }
    }
}

