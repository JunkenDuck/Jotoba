use std::{
    cmp::Ordering,
    collections::HashMap,
    io::{stdout, Write},
};

use super::dict::Dict;
use crate::{
    error::Error,
    models::kanji::{find_readings_by_liteal, ReadingType},
    search::{query::KanjiReading, SearchMode},
    utils::{self, invert_ordering},
    DbPool,
};

#[cfg(feature = "tokenizer")]
use crate::JA_NL_PARSER;

use diesel::prelude::*;
use futures::future::try_join_all;
use itertools::Itertools;
use tokio_diesel::*;

/// Update kun/on reading compounds
pub async fn update_links(db: &DbPool) -> Result<(), Error> {
    use crate::schema::kanji::dsl::*;

    clear_links(db).await?;

    let all_kanji: Vec<(i32, String, Option<Vec<String>>, Option<Vec<String>>)> = kanji
        .select((id, literal, kunyomi, onyomi))
        .get_results_async(db)
        .await?;

    let mut dict_cache: HashMap<i32, Dict> = HashMap::new();
    print!("Generating kun readings... 0%");

    let all_kuns = all_kanji
        .iter()
        .filter_map(|i| {
            (i.2.is_some() && !i.2.as_ref().unwrap().is_empty())
                .then(|| (i.0, &i.1, i.2.as_ref().unwrap(), &i.3))
        })
        .enumerate()
        .filter_map(|(pos, (kanji_id, kanji_literal, kun_readings, _))| {
            // For every kanji in DB
            print!(
                "\rGenerating kun readings... {}%",
                pos * 100 / all_kanji.len()
            );
            utils::to_option(
                find_kun_readings(db, kanji_literal.clone(), &kun_readings, &mut dict_cache)
                    .unwrap_or_default(),
            )
            .map(|r| (kanji_id, r))
        })
        .collect::<Vec<(i32, Vec<_>)>>();

    println!();

    let mut kun_insert_counter = 0;
    for k in all_kuns.chunks(100).into_iter() {
        print!(
            "\rInserting kun readings: {}/{}",
            kun_insert_counter,
            all_kuns.len()
        );
        stdout().flush().ok();

        futures::future::try_join_all(
            k.iter()
                .map(|(k_id, dict_ids)| update_kun_link(db, *k_id, dict_ids)),
        )
        .await?;
        kun_insert_counter += k.len();
    }

    print!("\nUpdating on readings: 0/{}", all_kanji.len());
    stdout().flush().ok();

    let mut on_counter = 0;
    for kanji_on_chunk in all_kanji
        .iter()
        .filter_map(|i| {
            (i.3.is_some() && !i.3.as_ref().unwrap().is_empty())
                .then(|| (i.0, &i.1, i.3.as_ref().unwrap()))
        })
        .chunks(50)
        .into_iter()
    {
        print!("\rUpdating on readings: {}/{}", on_counter, all_kanji.len());
        stdout().flush().ok();

        try_join_all(
            kanji_on_chunk
                .into_iter()
                .map(|(kanji_id, kanji_literal, on_readings)| {
                    find_on_readings(db, kanji_id, kanji_literal.clone(), &on_readings)
                }),
        )
        .await?;

        on_counter += 50;
    }

    println!();
    Ok(())
}

/// Returns all kun reading compounds for a kanji given by its literal
async fn find_on_readings(
    db: &DbPool,
    id: i32,
    literal: String,
    ons: &[String], // All kanji on readings
) -> Result<(), Error> {
    let ons = try_join_all(ons.iter().map(|on| {
        find_readings_by_liteal(
            &literal,
            db,
            KanjiReading::new(&literal, on),
            ReadingType::Onyomi,
            SearchMode::Exact,
            true,
        )
    }))
    .await?
    .into_iter()
    .flatten()
    .collect_vec();

    if ons.is_empty() {
        return Ok(());
    }

    update_on_link(db, id, &ons.into_iter().take(9).collect_vec()).await?;

    Ok(())
}

/// Returns all kun reading compounds for a kanji given by its literal
fn find_kun_readings(
    db: &DbPool,
    literal: String,
    kun: &[String], // All kanji kun readings
    cache: &mut HashMap<i32, Dict>,
) -> Result<Vec<i32>, Error> {
    let db = db.get().unwrap();
    use crate::schema::dict::dsl::*;

    // Find all Dict-seq_ids starting with the literal
    let seq_ids: Vec<i32> = dict
        .select(sequence)
        .filter(reading.like(format!("{}%", literal)))
        .filter(kanji.eq(true))
        .get_results(&db)?;

    // Get precached
    let cached = seq_ids
        .iter()
        .filter_map(|i| cache.get(i).cloned())
        .collect_vec();

    let dicts: Vec<Dict> = dict
        .filter(
            sequence.eq_any(
                seq_ids
                    .iter()
                    .filter(|i| !cache.contains_key(i))
                    .collect_vec(),
            ),
        )
        .order_by(id)
        .get_results(&db)?;

    // add to cache
    for d in dicts.iter() {
        cache.insert(d.sequence, d.clone());
    }

    // Concat results + cached
    let dicts = dicts.into_iter().chain(cached).collect_vec();

    // result vec
    let mut kuns: Vec<Dict> = Vec::new();

    // Iterate over all dicts containing the literal
    for (_, val) in dicts.iter().group_by(|i| i.sequence).into_iter() {
        let (kanji_r, kana_r): (Vec<Dict>, Vec<Dict>) =
            val.into_iter().cloned().partition(|i| i.kanji);
        if kanji_r.is_empty() {
            continue;
        }
        // kana reading of curr dict
        let dict_kana = kana_r[0].clone();
        // kanji reading of curr dict
        let dict_kanji = kanji_r[0].clone();

        for ku in kun.iter() {
            if matches_kanji(&literal, ku, &dict_kana.reading, &dict_kanji.reading)
                && len(ku) <= dict_kana.len()
            {
                kuns.push(dict_kanji);
                break;
            }
        }
    }

    let clean_kuns = kun.iter().map(|i| literal_reading(i)).collect_vec();
    if kuns.len() > 10 {
        kuns.sort_by(|a, b| order_kuns(a, b, &clean_kuns));
        kuns.truncate(10);
    }

    Ok(kuns.iter().map(|i| i.sequence).collect())
}

fn order_kuns(a: &Dict, b: &Dict, clean_kuns: &Vec<String>) -> Ordering {
    let a_kunr = clean_kuns.contains(&a.reading);
    let b_kunr = clean_kuns.contains(&b.reading);

    if a_kunr && b_kunr {
        if let Some(order) = utils::get_item_order(&clean_kuns, &a.reading, &b.reading) {
            return order;
        }
    } else if a_kunr && !b_kunr {
        return Ordering::Less;
    } else if !a_kunr && b_kunr {
        return Ordering::Greater;
    }

    #[cfg(feature = "tokenizer")]
    let a_parsed = JA_NL_PARSER.parse(&a.reading).len();
    #[cfg(feature = "tokenizer")]
    let b_parsed = JA_NL_PARSER.parse(&b.reading).len();
    #[cfg(feature = "tokenizer")]
    if a_parsed == 1 && b_parsed > 0 {
        return Ordering::Less;
    } else if a_parsed > 1 && b_parsed == 0 {
        return Ordering::Greater;
    }

    let a_prio = a.priorities.as_ref().map(|i| i.len()).unwrap_or_default();
    let b_prio = b.priorities.as_ref().map(|i| i.len()).unwrap_or_default();
    if a_prio > 0 && b_prio == 0 {
        return Ordering::Less;
    } else if b_prio > 0 && a_prio == 0 {
        return Ordering::Greater;
    }

    let a_jlpt = a.jlpt_lvl;
    let b_jlpt = b.jlpt_lvl;

    if a_jlpt.is_some() && b_jlpt.is_none() {
        return Ordering::Less;
    } else if a_jlpt.is_none() && b_jlpt.is_some() {
        return Ordering::Greater;
    }

    if a_jlpt.is_some() && b_jlpt.is_some() {
        let a_jlpt = a_jlpt.unwrap();
        let b_jlpt = b_jlpt.unwrap();
        return invert_ordering(a_jlpt.cmp(&b_jlpt));
    }

    Ordering::Equal
}

pub fn len(kun: &str) -> usize {
    utils::real_string_len(&super::format_reading(kun))
}

pub fn literal_reading(kun: &str) -> String {
    kun.replace('-', "").split('.').next().unwrap().to_string()
}

async fn update_on_link(db: &DbPool, kanji_id: i32, dict_ids: &[i32]) -> Result<(), Error> {
    use crate::schema::kanji::dsl::*;
    diesel::update(kanji)
        .filter(id.eq(kanji_id))
        .set(on_dicts.eq(dict_ids))
        .execute_async(db)
        .await?;
    Ok(())
}

async fn update_kun_link(db: &DbPool, kanji_id: i32, dict_ids: &[i32]) -> Result<(), Error> {
    use crate::schema::kanji::dsl::*;
    diesel::update(kanji)
        .filter(id.eq(kanji_id))
        .set(kun_dicts.eq(dict_ids))
        .execute_async(db)
        .await?;
    Ok(())
}

fn matches_kanji(literal: &str, kun: &str, kana_reading: &str, kanji_reading: &str) -> bool {
    let match_mode = if kun.starts_with('-') {
        SearchMode::RightVariable
    } else if kun.ends_with('-') || kanji_reading.starts_with(&literal) {
        SearchMode::LeftVariable
    } else {
        SearchMode::Exact
    };

    let mut kanji_out = kun.to_string().replace('-', "");

    if kun.contains('.') {
        let kun_left = kun.split('.').next().unwrap();
        kanji_out = kanji_out.replace(&format!("{}.", kun_left), literal);
    } else {
        kanji_out = literal.to_owned();
    }

    let kanji_out = kanji_out.replace(literal, &literal_reading(kun));
    match_mode.str_eq(kana_reading, kanji_out.as_str(), false)
}

/// Clear existinig kun links
async fn clear_links(db: &DbPool) -> Result<(), Error> {
    use crate::schema::kanji::dsl::*;

    let empty: Option<Vec<i32>> = None;
    diesel::update(kanji)
        .set((kun_dicts.eq(&empty), on_dicts.eq(&empty)))
        .execute_async(&db)
        .await?;

    Ok(())
}