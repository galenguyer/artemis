use chrono::DateTime;
use filetime::{self, FileTime};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use regex::Regex;
use sqlx::sqlite::SqlitePool;
use sqlx::{Pool, Sqlite};
use std::fs::{self, File};
use std::io::BufRead;
use std::io::{Read, Write};
mod fcc_date;
mod types;
use crate::types::*;

const WEEKLY_DUMP_URL: &str = "https://data.fcc.gov/download/pub/uls/complete/l_amat.zip";
const INSERT_AMATEUR_SQL: &str = r"INSERT INTO amateurs (record_type, unique_system_identifier, uls_file_number, ebf_number, call_sign, operator_class, group_code, region_code, trustee_call_sign, trustee_indicator, physician_certification, ve_signature, systematic_call_sign_change, vanity_call_sign_change, vainty_relationship, previous_call_sign, previous_operator_class, trustee_name) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
const INSERT_COMMENT_SQL: &str = r"INSERT INTO comments (record_type, unique_system_identifier, uls_file_number, call_sign, comment_date, description, status_code, status_date) VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
const INSERT_ENTITY_SQL: &str = r"INSERT INTO entities (record_type, unique_system_identifier, uls_file_number, ebf_number, call_sign, entity_type, licensee_id, entity_name, first_name, mi, last_name, suffix, phone, fax, email, street_address, city, state, zip_code, po_box, attention_line, sgin, frn, applicant_type_code, applicant_type_other, status_code, status_date, lic_category_code, linked_license_id, linked_callsign) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
const INSERT_HEADER_SQL: &str = r"INSERT INTO headers (record_type, unique_system_identifier, uls_file_number, ebf_number, call_sign, license_status, radio_service_code, grant_date, expired_date, cancellation_date, eligibility_rule_number, reserved, alien, alien_government, alien_corporation, alien_officer, alien_control, revoked, convicted, adjudged, reserved2, common_carrier, non_common_carrier, private_comm, fixed, mobile, radiolocation, satellite, developmental_or_sta, interconnected_service, certifier_first_name, certifier_mi, certifier_last_name, certifier_suffix, certifier_title, gender, african_american, native_american, hawaiian, asian, white, ethnicity, effective_date, last_action_date, auction_id, reg_stat_broad_serv, band_manager, type_serv_broad_serv, alien_ruling, licensee_name_change, whitespace_ind, additional_cert_choice, additional_cert_answer, discontinuation_ind, regulatory_compliance_ind, eligibility_cert_900, transition_plan_cert_900, return_spectrum_cert_900, payment_cert_900) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
const INSERT_HISTORY_SQL: &str = r"INSERT INTO history (record_type, unique_system_identifier, uls_file_number, call_sign, log_date, code) VALUES (?, ?, ?, ?, ?, ?)";
const INSERT_LICENSE_ATTACHMENT_SQL: &str = r"INSERT INTO license_attachments (record_type, unique_system_identifier, call_sign, attachment_code, attachment_description, attachment_date, attachment_file_name, action_performed) VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
const INSERT_SPECIAL_CONDITION_SQL: &str = r"INSERT INTO special_conditions (record_type, unique_system_identifier, uls_file_number, ebf_number, call_sign, special_conditions_type, special_conditions_code, status_code, status_date) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)";
const INSERT_SPECIAL_CONDITION_FREE_FORM_SQL: &str = r"INSERT INTO special_conditions_free_form (record_type, unique_system_identifier, uls_file_number, ebf_number, call_sign, license_free_form_type, unique_license_free_form_identifier, sequence_number, license_free_form_condition, status_code, status_date) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

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

async fn load_amateurs(db: &Pool<Sqlite>) {
    let amateurs_file = File::open("AM.dat").expect("Error opening file");
    let amateurs_file_meta = fs::metadata("AM.dat").expect("Error getting file metadata");
    let line_count = std::io::BufReader::new(&amateurs_file).lines().count();
    drop(amateurs_file);

    let amateurs_file = File::open("AM.dat").expect("Error opening file");
    let mut transaction = db.begin().await.expect("Error starting transaction");
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'|')
        .quoting(false)
        .from_reader(amateurs_file);

    let progress_bar = ProgressBar::new(line_count.try_into().unwrap());
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed}+{eta}/{duration}] [{bar:40.cyan/blue}] {human_pos}/{human_len} ({per_sec}) {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    progress_bar.set_message("AM.dat");

    for line in reader.records() {
        let line = line.expect("Error reading entry");
        let amateur: Amateur = line.deserialize(None).expect("Error deserializing entry");
        let statement = sqlx::query(INSERT_AMATEUR_SQL);
        statement
            .bind(amateur.RecordType)
            .bind(amateur.UniqueSystemIdentifier)
            .bind(amateur.UlsFileNumber)
            .bind(amateur.EBFNumber)
            .bind(amateur.CallSign)
            .bind(amateur.OperatorClass)
            .bind(amateur.GroupCode)
            .bind(amateur.RegionCode)
            .bind(amateur.TrusteeCallSign)
            .bind(amateur.TrusteeIndicator)
            .bind(amateur.PhysicianCertification)
            .bind(amateur.VESignature)
            .bind(amateur.SystematicCallSignChange)
            .bind(amateur.VanityCallSignChange)
            .bind(amateur.VanityRelationship)
            .bind(amateur.PreviousCallSign)
            .bind(amateur.PreviousOperatorClass)
            .bind(amateur.TrusteeName)
            .execute(&mut transaction)
            .await
            .expect("Error executing statement");
        progress_bar.set_position(line.position().unwrap().line());
    }

    transaction
        .commit()
        .await
        .expect("Error committing transaction");
    progress_bar.finish();
}

async fn load_comments(db: &Pool<Sqlite>) {
    let comments_file = File::open("CO.dat").expect("Error opening file");
    // let comments_file_meta = fs::metadata("CO.dat").expect("Error getting file metadata");
    let line_count = std::io::BufReader::new(&comments_file).lines().count();
    drop(comments_file);

    let comments_file = File::open("CO.dat").expect("Error opening file");
    let mut transaction = db.begin().await.expect("Error starting transaction");
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'|')
        .quoting(false)
        .from_reader(comments_file);

    let progress_bar = ProgressBar::new(line_count.try_into().unwrap());
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed}+{eta}/{duration}] [{bar:40.cyan/blue}] {human_pos}/{human_len} ({per_sec}) {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    progress_bar.set_message("CO.dat");

    for line in reader.records() {
        let line = line.expect("Error reading entry");
        let comment: Comment = line.deserialize(None).expect("Error deserializing entry");
        let statement = sqlx::query(INSERT_COMMENT_SQL);
        statement
            .bind(comment.RecordType)
            .bind(comment.UniqueSystemIdentifier)
            .bind(comment.UlsFileNumber)
            .bind(comment.CallSign)
            .bind(comment.CommentDate)
            .bind(comment.Description)
            .bind(comment.StatusCode)
            .bind(comment.StatusDate)
            .execute(&mut transaction)
            .await
            .expect("Error executing statement");
        progress_bar.set_position(line.position().unwrap().line());
    }

    transaction
        .commit()
        .await
        .expect("Error committing transaction");

    progress_bar.finish();
}

async fn load_entities(db: &Pool<Sqlite>) {
    let entities_file = File::open("EN.dat").expect("Error opening file");
    let entities_file_meta = fs::metadata("EN.dat").expect("Error getting file metadata");
    let line_count = std::io::BufReader::new(&entities_file).lines().count();
    drop(entities_file);

    let entities_file = File::open("EN.dat").expect("Error opening file");
    let mut transaction = db.begin().await.expect("Error starting transaction");
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'|')
        .quoting(false)
        .from_reader(entities_file);

    let progress_bar = ProgressBar::new(line_count.try_into().unwrap());
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed}+{eta}/{duration}] [{bar:40.cyan/blue}] {human_pos}/{human_len} ({per_sec}) {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    progress_bar.set_message("EN.dat");

    for line in reader.records() {
        let line = line.expect("Error reading entry");
        let entity: Entity = line.deserialize(None).expect("Error deserializing entry");
        let statement = sqlx::query(INSERT_ENTITY_SQL);
        statement
            .bind(entity.RecordType)
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
        progress_bar.set_position(line.position().unwrap().line());
    }

    transaction
        .commit()
        .await
        .expect("Error committing transaction");
    progress_bar.finish();
}

async fn load_headers(db: &Pool<Sqlite>) {
    let headers_file = File::open("HD.dat").expect("Error opening file");
    // let comments_file_meta = fs::metadata("CO.dat").expect("Error getting file metadata");
    let line_count = std::io::BufReader::new(&headers_file).lines().count();
    drop(headers_file);

    let headers_file = File::open("HD.dat").expect("Error opening file");
    let mut transaction = db.begin().await.expect("Error starting transaction");
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'|')
        .quoting(false)
        .from_reader(headers_file);

    let progress_bar = ProgressBar::new(line_count.try_into().unwrap());
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed}+{eta}/{duration}] [{bar:40.cyan/blue}] {human_pos}/{human_len} ({per_sec}) {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    progress_bar.set_message("HD.dat");

    for line in reader.records() {
        let line = line.expect("Error reading entry");
        let header: Header = line.deserialize(None).expect("Error deserializing entry");
        let statement = sqlx::query(INSERT_HEADER_SQL);
        statement
            .bind(header.RecordType)
            .bind(header.UniqueSystemIdentifier)
            .bind(header.UlsFileNumber)
            .bind(header.EBFNumber)
            .bind(header.CallSign)
            .bind(header.LicenseStatus)
            .bind(header.RadioServiceCode)
            .bind(header.GrantDate)
            .bind(header.ExpiredDate)
            .bind(header.CancellationDate)
            .bind(header.EligibilityRuleNumber)
            .bind(header.Reserved)
            .bind(header.Alien)
            .bind(header.AlienGovernment)
            .bind(header.AlienCorporation)
            .bind(header.AlienOfficers)
            .bind(header.AlienControl)
            .bind(header.Revoked)
            .bind(header.Convicted)
            .bind(header.Adjudged)
            .bind(header.Reserved2)
            .bind(header.CommonCarrier)
            .bind(header.NonCommonCarrier)
            .bind(header.PrivateComm)
            .bind(header.Fixed)
            .bind(header.Mobile)
            .bind(header.Radiolocation)
            .bind(header.Sattelite)
            .bind(header.DevelopmentalOrSta)
            .bind(header.InterconnectedService)
            .bind(header.CertifierFirstName)
            .bind(header.CertifierMiddleInitial)
            .bind(header.CertifierLastName)
            .bind(header.CertifierSuffix)
            .bind(header.CertifierTitle)
            .bind(header.Female)
            .bind(header.BlackOrAfricanAmerican)
            .bind(header.NativeAmerican)
            .bind(header.Hawaiian)
            .bind(header.Asian)
            .bind(header.White)
            .bind(header.Hispanic)
            .bind(header.EffectiveDate)
            .bind(header.LastActionDate)
            .bind(header.AuctionId)
            .bind(header.BroadcastServicesRegulatoryStatus)
            .bind(header.BandManagerRegulatoryStatus)
            .bind(header.BroadcastServicesTypeOfRadioService)
            .bind(header.AlienRuling)
            .bind(header.LicenseeNameChange)
            .bind(header.WhitespaceIndicator)
            .bind(header.OperationRequirementChoice)
            .bind(header.OperationRequirementAnswer)
            .bind(header.DiscontinuationOfService)
            .bind(header.RegulatoryCompliance)
            .bind(header.EligibilityCertification900Mhz)
            .bind(header.TransitionPlanCertification900Mhz)
            .bind(header.ReturnSpectrumCertification900Mhz)
            .bind(header.PaymentCertification900Mhz)
            .execute(&mut transaction)
            .await
            .expect("Error executing statement");
        progress_bar.set_position(line.position().unwrap().line());
    }

    transaction
        .commit()
        .await
        .expect("Error committing transaction");

    progress_bar.finish();
}

async fn load_history(db: &Pool<Sqlite>) {
    let history_file = File::open("HS.dat").expect("Error opening file");
    // let comments_file_meta = fs::metadata("CO.dat").expect("Error getting file metadata");
    let line_count = std::io::BufReader::new(&history_file).lines().count();
    drop(history_file);

    let history_file = File::open("HS.dat").expect("Error opening file");
    let mut transaction = db.begin().await.expect("Error starting transaction");
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'|')
        .quoting(false)
        .from_reader(history_file);

    let progress_bar = ProgressBar::new(line_count.try_into().unwrap());
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed}+{eta}/{duration}] [{bar:40.cyan/blue}] {human_pos}/{human_len} ({per_sec}) {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    progress_bar.set_message("HS.dat");

    for line in reader.records() {
        let line = line.expect("Error reading entry");
        let history: History = line.deserialize(None).expect("Error deserializing entry");
        let statement = sqlx::query(INSERT_HISTORY_SQL);
        statement
            .bind(history.RecordType)
            .bind(history.UniqueSystemIdentifier)
            .bind(history.UlsFileNumber)
            .bind(history.CallSign)
            .bind(history.LogDate)
            .bind(history.Code)
            .execute(&mut transaction)
            .await
            .expect("Error executing statement");
        progress_bar.set_position(line.position().unwrap().line());
    }

    transaction
        .commit()
        .await
        .expect("Error committing transaction");

    progress_bar.finish();
}

async fn load_license_attachments(db: &Pool<Sqlite>) {
    let attachments_file = File::open("LA.dat").expect("Error opening file");
    // let comments_file_meta = fs::metadata("CO.dat").expect("Error getting file metadata");
    let line_count = std::io::BufReader::new(&attachments_file).lines().count();
    drop(attachments_file);

    let attachments_file = File::open("LA.dat").expect("Error opening file");
    let mut transaction = db.begin().await.expect("Error starting transaction");
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'|')
        .quoting(false)
        .from_reader(attachments_file);

    let progress_bar = ProgressBar::new(line_count.try_into().unwrap());
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed}+{eta}/{duration}] [{bar:40.cyan/blue}] {human_pos}/{human_len} ({per_sec}) {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    progress_bar.set_message("LA.dat");

    for line in reader.records() {
        let line = line.expect("Error reading entry");
        let attachment: LicenseAttachment =
            line.deserialize(None).expect("Error deserializing entry");
        let statement = sqlx::query(INSERT_LICENSE_ATTACHMENT_SQL);
        statement
            .bind(attachment.RecordType)
            .bind(attachment.UniqueSystemIdentifier)
            .bind(attachment.CallSign)
            .bind(attachment.AttachmentCode)
            .bind(attachment.AttachmentDescription)
            .bind(attachment.AttachmentDate)
            .bind(attachment.AttachmentFileName)
            .bind(attachment.ActionPerformed)
            .execute(&mut transaction)
            .await
            .expect("Error executing statement");
        progress_bar.set_position(line.position().unwrap().line());
    }

    transaction
        .commit()
        .await
        .expect("Error committing transaction");

    progress_bar.finish();
}

async fn load_special_conditions(db: &Pool<Sqlite>) {
    let conditions_file = File::open("SC.dat").expect("Error opening file");
    // let comments_file_meta = fs::metadata("CO.dat").expect("Error getting file metadata");
    let line_count = std::io::BufReader::new(&conditions_file).lines().count();
    drop(conditions_file);

    let conditions_file = File::open("SC.dat").expect("Error opening file");
    let mut transaction = db.begin().await.expect("Error starting transaction");
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'|')
        .quoting(false)
        .from_reader(conditions_file);

    let progress_bar = ProgressBar::new(line_count.try_into().unwrap());
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed}+{eta}/{duration}] [{bar:40.cyan/blue}] {human_pos}/{human_len} ({per_sec}) {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    progress_bar.set_message("SC.dat");

    for line in reader.records() {
        let line = line.expect("Error reading entry");
        let condition: SpecialCondition =
            line.deserialize(None).expect("Error deserializing entry");
        let statement = sqlx::query(INSERT_SPECIAL_CONDITION_SQL);
        statement
            .bind(condition.RecordType)
            .bind(condition.UniqueSystemIdentifier)
            .bind(condition.UlsFileNumber)
            .bind(condition.EBFNumber)
            .bind(condition.CallSign)
            .bind(condition.SpecialConditionType)
            .bind(condition.SpecialConditionCode)
            .bind(condition.StatusCode)
            .bind(condition.StatusDate)
            .execute(&mut transaction)
            .await
            .expect("Error executing statement");
        progress_bar.set_position(line.position().unwrap().line());
    }

    transaction
        .commit()
        .await
        .expect("Error committing transaction");

    progress_bar.finish();
}

async fn load_special_conditions_free_form(db: &Pool<Sqlite>) {
    let conditions_file = File::open("SF.dat").expect("Error opening file");
    // let comments_file_meta = fs::metadata("CO.dat").expect("Error getting file metadata");
    let line_count = std::io::BufReader::new(&conditions_file).lines().count();
    drop(conditions_file);

    let conditions_file = File::open("SF.dat").expect("Error opening file");
    let mut transaction = db.begin().await.expect("Error starting transaction");
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'|')
        .quoting(false)
        .from_reader(conditions_file);

    let progress_bar = ProgressBar::new(line_count.try_into().unwrap());
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed}+{eta}/{duration}] [{bar:40.cyan/blue}] {human_pos}/{human_len} ({per_sec}) {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    progress_bar.set_message("SF.dat");

    for line in reader.records() {
        let line = line.expect("Error reading entry");
        let condition: SpecialConditionFreeForm =
            line.deserialize(None).expect("Error deserializing entry");
        let statement = sqlx::query(INSERT_SPECIAL_CONDITION_FREE_FORM_SQL);
        statement
            .bind(condition.RecordType)
            .bind(condition.UniqueSystemIdentifier)
            .bind(condition.UlsFileNumber)
            .bind(condition.EBFNumber)
            .bind(condition.CallSign)
            .bind(condition.LicenseFreeFormType)
            .bind(condition.UniqueLicenseFreeFormIdentifier)
            .bind(condition.SequenceNumber)
            .bind(condition.LicenseFreeFormCondition)
            .bind(condition.StatusCode)
            .bind(condition.StatusDate)
            .execute(&mut transaction)
            .await
            .expect("Error executing statement");
        progress_bar.set_position(line.position().unwrap().line());
    }

    transaction
        .commit()
        .await
        .expect("Error committing transaction");

    progress_bar.finish();
}

#[tokio::main]
async fn main() {
    // let output_file = download_file().expect("Error downloading file");
    let output_file = File::open("l_amat.zip").expect("Error opening file");

    unzip_file(output_file).expect("Error unzipping file");

    let db = SqlitePool::connect("sqlite://fcc.db")
        .await
        .expect("Error connecting to database");

    // Some idiot at the FCC decided that unescaped newlines in the middle of a field were cool
    // Uncle Ted may have had some good ideas after all
    let re = Regex::new(r"\s*\r\r\n").unwrap();
    let comments = fs::read_to_string("CO.dat").expect("Error reading file");
    fs::write("CO.dat", re.replace_all(&comments, " ").to_string()).expect("Error writing file");

    load_amateurs(&db).await;
    load_comments(&db).await;
    load_entities(&db).await;
    load_headers(&db).await;
    load_history(&db).await;
    load_license_attachments(&db).await;
    load_special_conditions(&db).await;
    load_special_conditions_free_form(&db).await;
}
