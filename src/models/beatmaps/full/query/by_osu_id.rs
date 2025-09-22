use crate::models::beatmaps::full::types::{Beatmap, Beatmapset, ManiaRating, ModeRating, Rates, Rating};
use bigdecimal::ToPrimitive;
use db::models::beatmaps::beatmap::BeatmapRow;
use db::models::beatmaps::beatmapset::BeatmapsetRow;
use db::models::beatmaps::rates::RatesRow;
use db::models::rating::beatmap_mania_rating::BeatmapManiaRatingRow;
use db::models::rating::beatmap_rating::BeatmapRatingRow;
use sqlx::PgPool;

pub async fn find_full_by_osu_id(pool: &PgPool, osu_id: i32) -> Result<Option<Beatmapset>, sqlx::Error> {
    let Some(set_row) = BeatmapsetRow::find_by_osu_id(pool, osu_id).await? else {
        return Ok(None);
    };

    // Load beatmaps for this set
    let beatmaps_rows: Vec<BeatmapRow> = sqlx::query_as!(
        BeatmapRow,
        r#"
        SELECT id, osu_id, beatmapset_id, difficulty, count_circles, count_sliders, count_spinners, max_combo, main_pattern, cs, ar, od, hp, mode, status, created_at, updated_at
        FROM beatmap
        WHERE beatmapset_id = $1
        ORDER BY id ASC
        "#,
        set_row.id
    )
    .fetch_all(pool)
    .await?;

    let mut dto_set = Beatmapset {
        id: Some(set_row.id),
        osu_id: set_row.osu_id,
        artist: set_row.artist,
        artist_unicode: set_row.artist_unicode,
        title: set_row.title,
        title_unicode: set_row.title_unicode,
        creator: set_row.creator,
        source: set_row.source,
        tags: set_row
            .tags
            .and_then(|v| if v.is_empty() { None } else { Some(v.join(" ")) }),
        has_video: set_row.has_video,
        has_storyboard: set_row.has_storyboard,
        is_explicit: set_row.is_explicit,
        is_featured: set_row.is_featured,
        cover_url: set_row.cover_url,
        preview_url: set_row.preview_url,
        osu_file_url: set_row.osu_file_url,
        beatmaps: Vec::new(),
    };

    for b in beatmaps_rows {
        // Load rates for beatmap
        let rates_rows: Vec<RatesRow> = sqlx::query_as!(
            RatesRow,
            r#"
            SELECT id, beatmap_id, osu_hash, centirate, drain_time, total_time, bpm, created_at
            FROM rates
            WHERE beatmap_id = $1
            ORDER BY centirate ASC
            "#,
            b.id
        )
        .fetch_all(pool)
        .await?;

        let mut dto_b = Beatmap {
            id: Some(b.id),
            osu_id: b.osu_id,
            beatmapset_id: b.beatmapset_id,
            difficulty: b.difficulty,
            count_circles: b.count_circles,
            count_sliders: b.count_sliders,
            count_spinners: b.count_spinners,
            max_combo: b.max_combo,
            cs: b.cs.to_f64().unwrap_or_default(),
            ar: b.ar.to_f64().unwrap_or_default(),
            od: b.od.to_f64().unwrap_or_default(),
            hp: b.hp.to_f64().unwrap_or_default(),
            mode: b.mode,
            status: b.status,
            main_pattern: b.main_pattern,
            rates: Vec::new(),
        };

        for r in rates_rows {
            // Load ratings for rates
            let rating_rows: Vec<BeatmapRatingRow> = sqlx::query_as!(
                BeatmapRatingRow,
                r#"
                SELECT id, rates_id, rating, rating_type, created_at
                FROM beatmap_rating
                WHERE rates_id = $1
                ORDER BY id ASC
                "#,
                r.id
            )
            .fetch_all(pool)
            .await?;

            let mut dto_r = Rates {
                id: Some(r.id),
                osu_hash: Some(r.osu_hash),
                centirate: r.centirate,
                drain_time: r.drain_time,
                total_time: r.total_time,
                bpm: r.bpm.to_f32().unwrap_or_default(),
                rating: Vec::new(),
            };

            for ra in rating_rows {
                // Load mania rating if exists
                let mania_row: Option<BeatmapManiaRatingRow> = sqlx::query_as!(
                    BeatmapManiaRatingRow,
                    r#"
                    SELECT id, rating_id, stream, jumpstream, handstream, stamina, jackspeed, chordjack, technical, created_at, updated_at
                    FROM beatmap_mania_rating
                    WHERE rating_id = $1
                    LIMIT 1
                    "#,
                    ra.id
                )
                .fetch_optional(pool)
                .await?;

                let mode_rating = if let Some(m) = mania_row {
                    ModeRating::Mania(ManiaRating {
                        id: Some(m.id),
                        stream: m.stream.and_then(|v| v.to_f64()).unwrap_or_default(),
                        jumpstream: m.jumpstream.and_then(|v| v.to_f64()).unwrap_or_default(),
                        handstream: m.handstream.and_then(|v| v.to_f64()).unwrap_or_default(),
                        stamina: m.stamina.and_then(|v| v.to_f64()).unwrap_or_default(),
                        jackspeed: m.jackspeed.and_then(|v| v.to_f64()).unwrap_or_default(),
                        chordjack: m.chordjack.and_then(|v| v.to_f64()).unwrap_or_default(),
                        technical: m.technical.and_then(|v| v.to_f64()).unwrap_or_default(),
                    })
                } else {
                    ModeRating::Mania(ManiaRating {
                        id: None,
                        stream: 0.0,
                        jumpstream: 0.0,
                        handstream: 0.0,
                        stamina: 0.0,
                        jackspeed: 0.0,
                        chordjack: 0.0,
                        technical: 0.0,
                    })
                };

                dto_r.rating.push(Rating {
                    id: Some(ra.id),
                    rates_id: ra.rates_id,
                    rating: ra.rating.to_f64().unwrap_or_default(),
                    rating_type: ra.rating_type,
                    mode_rating,
                });
            }

            dto_b.rates.push(dto_r);
        }

        dto_set.beatmaps.push(dto_b);
    }

    Ok(Some(dto_set))
}


