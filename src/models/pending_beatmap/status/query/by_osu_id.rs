use crate::models::pending_beatmap::status::types::PendingStatusDto;
use sqlx::PgPool;

pub async fn find_status_by_osu_id(
    pool: &PgPool,
    osu_id: i32,
) -> Result<Option<PendingStatusDto>, sqlx::Error> {
    // Position in queue by created_at ascending
    let row_opt: Option<(i64,)> = sqlx::query_as(
        r#"
        WITH ordered AS (
            SELECT id, osu_id, created_at,
                   ROW_NUMBER() OVER (ORDER BY created_at ASC) AS rn
            FROM pending_beatmap
        )
        SELECT rn::bigint
        FROM ordered
        WHERE osu_id = $1
        LIMIT 1
        "#,
    )
    .bind(osu_id)
    .fetch_optional(pool)
    .await?;

    let Some((position,)) = row_opt else {
        return Ok(None);
    };

    let (total,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::bigint FROM pending_beatmap",
    )
    .fetch_one(pool)
    .await?;

    Ok(Some(PendingStatusDto { position, total }))
}


