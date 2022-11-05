use chrono::DateTime;
use filetime::{self, FileTime};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use serde::Deserialize;
use sqlx::sqlite::SqlitePool;
use sqlx::{Executor, Pool, Sqlite, Statement};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::sync::Arc;

const WEEKLY_DUMP_URL: &str = "https://data.fcc.gov/download/pub/uls/complete/l_amat.zip";
const INSERT_ENTITY_SQL: &str = r"INSERT INTO entities (record_type, unique_system_identifier, uls_file_number, ebf_number, call_sign, entity_type, licensee_id, entity_name, first_name, mi, last_name, suffix, phone, fax, email, street_address, city, state, zip_code, po_box, attention_line, sgin, frn, applicant_type_code, applicant_type_other, status_code, status_date, lic_category_code, linked_license_id, linked_callsign) VALUES ('EN', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

#[allow(dead_code, non_snake_case)]
#[derive(Deserialize, Debug)]
struct Entity<'a> {
    pub RecordType: &'a str,
    pub UniqueSystemIdentifier: u32,
    pub UlsFileNumber: &'a str,
    pub EBFNumber: &'a str,
    pub CallSign: &'a str,
    pub EntityType: &'a str,
    pub LicenseeId: &'a str,
    pub EntityName: &'a str,
    pub FirstName: &'a str,
    pub MiddleInitial: &'a str,
    pub LastName: &'a str,
    pub Suffix: &'a str,
    pub Phone: &'a str,
    pub Fax: &'a str,
    pub Email: &'a str,
    pub StreetAddress: &'a str,
    pub City: &'a str,
    pub State: &'a str,
    pub ZipCode: &'a str,
    pub POBox: &'a str,
    pub AttentionLine: &'a str,
    pub SGIN: &'a str,
    pub FRN: &'a str,
    pub ApplicantTypeCode: &'a str,
    pub ApplicantTypeCodeOther: &'a str,
    pub StatusCode: &'a str,
    pub StatusDate: &'a str,
    pub ThreePointSevenGhzLicenseType: &'a str,
    pub LinkedUniqueSystemIdentifier: &'a str,
    pub LinkedCallsign: &'a str,
}

fn download_file() -> Result<File, ()> {
    let resp = ureq::get(WEEKLY_DUMP_URL)
        .call()
        .expect("Error downloading file");

    // We can work on handling not having a Content-Length header later
    assert!(resp.has("Content-Length"));
    let len: u64 = resp
        .header("Content-Length")
        .unwrap()
        .parse()
        .expect("Error parsing Content-Length header");

    let last_modified = DateTime::parse_from_rfc2822(
        resp.header("Last-Modified")
            .expect("Error getting Last-Modified header"),
    )
    .expect("Error parsing Last-Modified header")
    .timestamp();

    let output_file_name = "l_amat.zip";

    if std::path::Path::new(output_file_name).exists() {
        let file_metadata = fs::metadata(output_file_name).expect("Error getting file metadata");
        let mtime = FileTime::from_last_modification_time(&file_metadata);

        match (mtime.seconds() >= last_modified, file_metadata.len() == len) {
            (true, true) => {
                println!("File already downloaded");
                return Ok(File::open(output_file_name).expect("Error opening file"));
            }
            (true, false) => {
                println!("File already downloaded, but is incomplete");
            }
            (false, _) => {
                println!("File already downloaded, but is out of date");
            }
        }
    } else {
        println!("File does not exist, downloading");
    }

    let mut output_file = fs::File::create(output_file_name).expect("Error creating output file");

    let mut reader = resp.into_reader();
    let chunk_size = len / 99;

    let progress_bar = ProgressBar::new(len);
    progress_bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(1));
    progress_bar.set_message(output_file_name);
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed}+{eta}/{duration}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}) {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    loop {
        let mut chunk = vec![0u8; chunk_size as usize];
        let bytes_read = reader.read(&mut chunk[..]).expect("Error reading chunk");
        chunk.truncate(bytes_read); // This way we don't end with a ton of leading 0s
        if bytes_read > 0 {
            output_file
                .write_all(chunk.as_slice())
                .expect("Error writing to output file");

            progress_bar.inc(bytes_read as u64);
        } else {
            break;
        }
    }

    output_file.flush().expect("Error flushing output file");
    progress_bar.finish();

    Ok(output_file)
}

fn unzip_file(zip_file: File) -> Result<(), ()> {
    let mut archive = zip::ZipArchive::new(zip_file).expect("Error opening zip archive");

    let progress_bar = ProgressBar::new(archive.len().try_into().unwrap());
    // progress_bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(1));
    progress_bar.set_message("");
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed}+{eta}/{duration}] [{bar:40.cyan/blue}] {human_pos}/{human_len} {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .expect("Error getting file from archive");
        let unzip_path = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };
        progress_bar.set_message(format!("{}", unzip_path.display()));

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&unzip_path).expect("Error creating directory");
        } else {
            if let Some(p) = unzip_path.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).expect("Error creating directory");
                }
            }
            let mut unzip_file = fs::File::create(&unzip_path).expect("Error creating file");
            std::io::copy(&mut file, &mut unzip_file).expect("Error copying file");
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&unzip_path, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
        // TODO: Also set and check file mtime
        progress_bar.set_position((i + 1).try_into().unwrap());
    }

    progress_bar.finish();
    Ok(())
}

async fn load_entities(db: Pool<Sqlite>) {
    let entities_file = File::open("EN.dat").expect("Error opening file");
    let entities_file_meta = fs::metadata("EN.dat").expect("Error getting file metadata");
    let mut transaction = db.begin().await.expect("Error starting transaction");
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'|')
        .quoting(false)
        .from_reader(entities_file);

    // let statement = sqlx::query(INSERT_ENTRY_SQL);

    for line in reader.records() {
        let line = line.expect("Error reading entry");
        let entity: Entity = line.deserialize(None).expect("Error deserializing entry");
        let statement = sqlx::query(INSERT_ENTITY_SQL);
        statement
            .bind(entity.UniqueSystemIdentifier)
            .bind(entity.UlsFileNumber)
            .bind(entity.EBFNumber)
            .bind(entity.CallSign)
            .bind(entity.EntityType)
            .bind(entity.LicenseeId)
            .bind(entity.EntityName)
            .bind(entity.FirstName)
            .bind(entity.MiddleInitial)
            .bind(entity.LastName)
            .bind(entity.Suffix)
            .bind(entity.Phone)
            .bind(entity.Fax)
            .bind(entity.Email)
            .bind(entity.StreetAddress)
            .bind(entity.City)
            .bind(entity.State)
            .bind(entity.ZipCode)
            .bind(entity.POBox)
            .bind(entity.AttentionLine)
            .bind(entity.SGIN)
            .bind(entity.FRN)
            .bind(entity.ApplicantTypeCode)
            .bind(entity.ApplicantTypeCodeOther)
            .bind(entity.StatusCode)
            .bind(entity.StatusDate)
            .bind(entity.ThreePointSevenGhzLicenseType)
            .bind(entity.LinkedUniqueSystemIdentifier)
            .bind(entity.LinkedCallsign)
            .execute(&mut transaction)
            .await
            .expect("Error executing statement");
        // println!("{:?}", entity);
    }

    transaction
        .commit()
        .await
        .expect("Error committing transaction");
}

#[tokio::main]
async fn main() {
    // let output_file = download_file().expect("Error downloading file");

    // unzip_file(output_file).expect("Error unzipping file");

    let db = SqlitePool::connect("sqlite://fcc.db")
        .await
        .expect("Error connecting to database");
    load_entities(db).await;
}
