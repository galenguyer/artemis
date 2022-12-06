use regex::Regex;
use sqlx::sqlite::SqlitePool;
use std::fs;

mod fcc_date;
mod file;
mod load;
mod types;
use file::{download_file, unzip_file};

const WEEKLY_DUMP_URL: &str = "https://data.fcc.gov/download/pub/uls/complete/l_amat.zip";
const SPECIAL_CONDITIONS_URL: &str = "https://www.fcc.gov/file/20669/download";

async fn load_weekly(db: &SqlitePool) {
    let output_file =
        download_file(WEEKLY_DUMP_URL, None).expect("Error downloading weekly dump file");
    // Hardcoding this file name because it might change and I don't want to deal with that
    let _conditions_file =
        download_file(SPECIAL_CONDITIONS_URL, Some("special_condition_codes.txt"))
            .expect("Error downloading Special Conditions file");

    unzip_file(output_file).expect("Error unzipping file");

    // Some idiot at the FCC decided that unescaped newlines in the middle of a field were cool
    // Uncle Ted may have had some good ideas after all
    let comments_regex = Regex::new(r"\s*\r\r\n").unwrap();
    let comments = fs::read_to_string("CO.dat").expect("Error reading file");
    fs::write(
        "CO.dat",
        comments_regex.replace_all(&comments, " ").to_string(),
    )
    .expect("Error writing file");

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

    load::load_amateurs(&db).await;
    load::load_comments(&db).await;
    load::load_entities(&db).await;
    load::load_headers(&db).await;
    load::load_history(&db).await;
    load::load_license_attachments(&db).await;
    load::load_special_conditions(&db).await;
    load::load_special_conditions_free_form(&db).await;

    load::load_special_condition_codes(&db).await;
}

#[tokio::main]
async fn main() {
    let db = SqlitePool::connect("sqlite://fcc.db")
        .await
        .expect("Error connecting to database");

    load_weekly(&db).await;
}
