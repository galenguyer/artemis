use chrono::DateTime;
use filetime::{self, FileTime};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use regex::Regex;
use sqlx::sqlite::SqlitePool;
use std::fs::{self, File};
use std::io::{Read, Write};
mod fcc_date;
mod load;
mod types;

const WEEKLY_DUMP_URL: &str = "https://data.fcc.gov/download/pub/uls/complete/l_amat.zip";
const SPECIAL_CONDITIONS_URL: &str = "https://www.fcc.gov/file/20669/download";

/// Downloads a file from the given URL to the given path
///
/// # Arguments
///
/// * `url` - The URL to download from
/// * `path` - The path to download to. If None, try and use the Content-Disposition
/// header to determine the filename, and fall back to the last segment of the URL
///
/// # Examples
/// ```
/// download_file("https://data.fcc.gov/download/pub/uls/complete/l_amat.zip", None);
/// ```
fn download_file(url: &str, file_name: Option<&str>) -> Result<File, ()> {
    let resp = ureq::get(url).call().expect("Error downloading file");

    // We can work on handling not having a Content-Length header later
    assert!(resp.has("Content-Length"));
    let len: u64 = resp
        .header("Content-Length")
        .unwrap()
        .parse()
        .expect("Error parsing Content-Length header");

    let last_modified = match resp.header("Last-Modified") {
        Some(last_mod) => {
            match DateTime::parse_from_rfc2822(last_mod) {
                Ok(dt) => Some(dt.timestamp()),
                Err(_) => None,
            }
        }
        None => {
            None
        }
    };

    // Time to determine the file name!
    // Start by seeing if we were told anything, that makes it easy
    // This is just a helper. It should be its own function. lmao.
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
    let output_file_name = match file_name {
        Some(n) => String::from(n),
        None => {
            // We weren't given a file name by the user, so we need to figure it out ourself
            match resp.header("Content-Disposition") {
                // A Content-Disposition header is present, so we can use that
                Some(content_disposition) => {
                    let content_disposition_regex =
                        Regex::new(r#"filename="([\w\.]+)""#).expect("Error compiling regex");
                    // Check if the Content-Disposition header specifies a filename
                    match content_disposition_regex.captures(content_disposition) {
                        Some(cd_match) => {
                            // We have a filename, so use that
                            // TODO: Make less unwrappy
                            cd_match.iter().last().unwrap().unwrap().as_str().to_string()
                        }
                        None => {
                            // It doesn't, so we have to fall back to the file name in the URL
                            parse_file_name_from_url(url)
                        }
                    }
                }
                // No Content-Disposition header, so we have to fall back to the file name in the URL
                None => parse_file_name_from_url(url),
            }
        }
    };

    if std::path::Path::new(&output_file_name).exists() {
        let file_metadata = fs::metadata(&output_file_name).expect("Error getting file metadata");
        let mtime = FileTime::from_last_modification_time(&file_metadata);

        match (mtime.seconds() >= last_modified.unwrap_or(1), file_metadata.len() == len) {
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

    let mut output_file = fs::File::create(&output_file_name).expect("Error creating output file");

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

#[tokio::main]
async fn main() {
    let output_file = download_file(WEEKLY_DUMP_URL, None).expect("Error downloading weekly dump file");

    #[allow(unused_variables)]
    let conditions_file = download_file(SPECIAL_CONDITIONS_URL, None).expect("Error downloading Special Conditions file");

    unzip_file(output_file).expect("Error unzipping file");

    let db = SqlitePool::connect("sqlite://fcc.db")
        .await
        .expect("Error connecting to database");

    // Some idiot at the FCC decided that unescaped newlines in the middle of a field were cool
    // Uncle Ted may have had some good ideas after all
    let re = Regex::new(r"\s*\r\r\n").unwrap();
    let comments = fs::read_to_string("CO.dat").expect("Error reading file");
    fs::write("CO.dat", re.replace_all(&comments, " ").to_string()).expect("Error writing file");

    load::load_amateurs(&db).await;
    load::load_comments(&db).await;
    load::load_entities(&db).await;
    load::load_headers(&db).await;
    load::load_history(&db).await;
    load::load_license_attachments(&db).await;
    load::load_special_conditions(&db).await;
    load::load_special_conditions_free_form(&db).await;
}
