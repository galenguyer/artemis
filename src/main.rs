use chrono::DateTime;
use filetime::{self, FileTime};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::fs;
use std::io::{Read, Write};

const WEEKLY_DUMP_URL: &str = "https://data.fcc.gov/download/pub/uls/complete/l_amat.zip";

fn main() {
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
                return;
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

    progress_bar.finish();
}
