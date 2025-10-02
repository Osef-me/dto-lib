use crate::models::beatmaps::full::types::{
    Beatmap, Beatmapset,
};
use crate::models::rate::{ManiaRating, ModeRating, Rates, Rating};
use bigdecimal::ToPrimitive;
use sqlx::PgPool;
use std::collections::HashMap;

pub async fn find_full_by_osu_id(
    pool: &PgPool,
    osu_id: i32,
) -> Result<Option<Beatmapset>, sqlx::Error> {
    // Single SQL with joins, restrict rates to centirate = 100
    let rows = sqlx::query!(
        r#"
        SELECT
            -- beatmapset
            bs.id                    AS bs_id,
            bs.osu_id                AS bs_osu_id,
            bs.artist                AS bs_artist,
            bs.artist_unicode        AS bs_artist_unicode,
            bs.title                 AS bs_title,
            bs.title_unicode         AS bs_title_unicode,
            bs.creator               AS bs_creator,
            bs.source                AS bs_source,
            bs.tags                  AS bs_tags,
            bs.has_video             AS bs_has_video,
            bs.has_storyboard        AS bs_has_storyboard,
            bs.is_explicit           AS bs_is_explicit,
            bs.is_featured           AS bs_is_featured,
            bs.cover_url             AS bs_cover_url,
            bs.preview_url           AS bs_preview_url,
            bs.osu_file_url          AS bs_osu_file_url,
            bs.osu_status_changed_at AS bs_osu_status_changed_at,
            -- beatmap
            b.id                     AS b_id,
            b.osu_id                 AS b_osu_id,
            b.beatmapset_id          AS b_beatmapset_id,
            b.difficulty             AS b_difficulty,
            b.count_circles          AS b_count_circles,
            b.count_sliders          AS b_count_sliders,
            b.count_spinners         AS b_count_spinners,
            b.max_combo              AS b_max_combo,
            b.main_pattern           AS b_main_pattern,
            b.cs                     AS b_cs,
            b.ar                     AS b_ar,
            b.od                     AS b_od,
            b.hp                     AS b_hp,
            b.mode                   AS b_mode,
            b.status                 AS b_status,

            -- rates (centirate=100)
            r.id                     AS r_id,
            r.osu_hash               AS r_osu_hash,
            r.centirate              AS r_centirate,
            r.drain_time             AS r_drain_time,
            r.total_time             AS r_total_time,
            r.bpm                    AS r_bpm,

            -- rating
            br.id                    AS br_id,
            br.rates_id              AS br_rates_id,
            br.rating                AS br_rating,
            br.rating_type           AS br_rating_type,

            -- mania rating (optional)
            bmr.id                   AS bmr_id,
            bmr.stream               AS bmr_stream,
            bmr.jumpstream           AS bmr_jumpstream,
            bmr.handstream           AS bmr_handstream,
            bmr.stamina              AS bmr_stamina,
            bmr.jackspeed            AS bmr_jackspeed,
            bmr.chordjack            AS bmr_chordjack,
            bmr.technical            AS bmr_technical
        FROM beatmapset bs
        INNER JOIN beatmap b ON bs.id = b.beatmapset_id
        LEFT JOIN rates r ON r.beatmap_id = b.id AND r.centirate = 100
        LEFT JOIN beatmap_rating br ON br.rates_id = r.id
        LEFT JOIN beatmap_mania_rating bmr ON bmr.rating_id = br.id
        WHERE bs.osu_id = $1
        ORDER BY b.id ASC, r.id ASC, br.id ASC
        "#,
        osu_id
    )
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Ok(None);
    }

    // Build Beatmapset
    let first = &rows[0];
    let mut dto_set = Beatmapset {
        id: Some(first.bs_id),
        osu_id: first.bs_osu_id,
        artist: first.bs_artist.clone(),
        artist_unicode: first.bs_artist_unicode.clone(),
        title: first.bs_title.clone(),
        title_unicode: first.bs_title_unicode.clone(),
        creator: first.bs_creator.clone(),
        source: first.bs_source.clone(),
        tags: first
            .bs_tags
            .as_ref()
            .map(|v| if v.is_empty() { None } else { Some(v.join(" ")) })
            .flatten(),
        has_video: first.bs_has_video,
        has_storyboard: first.bs_has_storyboard,
        is_explicit: first.bs_is_explicit,
        is_featured: first.bs_is_featured,
        cover_url: first.bs_cover_url.clone(),
        preview_url: first.bs_preview_url.clone(),
        osu_file_url: first.bs_osu_file_url.clone(),
        beatmaps: Vec::new(),
        osu_status_changed_at: first.bs_osu_status_changed_at.clone(),
    };

    // Group by beatmap id, then attach a single rate (centirate=100) and its ratings
    let mut beatmap_map: HashMap<i32, usize> = HashMap::new();
    // For each beatmap, we will maintain at most one Rates
    let mut rates_by_beatmap: HashMap<i32, Rates> = HashMap::new();

    for row in rows {
        // Ensure beatmap exists in dto_set
        let b_id = row.b_id;
        {
            if !beatmap_map.contains_key(&b_id) {
                let dto_b = Beatmap {
                    id: Some(row.b_id),
                    osu_id: row.b_osu_id,
                    beatmapset_id: row.b_beatmapset_id,
                    difficulty: row.b_difficulty.clone(),
                    count_circles: row.b_count_circles,
                    count_sliders: row.b_count_sliders,
                    count_spinners: row.b_count_spinners,
                    max_combo: row.b_max_combo,
                    cs: row.b_cs.to_f64().unwrap_or_default(),
                    ar: row.b_ar.to_f64().unwrap_or_default(),
                    od: row.b_od.to_f64().unwrap_or_default(),
                    hp: row.b_hp.to_f64().unwrap_or_default(),
                    mode: row.b_mode,
                    status: row.b_status.clone(),
                    main_pattern: row.b_main_pattern.clone(),
                    rates: Vec::new(),
                };
                let idx = dto_set.beatmaps.len();
                dto_set.beatmaps.push(dto_b);
                beatmap_map.insert(b_id, idx);
            }

            // Rates row may be NULL if no centirate=100 exists
            {
                let r_id = row.r_id;
                // Initialize rate per beatmap
                let entry = rates_by_beatmap.entry(b_id).or_insert_with(|| Rates {
                    id: Some(r_id),
                    osu_hash: Some(row.r_osu_hash.clone()),
                    centirate: row.r_centirate,
                    drain_time: row.r_drain_time,
                    total_time: row.r_total_time,
                    bpm: row.r_bpm.to_f32().unwrap_or_default(),
                    rating: Vec::new(),
                });

                // Attach rating
                {
                    let br_id = row.br_id;
                    let br_rates_id = row.br_rates_id.unwrap_or(r_id);
                    let br_rating = row.br_rating;
                    let br_type = row.br_rating_type.clone();
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

                    entry.rating.push(Rating {
                        id: Some(br_id),
                        rates_id: Some(br_rates_id),
                        rating: br_rating.to_f64().unwrap_or_default(),
                        rating_type: br_type,
                        mode_rating,
                    });
                }
            }
        }
    }

    // Move the single rate into each beatmap if present
    for (b_id, idx) in beatmap_map {
        if let Some(r) = rates_by_beatmap.remove(&b_id) {
            if let Some(b) = dto_set.beatmaps.get_mut(idx) {
                b.rates.push(r);
            }
        }
    }

    Ok(Some(dto_set))
}

pub async fn find_ratings_by_osu_id_and_centirate(
    pool: &PgPool,
    beatmap_osu_id: i32,
    centirate: i32,
) -> Result<Vec<Rating>, sqlx::Error> {
    use bigdecimal::ToPrimitive;
    let rows = sqlx::query!(
        r#"
        SELECT
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
        INNER JOIN beatmap_rating br ON br.rates_id = r.id
        LEFT JOIN beatmap_mania_rating bmr ON bmr.rating_id = br.id
        WHERE b.osu_id = $1
        ORDER BY br.id ASC
        "#,
        beatmap_osu_id,
        centirate
    )
    .fetch_all(pool)
    .await?;

    let mut ratings = Vec::with_capacity(rows.len());
    for row in rows {
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
        ratings.push(Rating {
            id: Some(row.br_id),
            rates_id: row.br_rates_id,
            rating: row.br_rating.to_f64().unwrap_or_default(),
            rating_type: row.br_rating_type,
            mode_rating,
        });
    }
    Ok(ratings)
}

pub async fn find_rate_by_osu_id_and_centirate(
    pool: &PgPool,
    beatmap_osu_id: i32,
    centirate: i32,
) -> Result<Option<Rates>, sqlx::Error> {
    use bigdecimal::ToPrimitive;
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