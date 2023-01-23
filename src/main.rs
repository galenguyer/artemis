use chrono::{DateTime, Utc};
use regex::Regex;
use sqlx::sqlite::SqlitePool;
use std::{fs, os::unix::prelude::MetadataExt, time::Duration};

mod fcc_date;
mod file;
mod load;
mod meta;
mod types;
use file::{download_file, unzip_file};
use types::Update;

const WEEKLY_DUMP_URL: &str = "https://data.fcc.gov/download/pub/uls/complete/l_amat.zip";
const SUNDAY_DUMP_URL: &str = "https://data.fcc.gov/download/pub/uls/daily/l_am_sun.zip";
const MONDAY_DUMP_URL: &str = "https://data.fcc.gov/download/pub/uls/daily/l_am_mon.zip";
const TUESDAY_DUMP_URL: &str = "https://data.fcc.gov/download/pub/uls/daily/l_am_tue.zip";
const WEDNESDAY_DUMP_URL: &str = "https://data.fcc.gov/download/pub/uls/daily/l_am_wed.zip";
const THURSDAY_DUMP_URL: &str = "https://data.fcc.gov/download/pub/uls/daily/l_am_thu.zip";
const FRIDAY_DUMP_URL: &str = "https://data.fcc.gov/download/pub/uls/daily/l_am_fri.zip";
const SATURDAY_DUMP_URL: &str = "https://data.fcc.gov/download/pub/uls/daily/l_am_sat.zip";
const SPECIAL_CONDITIONS_URL: &str = "https://www.fcc.gov/file/20669/download";

#[allow(dead_code)]
#[derive(Debug)]
struct FccUpdates {
    weekly: Option<DateTime<Utc>>,
    sunday: Option<DateTime<Utc>>,
    monday: Option<DateTime<Utc>>,
    tuesday: Option<DateTime<Utc>>,
    wednesday: Option<DateTime<Utc>>,
    thursday: Option<DateTime<Utc>>,
    friday: Option<DateTime<Utc>>,
    saturday: Option<DateTime<Utc>>,
}
impl FccUpdates {
    fn new() -> Self {
        Self {
            weekly: get_last_updated_header(WEEKLY_DUMP_URL),
            sunday: get_last_updated_header(SUNDAY_DUMP_URL),
            monday: get_last_updated_header(MONDAY_DUMP_URL),
            tuesday: get_last_updated_header(TUESDAY_DUMP_URL),
            wednesday: get_last_updated_header(WEDNESDAY_DUMP_URL),
            thursday: get_last_updated_header(THURSDAY_DUMP_URL),
            friday: get_last_updated_header(FRIDAY_DUMP_URL),
            saturday: get_last_updated_header(SATURDAY_DUMP_URL),
        }
    }

    fn get_pending(&self, last_update: DateTime<Utc>) -> Vec<(DateTime<Utc>, String)> {
        let mut pending = Vec::new();

        if let Some(sunday) = self.sunday {
            if sunday > last_update {
                pending.push((sunday, SUNDAY_DUMP_URL.to_string()));
            }
        }
        if let Some(monday) = self.monday {
            if monday > last_update {
                pending.push((monday, MONDAY_DUMP_URL.to_string()));
            }
        }
        if let Some(tuesday) = self.tuesday {
            if tuesday > last_update {
                pending.push((tuesday, TUESDAY_DUMP_URL.to_string()));
            }
        }
        if let Some(wednesday) = self.wednesday {
            if wednesday > last_update {
                pending.push((wednesday, WEDNESDAY_DUMP_URL.to_string()));
            }
        }
        if let Some(thursday) = self.thursday {
            if thursday > last_update {
                pending.push((thursday, THURSDAY_DUMP_URL.to_string()));
            }
        }
        if let Some(friday) = self.friday {
            if friday > last_update {
                pending.push((friday, FRIDAY_DUMP_URL.to_string()));
            }
        }
        if let Some(saturday) = self.saturday {
            if saturday > last_update {
                pending.push((saturday, SATURDAY_DUMP_URL.to_string()));
            }
        }

        pending.sort_by(|a, b| a.0.cmp(&b.0));

        pending
    }
}

fn get_last_updated_header(url: &str) -> Option<DateTime<Utc>> {
    let resp = ureq::head(url).call().expect("Error downloading file");

    assert!(resp.has("Content-Length"));
    let len = resp
        .header("Content-Length")
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);

    // This is the size given when there's no updates
    if len <= 211 {
        return None;
    }

    match resp.header("Last-Modified") {
        Some(last_mod) => match DateTime::parse_from_rfc2822(last_mod) {
            Ok(dt) => Some(dt.into()),
            Err(_) => None,
        },
        None => None,
    }
}

async fn load_weekly(db: &SqlitePool) -> chrono::DateTime<Utc> {
    let output_file =
        download_file(WEEKLY_DUMP_URL, None).expect("Error downloading weekly dump file");
    // Hardcoding this file name because it might change and I don't want to deal with that
    let _conditions_file =
        download_file(SPECIAL_CONDITIONS_URL, Some("special_condition_codes.txt"))
            .expect("Error downloading Special Conditions file");

    unzip_file(&output_file).expect("Error unzipping file");
    std::fs::remove_file("counts").expect("Error deleting counts file");

    // This is somehow worse, newlines can either be \n (more common) OR \r\n.
    // The first one is easy, if there's a newline without a preceeding carriage return, it's bad and should be gone
    // CRLF is what's normally used, however the last character of every entry is either R, P, T, or |, so if there's a CRLF
    // without one of those immediately before, yeet it
    let conditions_regex = Regex::new(r"(([^\r]\n)|([^RPT\|]\r\n))").unwrap();
    let conditions = fs::read_to_string("special_condition_codes.txt").expect("Error reading file");
    fs::write(
        "special_condition_codes.txt",
        conditions_regex.replace_all(&conditions, " ").to_string(),
    )
    .expect("Error writing file");

    load::load_amateurs(db, true).await;
    load::load_comments(db, true).await;
    load::load_entities(db, true).await;
    load::load_headers(db, true).await;
    load::load_history(db, true).await;
    load::load_license_attachments(db, true).await;
    load::load_special_conditions(db, true).await;
    load::load_special_conditions_free_form(db, true).await;

    load::load_special_condition_codes(db, true).await;

    let meta = output_file.metadata().unwrap();
    // std::fs::remove_file("l_amat.zip").expect("Error deleting l_amat.zip");
    DateTime::<Utc>::from(
        std::time::UNIX_EPOCH + Duration::from_secs(meta.mtime().try_into().unwrap()),
    )
}


async fn load_daily(url: &str, db: &SqlitePool) -> chrono::DateTime<Utc> {
    let parse_file_name_from_url = |url: &str| {
        let output_file_name_regex = Regex::new(r"/(\w+\.?\w*)").expect("Error constructing regex");
        let Some(file_name_captures) = output_file_name_regex.captures_iter(url).last() else {
            panic!("Error parsing file name from URL");
        };
        let Some(maybe_match) = file_name_captures.iter().last() else {
            panic!("Error parsing file name from URL");
        };
        let Some(file_name_match) = maybe_match else {
            panic!("Error parsing file name from URL");
        };
        String::from(file_name_match.as_str())
    };

    let output_file =
        download_file(url, None).expect("Error downloading weekly dump file");

    unzip_file(&output_file).expect("Error unzipping file");
    std::fs::remove_file("counts").expect("Error deleting counts file");

    load::load_amateurs(db, false).await;
    load::load_comments(db, false).await;
    load::load_entities(db, false).await;
    load::load_headers(db, false).await;
    load::load_history(db, false).await;
    load::load_license_attachments(db, false).await;
    load::load_special_conditions(db, false).await;
    load::load_special_conditions_free_form(db, false).await;

    let meta = output_file.metadata().unwrap();

    // let file_name = parse_file_name_from_url(url);
    // std::fs::remove_file(&file_name).unwrap_or_else(|_| panic!("Error deleting {}", file_name));
    DateTime::<Utc>::from(
        std::time::UNIX_EPOCH + Duration::from_secs(meta.mtime().try_into().unwrap()),
    )
}

#[tokio::main]
async fn main() {
    let db = SqlitePool::connect("sqlite://fcc.db")
        .await
        .expect("Error connecting to database");

    let fcc_updates = dbg!(FccUpdates::new());

    let last_weekly = meta::get_last_update(&db, meta::UpdateType::Weekly)
        .await
        .expect("Error getting last weekly update");

    // if this is the first time the database is being updated
    if let Some(last_weekly) = last_weekly {
        if fcc_updates.weekly.is_some() && fcc_updates.weekly.unwrap() > last_weekly.date {
            println!("New weekly update found, loading weekly dump");
            let update_date = load_weekly(&db).await;
            meta::insert_update(
                &db,
                &Update {
                    id: 0, // placeholder
                    daily: false,
                    weekly: true,
                    date: update_date,
                },
            )
            .await
            .expect("Error inserting weekly update");
        }
    } else {
        println!("No weekly updates found, loading weekly dump");
        let update_date = load_weekly(&db).await;
        meta::insert_update(
            &db,
            &Update {
                id: 0, // placeholder
                daily: false,
                weekly: true,
                date: update_date,
            },
        )
        .await
        .expect("Error inserting update");
    }

    let last_update = meta::get_last_update(&db, meta::UpdateType::Any)
        .await
        .expect("Error getting last update")
        .expect("No updates found");

    let pending = dbg!(fcc_updates.get_pending(dbg!(last_update.date)));
    for update in pending {
        let update_date = load_daily(&dbg!(update.1), &db).await;
        meta::insert_update(
            &db,
            &Update {
                id: 0, // placeholder
                daily: true,
                weekly: false,
                date: update_date,
            },
        )
        .await
        .expect("Error inserting daily update");
    }
}
