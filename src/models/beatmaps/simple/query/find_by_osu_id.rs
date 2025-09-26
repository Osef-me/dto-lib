use crate::models::beatmaps::simple::types::{Beatmapset, BeatmapRating};
use sqlx::PgPool;
use bigdecimal::ToPrimitive;

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
            b.id AS b_id,
            b.difficulty AS b_difficulty,
            br.rating AS br_rating
        FROM beatmapset bs
        LEFT JOIN beatmap b ON bs.id = b.beatmapset_id
        LEFT JOIN rates r ON b.id = r.beatmap_id AND r.centirate = 100
        LEFT JOIN beatmap_rating br ON r.id = br.rates_id
        WHERE bs.osu_id = $1
        AND ($2::text IS NULL OR br.rating_type = $2::text)
        ORDER BY b.id
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
    let mut beatmaps = Vec::new();

    for row in rows {
        if let (Some(b_id), Some(rating)) = (row.b_id, row.br_rating) {
            beatmaps.push(BeatmapRating {
                beatmap_id: b_id,
                rating_value: rating.to_f64().unwrap_or_default(),
                name: row.b_difficulty,
            });
        }
    }

    Ok(Some(Beatmapset {
        id: first_row.bs_id,
        osu_id: first_row.bs_osu_id,
        artist: first_row.bs_artist,
        artist_unicode: first_row.bs_artist_unicode,
        title: first_row.bs_title,
        title_unicode: first_row.bs_title_unicode,
        creator: first_row.bs_creator,
        source: first_row.bs_source,
        tags: first_row.bs_tags,
        has_video: first_row.bs_has_video,
        has_storyboard: first_row.bs_has_storyboard,
        is_explicit: first_row.bs_is_explicit,
        is_featured: first_row.bs_is_featured,
        cover_url: first_row.bs_cover_url,
        preview_url: first_row.bs_preview_url,
        osu_file_url: first_row.bs_osu_file_url,
        beatmaps,
    }))
}