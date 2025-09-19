use crate::filters::Filters;
use crate::models::beatmaps::short::types::BeatmapsetShort;
use bigdecimal::ToPrimitive;
use sqlx::PgPool;

pub async fn find_all_with_filters(
    pool: &PgPool,
    filters: Filters,
) -> Result<Vec<BeatmapsetShort>, sqlx::Error> {
    let page = filters.page.unwrap_or(0);
    let per_page = filters.per_page.unwrap_or(20);
    let offset = page * per_page;

    // Build the query with optional filters
    let query = sqlx::query!(
        r#"
        SELECT DISTINCT
            bs.id as beatmapset_id,
            bs.osu_id as beatmapset_osu_id,
            bs.artist,
            bs.title,
            bs.creator,
            bs.cover_url,
            b.id as beatmap_id,
            b.osu_id as beatmap_osu_id,
            b.difficulty,
            b.mode,
            b.status,
            br.id as rating_id,
            br.rating,
            br.rating_type
        FROM beatmapset bs
        INNER JOIN beatmap b ON bs.id = b.beatmapset_id
        INNER JOIN rates r ON b.id = r.beatmap_id
        INNER JOIN beatmap_rating br ON r.id = br.rates_id
        WHERE r.centirate = 100
        AND ($3::text IS NULL OR br.rating_type = $3)
        AND ($4::float8 IS NULL OR br.rating >= $4)
        AND ($5::float8 IS NULL OR br.rating <= $5)
        AND ($6::text IS NULL OR (bs.artist ILIKE $6 OR bs.title ILIKE $6 OR bs.creator ILIKE $6))
        AND ($7::int4 IS NULL OR r.total_time >= $7)
        AND ($8::int4 IS NULL OR r.total_time <= $8)
        AND ($9::float8 IS NULL OR r.bpm >= $9)
        AND ($10::float8 IS NULL OR r.bpm <= $10)
        ORDER BY bs.id, b.id, br.id
        LIMIT $1 OFFSET $2
        "#,
        per_page as i64,
        offset as i64,
        filters.rating.as_ref().and_then(|r| r.rating_type.as_ref()),
        filters.rating.as_ref().and_then(|r| r.rating_min),
        filters.rating.as_ref().and_then(|r| r.rating_max),
        filters
            .beatmap
            .as_ref()
            .and_then(|b| b.search_term.as_ref())
            .map(|s| format!("%{}%", s)),
        filters.beatmap.as_ref().and_then(|b| b.total_time_min),
        filters.beatmap.as_ref().and_then(|b| b.total_time_max),
        filters.beatmap.as_ref().and_then(|b| b.bpm_min),
        filters.beatmap.as_ref().and_then(|b| b.bpm_max),
    );

    let rows = query.fetch_all(pool).await?;

    // Group by beatmapset
    let mut beatmapsets: std::collections::HashMap<i32, BeatmapsetShort> =
        std::collections::HashMap::new();

    for row in rows {
        let beatmapset_id = row.beatmapset_id;

        let beatmapset = beatmapsets
            .entry(beatmapset_id)
            .or_insert_with(|| BeatmapsetShort {
                beatmapset: crate::models::beatmaps::short::types::Beatmapset {
                    osu_id: row.beatmapset_osu_id,
                    artist: row.artist,
                    title: row.title,
                    creator: row.creator,
                    cover_url: row.cover_url,
                },
                beatmaps: Vec::new(),
            });

        // Find or create beatmap
        let beatmap_exists = beatmapset
            .beatmaps
            .iter()
            .any(|b| b.osu_id == row.beatmap_osu_id);

        if !beatmap_exists {
            beatmapset
                .beatmaps
                .push(crate::models::beatmaps::short::types::Beatmap {
                    osu_id: row.beatmap_osu_id,
                    difficulty: row.difficulty,
                    mode: row.mode,
                    status: row.status,
                    ratings: Vec::new(),
                });
        }

        let beatmap = beatmapset
            .beatmaps
            .iter_mut()
            .find(|b| b.osu_id == row.beatmap_osu_id)
            .unwrap();

        // Add rating
        beatmap
            .ratings
            .push(crate::models::beatmaps::short::types::Rating {
                rating: row.rating.to_f64().unwrap_or(0.0),
                rating_type: row.rating_type,
            });
    }

    Ok(beatmapsets.into_values().collect())
}
