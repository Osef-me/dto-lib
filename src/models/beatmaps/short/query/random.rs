use crate::filters::Filters;
use crate::models::beatmaps::short::types::Beatmapset;
use sqlx::{PgPool, Postgres, QueryBuilder};

use super::common::{preferred_rating_type, sort_and_limit_beatmaps, apply_filters, group_beatmapset_rows};

/// Find random beatmapsets with filters - optimized for speed by using a single query with RANDOM()
pub async fn find_random_with_filters(
    pool: &PgPool,
    filters: Filters,
) -> Result<Vec<Beatmapset>, sqlx::Error> {
    // Phase 1: fetch 9 random beatmapset ids using a subquery to handle DISTINCT + ORDER BY RANDOM()
    // PostgreSQL doesn't allow ORDER BY RANDOM() directly with SELECT DISTINCT
    let mut ids_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT id FROM (SELECT DISTINCT bs.id FROM beatmapset bs \n            INNER JOIN beatmap b ON bs.id = b.beatmapset_id\n            INNER JOIN rates r ON b.id = r.beatmap_id\n            INNER JOIN beatmap_rating br ON r.id = br.rates_id\n            LEFT JOIN beatmap_mania_rating bmr ON br.id = bmr.rating_id\n            WHERE r.centirate = 100",
    );

    // Apply filters using the common function
    apply_filters(&mut ids_builder, &filters);

    ids_builder.push(") AS filtered_ids ORDER BY RANDOM() LIMIT 9");

    let beatmapset_ids: Vec<i32> = ids_builder
        .build_query_scalar()
        .fetch_all(pool)
        .await?;

    if beatmapset_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Phase 2: fetch detailed rows for selected beatmapset ids (same as find_all_with_filters)
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT\n            bs.id as beatmapset_id,\n            bs.osu_id as beatmapset_osu_id,\n            bs.artist,\n            bs.title,\n            bs.creator,\n            bs.cover_url,\n            b.id as beatmap_id,\n            b.osu_id as beatmap_osu_id,\n            b.difficulty,\n            b.mode,\n            b.status,\n            b.main_pattern,\n            b.od,\n            r.drain_time,\n            br.id as rating_id,\n            br.rating,\n            br.rating_type,\n            bmr.stream as mania_stream,\n            bmr.jumpstream as mania_jumpstream,\n            bmr.handstream as mania_handstream,\n            bmr.stamina as mania_stamina,\n            bmr.jackspeed as mania_jackspeed,\n            bmr.chordjack as mania_chordjack,\n            bmr.technical as mania_technical\n        FROM beatmapset bs\n        INNER JOIN beatmap b ON bs.id = b.beatmapset_id\n        INNER JOIN rates r ON b.id = r.beatmap_id\n        INNER JOIN beatmap_rating br ON r.id = br.rates_id\n        LEFT JOIN beatmap_mania_rating bmr ON br.id = bmr.rating_id\n        WHERE r.centirate = 100",
    );

    // Re-apply filters using the common function
    apply_filters(&mut builder, &filters);

    // Constrain to selected beatmapsets
    builder.push(" AND bs.id = ANY(").push_bind(&beatmapset_ids).push(")");
    builder.push(" ORDER BY bs.id, b.id, br.id");

    let rows = builder.build().fetch_all(pool).await?;

    // Group by beatmapset using the common function
    let mut beatmapsets = group_beatmapset_rows(rows)?;

    let preferred_type = preferred_rating_type(&filters);
    sort_and_limit_beatmaps(&mut beatmapsets, preferred_type);

    Ok(beatmapsets.into_values().collect())
}
