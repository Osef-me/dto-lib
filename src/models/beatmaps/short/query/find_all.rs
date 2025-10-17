use crate::filters::Filters;
use crate::models::beatmaps::short::types::Beatmapset;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

use super::common::{
    apply_filters, group_beatmapset_rows, preferred_rating_type, sort_and_limit_beatmaps,
};

pub async fn find_all_with_filters(
    pool: &PgPool,
    filters: Filters,
) -> Result<Vec<Beatmapset>, sqlx::Error> {
    let page = filters.page.unwrap_or(0);
    let per_page = filters.per_page.unwrap_or(9);
    let offset = page * per_page;

    // Determine which joins are actually needed for filters
    let needs_rating = true; // always join br to satisfy filters that reference it
    let needs_skill = filters
        .skillset
        .as_ref()
        .and_then(|s| s.pattern_type.as_ref())
        .is_some();

    // Phase 1: fetch paginated beatmapset ids using DISTINCT over filtered base
    let mut ids_sql = String::from(
        "SELECT DISTINCT bs.id FROM beatmapset bs\n            INNER JOIN beatmap b ON bs.id = b.beatmapset_id\n            INNER JOIN rates r ON b.id = r.beatmap_id",
    );
    // Always join rating table to avoid missing FROM when filters add conditions on br
    ids_sql.push_str("\n            INNER JOIN beatmap_rating br ON r.id = br.rates_id");
    if needs_skill {
        // requires br join too since bmr is linked via rating
        if !needs_rating {
            ids_sql.push_str("\n            INNER JOIN beatmap_rating br ON r.id = br.rates_id");
        }
        ids_sql.push_str("\n            LEFT JOIN beatmap_mania_rating bmr ON br.id = bmr.rating_id");
    }
    ids_sql.push_str("\n            WHERE r.centirate = 100");

    let mut ids_builder: QueryBuilder<Postgres> = QueryBuilder::new(ids_sql);

    // Apply filters using the common function
    apply_filters(&mut ids_builder, &filters);

    ids_builder
        .push(" ORDER BY bs.id LIMIT ")
        .push_bind(per_page as i64)
        .push(" OFFSET ")
        .push_bind(offset as i64);

    let beatmapset_ids: Vec<i32> = ids_builder.build_query_scalar().fetch_all(pool).await?;

    if beatmapset_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Phase 2: fetch detailed rows for selected beatmapset ids
    let mut detail_sql = String::from(
        "SELECT\n            bs.id as beatmapset_id,\n            bs.osu_id as beatmapset_osu_id,\n            bs.artist,\n            bs.title,\n            bs.creator,\n            bs.cover_url,\n            b.id as beatmap_id,\n            b.osu_id as beatmap_osu_id,\n            b.difficulty,\n            b.mode,\n            b.status,\n            b.main_pattern,\n            b.od,\n            r.drain_time",
    );
    detail_sql.push_str(",\n            br.id as rating_id,\n            br.rating,\n            br.rating_type");
    if needs_skill {
        detail_sql.push_str(",\n            bmr.stream as mania_stream,\n            bmr.jumpstream as mania_jumpstream,\n            bmr.handstream as mania_handstream,\n            bmr.stamina as mania_stamina,\n            bmr.jackspeed as mania_jackspeed,\n            bmr.chordjack as mania_chordjack,\n            bmr.technical as mania_technical");
    } else {
        detail_sql.push_str(",\n            NULL::numeric as mania_stream,\n            NULL::numeric as mania_jumpstream,\n            NULL::numeric as mania_handstream,\n            NULL::numeric as mania_stamina,\n            NULL::numeric as mania_jackspeed,\n            NULL::numeric as mania_chordjack,\n            NULL::numeric as mania_technical");
    }
    detail_sql.push_str(
        "\n        FROM beatmapset bs\n        INNER JOIN beatmap b ON bs.id = b.beatmapset_id\n        INNER JOIN rates r ON b.id = r.beatmap_id",
    );
    detail_sql.push_str("\n        INNER JOIN beatmap_rating br ON r.id = br.rates_id");
    if needs_skill {
        if !needs_rating {
            detail_sql.push_str("\n        INNER JOIN beatmap_rating br ON r.id = br.rates_id");
        }
        detail_sql.push_str("\n        LEFT JOIN beatmap_mania_rating bmr ON br.id = bmr.rating_id");
    }
    detail_sql.push_str("\n        WHERE r.centirate = 100");

    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(detail_sql);

    // Re-apply filters using the common function
    apply_filters(&mut builder, &filters);

    // Constrain to selected beatmapsets
    builder
        .push(" AND bs.id = ANY(")
        .push_bind(&beatmapset_ids)
        .push(")");
    builder.push(" ORDER BY bs.id, b.id, br.id");

    let rows = builder.build().fetch_all(pool).await?;

    // Group by beatmapset using the common function
    let mut beatmapsets = group_beatmapset_rows(rows)?;

    let preferred_type = preferred_rating_type(&filters);
    sort_and_limit_beatmaps(&mut beatmapsets, preferred_type);

    Ok(beatmapsets.into_values().collect())
}

pub async fn count_with_filters(pool: &PgPool, filters: &Filters) -> Result<i64, sqlx::Error> {
    let needs_rating = filters.rating.is_some();
    let needs_skill = filters
        .skillset
        .as_ref()
        .and_then(|s| s.pattern_type.as_ref())
        .is_some();

    let mut count_sql = String::from(
        "SELECT COUNT(DISTINCT bs.id) AS total\n        FROM beatmapset bs\n        INNER JOIN beatmap b ON bs.id = b.beatmapset_id\n        INNER JOIN rates r ON b.id = r.beatmap_id",
    );
    if needs_rating {
        count_sql.push_str("\n        INNER JOIN beatmap_rating br ON r.id = br.rates_id");
    }
    if needs_skill {
        if !needs_rating {
            count_sql.push_str("\n        INNER JOIN beatmap_rating br ON r.id = br.rates_id");
        }
        count_sql.push_str("\n        LEFT JOIN beatmap_mania_rating bmr ON br.id = bmr.rating_id");
    }
    count_sql.push_str("\n        WHERE r.centirate = 100");

    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(count_sql);

    // Apply the same filters as in find_all_with_filters
    apply_filters(&mut builder, filters);

    let row = builder.build().fetch_one(pool).await?;

    let total: i64 = row.try_get("total")?;
    Ok(total)
}
