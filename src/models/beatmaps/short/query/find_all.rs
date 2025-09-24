use crate::filters::Filters;
use crate::models::beatmaps::short::types::Beatmapset;
use bigdecimal::{BigDecimal, ToPrimitive};
use sqlx::{PgPool, Row};
use std::collections::HashMap;

/// Determine the preferred rating type from filters or fallback to "osu".
fn preferred_rating_type(filters: &Filters) -> &str {
    filters
        .rating
        .as_ref()
        .and_then(|r| r.rating_type.as_deref())
        .unwrap_or("osu")
}

/// Compute a comparable score for a beatmap given the preferred rating type.
/// Fallback to `osu` if the preferred type is unavailable; otherwise return -inf.
fn beatmap_score(
    beatmap: &crate::models::beatmaps::short::types::Beatmap,
    preferred_type: &str,
) -> f64 {
    if let Some(r) = beatmap
        .ratings
        .iter()
        .find(|r| r.rating_type == preferred_type)
    {
        r.rating
    } else if let Some(r) = beatmap.ratings.iter().find(|r| r.rating_type == "osu") {
        r.rating
    } else {
        f64::NEG_INFINITY
    }
}

/// Sort beatmaps per set by ascending difficulty according to `preferred_type`,
/// then keep the 5 easiest plus the single hardest (total up to 6). Also set
/// `total_beatmaps` to the pre-truncation count for each set.
fn sort_and_limit_beatmaps(
    beatmapsets: &mut HashMap<i32, Beatmapset>,
    preferred_type: &str,
) {
    for beatmapset in beatmapsets.values_mut() {
        let original_count = beatmapset.beatmaps.len();

        // Sort easiest first
        beatmapset.beatmaps.sort_by(|a, b| {
            beatmap_score(a, preferred_type)
                .partial_cmp(&beatmap_score(b, preferred_type))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if beatmapset.beatmaps.len() > 6 {
            // Identify the hardest
            let hardest = beatmapset
                .beatmaps
                .iter()
                .max_by(|a, b| {
                    beatmap_score(a, preferred_type)
                        .partial_cmp(&beatmap_score(b, preferred_type))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .cloned();

            // Keep first 5 (easiest) and the hardest
            let mut kept: Vec<crate::models::beatmaps::short::types::Beatmap> =
                beatmapset.beatmaps.iter().take(5).cloned().collect();
            if let Some(h) = hardest {
                if !kept.iter().any(|bm| bm.osu_id == h.osu_id) {
                    kept.push(h);
                }
            }
            beatmapset.beatmaps = kept;
        }

        beatmapset.total_beatmaps = original_count as i32;
    }
}

pub async fn find_all_with_filters(
    pool: &PgPool,
    filters: Filters,
) -> Result<Vec<Beatmapset>, sqlx::Error> {
    let page = filters.page.unwrap_or(0);
    let per_page = filters.per_page.unwrap_or(9);
    let offset = page * per_page;

    // Build the query with optional filters (runtime checked)
    let rows = sqlx::query(
        r#"
        WITH base AS (
            SELECT
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
                b.main_pattern,
                br.id as rating_id,
                br.rating,
                br.rating_type,
                bmr.stream as mania_stream,
                bmr.jumpstream as mania_jumpstream,
                bmr.handstream as mania_handstream,
                bmr.stamina as mania_stamina,
                bmr.jackspeed as mania_jackspeed,
                bmr.chordjack as mania_chordjack,
                bmr.technical as mania_technical
            FROM beatmapset bs
            INNER JOIN beatmap b ON bs.id = b.beatmapset_id
            INNER JOIN rates r ON b.id = r.beatmap_id
            INNER JOIN beatmap_rating br ON r.id = br.rates_id
            LEFT JOIN beatmap_mania_rating bmr ON br.id = bmr.rating_id
            WHERE r.centirate = 100
            AND ($3::text IS NULL OR br.rating_type = $3)
            AND ($4::float8 IS NULL OR br.rating >= $4)
            AND ($5::float8 IS NULL OR br.rating <= $5)
            AND ($6::text IS NULL OR (bs.artist ILIKE $6 OR bs.title ILIKE $6 OR bs.creator ILIKE $6))
            AND ($7::int4 IS NULL OR r.total_time >= $7)
            AND ($8::int4 IS NULL OR r.total_time <= $8)
            AND ($9::float8 IS NULL OR r.bpm >= $9)
            AND ($10::float8 IS NULL OR r.bpm <= $10)
            AND ($11::text IS NULL OR EXISTS (
                SELECT 1 FROM jsonb_array_elements_text(b.main_pattern) AS elem(value)
                WHERE elem.value = $11
            ))
            AND (
                $11::text IS NULL
                OR (
                    ($11 = 'jumpstream' AND ($12::float8 IS NULL OR bmr.jumpstream >= $12) AND ($13::float8 IS NULL OR bmr.jumpstream <= $13))
                    OR ($11 = 'stream' AND ($12::float8 IS NULL OR bmr.stream >= $12) AND ($13::float8 IS NULL OR bmr.stream <= $13))
                    OR ($11 = 'handstream' AND ($12::float8 IS NULL OR bmr.handstream >= $12) AND ($13::float8 IS NULL OR bmr.handstream <= $13))
                    OR ($11 = 'stamina' AND ($12::float8 IS NULL OR bmr.stamina >= $12) AND ($13::float8 IS NULL OR bmr.stamina <= $13))
                    OR ($11 = 'jackspeed' AND ($12::float8 IS NULL OR bmr.jackspeed >= $12) AND ($13::float8 IS NULL OR bmr.jackspeed <= $13))
                    OR ($11 = 'chordjack' AND ($12::float8 IS NULL OR bmr.chordjack >= $12) AND ($13::float8 IS NULL OR bmr.chordjack <= $13))
                    OR ($11 = 'technical' AND ($12::float8 IS NULL OR bmr.technical >= $12) AND ($13::float8 IS NULL OR bmr.technical <= $13))
                )
            )
        ),
        page_sets AS (
            SELECT DISTINCT beatmapset_id
            FROM base
            ORDER BY beatmapset_id
            LIMIT $1 OFFSET $2
        )
        SELECT base.*
        FROM base
        INNER JOIN page_sets ps ON base.beatmapset_id = ps.beatmapset_id
        ORDER BY base.beatmapset_id, base.beatmap_id, base.rating_id
        "#,
    )
    .bind(per_page as i64)
    .bind(offset as i64)
    .bind(filters.rating.as_ref().and_then(|r| r.rating_type.as_ref()))
    .bind(filters.rating.as_ref().and_then(|r| r.rating_min))
    .bind(filters.rating.as_ref().and_then(|r| r.rating_max))
    .bind(
        filters
            .beatmap
            .as_ref()
            .and_then(|b| b.search_term.as_ref())
            .map(|s| format!("%{}%", s)),
    )
    .bind(filters.beatmap.as_ref().and_then(|b| b.total_time_min))
    .bind(filters.beatmap.as_ref().and_then(|b| b.total_time_max))
    .bind(filters.beatmap.as_ref().and_then(|b| b.bpm_min))
    .bind(filters.beatmap.as_ref().and_then(|b| b.bpm_max))
    .bind(filters.pattern.as_ref().and_then(|p| p.pattern_type.as_ref()))
    .bind(filters.pattern.as_ref().and_then(|p| p.pattern_min))
    .bind(filters.pattern.as_ref().and_then(|p| p.pattern_max))
    .fetch_all(pool)
    .await?;

    // Group by beatmapset
    let mut beatmapsets: HashMap<i32, Beatmapset> = HashMap::new();

    for row in rows {
        let beatmapset_id: i32 = row.try_get("beatmapset_id")?;

        let beatmapset = beatmapsets
            .entry(beatmapset_id)
            .or_insert_with(|| Beatmapset {
                osu_id: row.try_get("beatmapset_osu_id").ok(),
                artist: row.try_get("artist").unwrap_or_default(),
                title: row.try_get("title").unwrap_or_default(),
                creator: row.try_get("creator").unwrap_or_default(),
                cover_url: row.try_get("cover_url").ok(),
                total_beatmaps: 0, // Sera mis à jour plus tard
                beatmaps: Vec::new(),
            });

        // Find or create beatmap
        let beatmap_osu_id: Option<i32> = row.try_get("beatmap_osu_id").ok();
        let beatmap_exists = beatmapset
            .beatmaps
            .iter()
            .any(|b| b.osu_id == beatmap_osu_id);

        if !beatmap_exists {
            beatmapset.beatmaps.push(crate::models::beatmaps::short::types::Beatmap {
                osu_id: beatmap_osu_id,
                difficulty: row.try_get("difficulty").unwrap_or_default(),
                mode: row.try_get("mode").unwrap_or(0),
                status: row.try_get("status").unwrap_or_default(),
                main_pattern: row.try_get("main_pattern").unwrap_or(serde_json::json!({})),
                ratings: Vec::new(),
            });
        }

        let beatmap = beatmapset
            .beatmaps
            .iter_mut()
            .find(|b| b.osu_id == beatmap_osu_id)
            .unwrap();

        // Add rating
        let rating_bd: BigDecimal = row.try_get("rating").unwrap_or_else(|_| BigDecimal::from(0));
        let rating_type: String = row.try_get("rating_type").unwrap_or_default();
        beatmap.ratings.push(crate::models::beatmaps::short::types::Rating {
            rating: rating_bd.to_f64().unwrap_or(0.0),
            rating_type,
        });
    }
    
    let preferred_type = preferred_rating_type(&filters);
    sort_and_limit_beatmaps(&mut beatmapsets, preferred_type);

    Ok(beatmapsets.into_values().collect())
}

pub async fn count_with_filters(
    pool: &PgPool,
    filters: &Filters,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT COUNT(DISTINCT bs.id) AS total
        FROM beatmapset bs
        INNER JOIN beatmap b ON bs.id = b.beatmapset_id
        INNER JOIN rates r ON b.id = r.beatmap_id
        INNER JOIN beatmap_rating br ON r.id = br.rates_id
        LEFT JOIN beatmap_mania_rating bmr ON br.id = bmr.rating_id
        WHERE r.centirate = 100
        AND ($1::text IS NULL OR br.rating_type = $1)
        AND ($2::float8 IS NULL OR br.rating >= $2)
        AND ($3::float8 IS NULL OR br.rating <= $3)
        AND ($4::text IS NULL OR (bs.artist ILIKE $4 OR bs.title ILIKE $4 OR bs.creator ILIKE $4))
        AND ($5::int4 IS NULL OR r.total_time >= $5)
        AND ($6::int4 IS NULL OR r.total_time <= $6)
        AND ($7::float8 IS NULL OR r.bpm >= $7)
        AND ($8::float8 IS NULL OR r.bpm <= $8)
        AND ($9::text IS NULL OR EXISTS (
            SELECT 1 FROM jsonb_array_elements_text(b.main_pattern) AS elem(value)
            WHERE elem.value = $9
        ))
        AND (
            $9::text IS NULL
            OR (
                ($9 = 'jumpstream' AND ($10::float8 IS NULL OR bmr.jumpstream >= $10) AND ($11::float8 IS NULL OR bmr.jumpstream <= $11))
                OR ($9 = 'stream' AND ($10::float8 IS NULL OR bmr.stream >= $10) AND ($11::float8 IS NULL OR bmr.stream <= $11))
                OR ($9 = 'handstream' AND ($10::float8 IS NULL OR bmr.handstream >= $10) AND ($11::float8 IS NULL OR bmr.handstream <= $11))
                OR ($9 = 'stamina' AND ($10::float8 IS NULL OR bmr.stamina >= $10) AND ($11::float8 IS NULL OR bmr.stamina <= $11))
                OR ($9 = 'jackspeed' AND ($10::float8 IS NULL OR bmr.jackspeed >= $10) AND ($11::float8 IS NULL OR bmr.jackspeed <= $11))
                OR ($9 = 'chordjack' AND ($10::float8 IS NULL OR bmr.chordjack >= $10) AND ($11::float8 IS NULL OR bmr.chordjack <= $11))
                OR ($9 = 'technical' AND ($10::float8 IS NULL OR bmr.technical >= $10) AND ($11::float8 IS NULL OR bmr.technical <= $11))
            )
        )
        "#,
    )
    .bind(filters.rating.as_ref().and_then(|r| r.rating_type.as_ref()))
    .bind(filters.rating.as_ref().and_then(|r| r.rating_min))
    .bind(filters.rating.as_ref().and_then(|r| r.rating_max))
    .bind(
        filters
            .beatmap
            .as_ref()
            .and_then(|b| b.search_term.as_ref())
            .map(|s| format!("%{}%", s)),
    )
    .bind(filters.beatmap.as_ref().and_then(|b| b.total_time_min))
    .bind(filters.beatmap.as_ref().and_then(|b| b.total_time_max))
    .bind(filters.beatmap.as_ref().and_then(|b| b.bpm_min))
    .bind(filters.beatmap.as_ref().and_then(|b| b.bpm_max))
    .bind(filters.pattern.as_ref().and_then(|p| p.pattern_type.as_ref()))
    .bind(filters.pattern.as_ref().and_then(|p| p.pattern_min))
    .bind(filters.pattern.as_ref().and_then(|p| p.pattern_max))
    .fetch_one(pool)
    .await?;

    let total: i64 = row.try_get("total")?;
    Ok(total)
}
