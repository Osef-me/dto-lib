use crate::models::beatmaps::simple::types::{Beatmapset, BeatmapInfo, RatingInfo};
use sqlx::PgPool;
use bigdecimal::ToPrimitive;
use std::collections::HashMap;
use serde_json;

pub async fn find_by_osu_id(
    pool: &PgPool,
    osu_id: i32,
    rating_type: Option<String>,
) -> Result<Option<Beatmapset>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            bs.id AS bs_id,
            bs.osu_id AS bs_osu_id,
            bs.artist AS bs_artist,
            bs.artist_unicode AS bs_artist_unicode,
            bs.title AS bs_title,
            bs.title_unicode AS bs_title_unicode,
            bs.creator AS bs_creator,
            bs.source AS bs_source,
            bs.tags AS bs_tags,
            bs.has_video AS bs_has_video,
            bs.has_storyboard AS bs_has_storyboard,
            bs.is_explicit AS bs_is_explicit,
            bs.is_featured AS bs_is_featured,
            bs.cover_url AS bs_cover_url,
            bs.preview_url AS bs_preview_url,
            bs.osu_file_url AS bs_osu_file_url,
            b.osu_id AS b_osu_id,
            b.difficulty AS b_difficulty,
            b.count_circles AS b_count_circles,
            b.count_sliders AS b_count_sliders,
            b.count_spinners AS b_count_spinners,
            b.od AS b_od,
            b.hp AS b_hp,
            b.main_pattern AS b_main_pattern,
            br.rating_type AS br_rating_type,
            br.rating AS br_rating
        FROM beatmapset bs
        INNER JOIN beatmap b ON bs.id = b.beatmapset_id
        INNER JOIN rates r ON b.id = r.beatmap_id AND r.centirate = 100
        LEFT JOIN beatmap_rating br ON r.id = br.rates_id
        WHERE bs.osu_id = $1
        AND ($2::text IS NULL OR br.rating_type = $2::text)
        ORDER BY b.osu_id, br.rating_type
        "#,
        osu_id,
        rating_type
    )
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let first_row = &rows[0];

    // Grouper les résultats par beatmap
    let mut beatmaps_map = HashMap::new();

    for row in &rows {
        let beatmap_osu_id = row.b_osu_id;
        let difficulty = row.b_difficulty.clone();
        let rating_type = row.br_rating_type.clone();
        let rating_value = row.br_rating.to_f64().unwrap_or(0.0);

        // Skip if rating_type is empty (no valid rating)
        if rating_type.is_empty() {
            continue;
        }

        let beatmap_info = beatmaps_map.entry(beatmap_osu_id).or_insert_with(|| BeatmapInfo {
            beatmap_osu_id: beatmap_osu_id.unwrap_or(0),
            name: difficulty,
            count_circles: row.b_count_circles,
            count_sliders: row.b_count_sliders,
            count_spinners: row.b_count_spinners,
            od: row.b_od.to_f64().unwrap_or(0.0),
            hp: row.b_hp.to_f64().unwrap_or(0.0),
            main_pattern: row.b_main_pattern.clone(),
            ratings: Vec::new(),
        });

        beatmap_info.ratings.push(RatingInfo {
            rating_type,
            rating_value,
        });
    }

    let beatmaps = beatmaps_map.into_values().collect();

    Ok(Some(Beatmapset {
        id: first_row.bs_id,
        osu_id: first_row.bs_osu_id,
        artist: first_row.bs_artist.clone(),
        artist_unicode: first_row.bs_artist_unicode.clone(),
        title: first_row.bs_title.clone(),
        title_unicode: first_row.bs_title_unicode.clone(),
        creator: first_row.bs_creator.clone(),
        source: first_row.bs_source.clone(),
        tags: first_row.bs_tags.clone(),
        has_video: first_row.bs_has_video,
        has_storyboard: first_row.bs_has_storyboard,
        is_explicit: first_row.bs_is_explicit,
        is_featured: first_row.bs_is_featured,
        cover_url: first_row.bs_cover_url.clone(),
        preview_url: first_row.bs_preview_url.clone(),
        osu_file_url: first_row.bs_osu_file_url.clone() ,
        beatmaps,
    }))
}