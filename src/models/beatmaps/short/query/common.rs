use crate::filters::Filters;
use crate::models::beatmaps::short::types::Beatmapset;
use bigdecimal::{BigDecimal, ToPrimitive};
use serde_json::json;
use sqlx::{Postgres, QueryBuilder};
use std::collections::HashMap;

/// Determine the preferred rating type from filters or fallback to "osu".
pub fn preferred_rating_type(filters: &Filters) -> &str {
    filters
        .rating
        .as_ref()
        .and_then(|r| r.rating_type.as_deref())
        .unwrap_or("osu")
}

/// Compute a comparable score for a beatmap given the preferred rating type.
/// Fallback to `osu` if the preferred type is unavailable; otherwise return -inf.
pub fn beatmap_score(
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
pub fn sort_and_limit_beatmaps(beatmapsets: &mut HashMap<i32, Beatmapset>, preferred_type: &str) {
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

/// Apply filters to a QueryBuilder - used by both find_all_with_filters and find_random_with_filters
pub fn apply_filters<'a>(builder: &mut QueryBuilder<'a, Postgres>, filters: &'a Filters) {
    if let Some(rating) = filters.rating.as_ref() {
        if let Some(rt) = rating.rating_type.as_ref() {
            builder.push(" AND br.rating_type = ").push_bind(rt);
        }
        if let Some(min) = rating.rating_min.as_ref() {
            builder.push(" AND br.rating >= ").push_bind(min);
        }
        if let Some(max) = rating.rating_max.as_ref() {
            builder.push(" AND br.rating <= ").push_bind(max);
        }
    }
    if let Some(beatmap) = filters.beatmap.as_ref() {
        if let Some(term) = beatmap.search_term.as_ref() {
            let like = format!("%{}%", term);
            builder
                .push(" AND (bs.artist ILIKE ")
                .push_bind(like.clone())
                .push(" OR bs.title ILIKE ")
                .push_bind(like.clone())
                .push(" OR bs.creator ILIKE ")
                .push_bind(like)
                .push(")");
        }
        if let Some(min) = beatmap.total_time_min.as_ref() {
            builder.push(" AND r.total_time >= ").push_bind(min);
        }
        if let Some(max) = beatmap.total_time_max.as_ref() {
            builder.push(" AND r.total_time <= ").push_bind(max);
        }
        if let Some(min) = beatmap.bpm_min.as_ref() {
            builder.push(" AND r.bpm >= ").push_bind(min);
        }
        if let Some(max) = beatmap.bpm_max.as_ref() {
            builder.push(" AND r.bpm <= ").push_bind(max);
        }
    }
    if let Some(bt) = filters.beatmap_technical.as_ref() {
        if let Some(min) = bt.od_min.as_ref() {
            builder.push(" AND b.od >= ").push_bind(min);
        }
        if let Some(max) = bt.od_max.as_ref() {
            builder.push(" AND b.od <= ").push_bind(max);
        }
        if let Some(status) = bt.status.as_ref() {
            builder.push(" AND b.status = ").push_bind(status);
        }
    }
    if let Some(skill) = filters.skillset.as_ref() {
        if let Some(pattern_type) = skill.pattern_type.as_ref() {
            // JSONB array contains optimization: b.main_pattern @> '["pattern"]'
            let arr = json!([pattern_type]);
            builder.push(" AND b.main_pattern @> ").push_bind(arr);
            if let Some(min) = skill.pattern_min.as_ref() {
                match pattern_type.as_str() {
                    "jumpstream" => {
                        builder.push(" AND bmr.jumpstream >= ").push_bind(min);
                    }
                    "stream" => {
                        builder.push(" AND bmr.stream >= ").push_bind(min);
                    }
                    "handstream" => {
                        builder.push(" AND bmr.handstream >= ").push_bind(min);
                    }
                    "stamina" => {
                        builder.push(" AND bmr.stamina >= ").push_bind(min);
                    }
                    "jackspeed" => {
                        builder.push(" AND bmr.jackspeed >= ").push_bind(min);
                    }
                    "chordjack" => {
                        builder.push(" AND bmr.chordjack >= ").push_bind(min);
                    }
                    "technical" => {
                        builder.push(" AND bmr.technical >= ").push_bind(min);
                    }
                    _ => {}
                }
            }
            if let Some(max) = skill.pattern_max.as_ref() {
                match pattern_type.as_str() {
                    "jumpstream" => {
                        builder.push(" AND bmr.jumpstream <= ").push_bind(max);
                    }
                    "stream" => {
                        builder.push(" AND bmr.stream <= ").push_bind(max);
                    }
                    "handstream" => {
                        builder.push(" AND bmr.handstream <= ").push_bind(max);
                    }
                    "stamina" => {
                        builder.push(" AND bmr.stamina <= ").push_bind(max);
                    }
                    "jackspeed" => {
                        builder.push(" AND bmr.jackspeed <= ").push_bind(max);
                    }
                    "chordjack" => {
                        builder.push(" AND bmr.chordjack <= ").push_bind(max);
                    }
                    "technical" => {
                        builder.push(" AND bmr.technical <= ").push_bind(max);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Group rows by beatmapset - shared logic between find_all_with_filters and find_random_with_filters
pub fn group_beatmapset_rows(
    rows: Vec<sqlx::postgres::PgRow>,
) -> Result<HashMap<i32, Beatmapset>, sqlx::Error> {
    use serde_json::json;
    use sqlx::Row;

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
                total_beatmaps: 0,
                beatmaps: Vec::new(),
            });

        // Find or create beatmap
        let beatmap_osu_id: Option<i32> = row.try_get("beatmap_osu_id").ok();
        let beatmap_exists = beatmapset
            .beatmaps
            .iter()
            .any(|b| b.osu_id == beatmap_osu_id);

        if !beatmap_exists {
            beatmapset
                .beatmaps
                .push(crate::models::beatmaps::short::types::Beatmap {
                    osu_id: beatmap_osu_id,
                    difficulty: row.try_get("difficulty").unwrap_or_default(),
                    mode: row.try_get("mode").unwrap_or(0),
                    status: row.try_get("status").unwrap_or_default(),
                    main_pattern: row.try_get("main_pattern").unwrap_or(json!({})),
                    ratings: Vec::new(),
                });
        }

        let beatmap = beatmapset
            .beatmaps
            .iter_mut()
            .find(|b| b.osu_id == beatmap_osu_id)
            .unwrap();

        // Add rating
        let rating_bd: BigDecimal = row
            .try_get("rating")
            .unwrap_or_else(|_| BigDecimal::from(0));
        let rating_type: String = row.try_get("rating_type").unwrap_or_default();
        beatmap
            .ratings
            .push(crate::models::beatmaps::short::types::Rating {
                rating: rating_bd.to_f64().unwrap_or(0.0),
                rating_type,
            });
    }

    Ok(beatmapsets)
}
