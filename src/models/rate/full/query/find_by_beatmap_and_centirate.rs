use crate::models::rate::{Rates, ManiaRating, ModeRating, Rating};
use bigdecimal::ToPrimitive;
use sqlx::PgPool;

pub async fn find_rate_by_beatmap_osu_id_and_centirate(
    pool: &PgPool,
    beatmap_osu_id: i32,
    centirate: i32,
) -> Result<Option<Rates>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            r.id                     AS r_id,
            r.osu_hash               AS r_osu_hash,
            r.centirate              AS r_centirate,
            r.drain_time             AS r_drain_time,
            r.total_time             AS r_total_time,
            r.bpm                    AS r_bpm,
            br.id                    AS br_id,
            br.rates_id              AS br_rates_id,
            br.rating                AS br_rating,
            br.rating_type           AS br_rating_type,
            bmr.id                   AS bmr_id,
            bmr.stream               AS bmr_stream,
            bmr.jumpstream           AS bmr_jumpstream,
            bmr.handstream           AS bmr_handstream,
            bmr.stamina              AS bmr_stamina,
            bmr.jackspeed            AS bmr_jackspeed,
            bmr.chordjack            AS bmr_chordjack,
            bmr.technical            AS bmr_technical
        FROM beatmap b
        INNER JOIN rates r ON r.beatmap_id = b.id AND r.centirate = $2
        LEFT JOIN beatmap_rating br ON br.rates_id = r.id
        LEFT JOIN beatmap_mania_rating bmr ON bmr.rating_id = br.id
        WHERE b.osu_id = $1
        ORDER BY br.id ASC
        "#,
        beatmap_osu_id,
        centirate
    )
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let mut rate = Rates {
        id: Some(rows[0].r_id),
        osu_hash: Some(rows[0].r_osu_hash.clone()),
        centirate: rows[0].r_centirate,
        drain_time: rows[0].r_drain_time,
        total_time: rows[0].r_total_time,
        bpm: rows[0].r_bpm.to_f32().unwrap_or_default(),
        rating: Vec::new(),
    };

    for row in rows {
        if row.br_id != 0 {
            let mode_rating = ModeRating::Mania(ManiaRating {
                id: Some(row.bmr_id),
                stream: row.bmr_stream.and_then(|v| v.to_f64()).unwrap_or_default(),
                jumpstream: row.bmr_jumpstream.and_then(|v| v.to_f64()).unwrap_or_default(),
                handstream: row.bmr_handstream.and_then(|v| v.to_f64()).unwrap_or_default(),
                stamina: row.bmr_stamina.and_then(|v| v.to_f64()).unwrap_or_default(),
                jackspeed: row.bmr_jackspeed.and_then(|v| v.to_f64()).unwrap_or_default(),
                chordjack: row.bmr_chordjack.and_then(|v| v.to_f64()).unwrap_or_default(),
                technical: row.bmr_technical.and_then(|v| v.to_f64()).unwrap_or_default(),
            });
            rate.rating.push(Rating {
                id: Some(row.br_id),
                rates_id: row.br_rates_id,
                rating: row.br_rating.to_f64().unwrap_or_default(),
                rating_type: row.br_rating_type.clone(),
                mode_rating,
            });
        }
    }

    Ok(Some(rate))
}