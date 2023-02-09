use crate::types::*;
use csv::StringRecord;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use regex::Regex;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use std::fs;
use std::fs::File;
use std::io::BufRead;

const INSERT_AMATEUR_SQL: &str = include_str!("sql/insert-amateur.sql");
const INSERT_COMMENT_SQL: &str = include_str!("sql/insert-comment.sql");
const INSERT_ENTITY_SQL: &str = include_str!("sql/insert-entity.sql");
const INSERT_HEADER_SQL: &str = include_str!("sql/insert-header.sql");
const INSERT_HISTORY_SQL: &str = include_str!("sql/insert-history.sql");
const INSERT_LICENSE_ATTACHMENT_SQL: &str = include_str!("sql/insert-license-attachment.sql");
const INSERT_SPECIAL_CONDITION_SQL: &str = include_str!("sql/insert-special-condition.sql");
const INSERT_SPECIAL_CONDITION_FREE_FORM_SQL: &str =
    include_str!("sql/insert-special-condition-free-form.sql");
const INSERT_SPECIAL_CONDITION_CODE_SQL: &str =
    include_str!("sql/insert-special-condition-code.sql");

const BIND_LIMIT: usize = 32766;

pub async fn load_amateurs(db: &SqlitePool, meili: &meilisearch_sdk::Client, clear_first: bool) {
    let amateurs_file = File::open("AM.dat");
    if amateurs_file.is_err() {
        println!("AM.dat not found, skipping");
        return;
    }
    let amateurs_file = amateurs_file.unwrap();
    //let amateurs_file_meta = fs::metadata("AM.dat").expect("Error getting file metadata");
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

    // if clear_first {
    //     QueryBuilder::new("DELETE FROM amateurs")
    //         .build()
    //         .execute(&mut transaction)
    //         .await
    //         .expect("Error deleting amateurs");
    // }

    let amatuers: Vec<Amateur> = reader.records().map(|record| {
        record.unwrap().deserialize::<Amateur>(None).unwrap()
    }).collect();

    // transaction
    //     .commit()
    //     .await
    //     .expect("Error committing transaction");

    let meili_task = meili
        .index("amateurs")
        .add_documents(&amatuers, Some("UniqueSystemIdentifier"))
        .await
        .unwrap();
    dbg!(meili_task);
    std::fs::remove_file("AM.dat").expect("Error deleting AM.dat");
    progress_bar.finish();
}

pub async fn load_comments(db: &SqlitePool, clear_first: bool) {
    let comments_file = File::open("CO.dat");
    if comments_file.is_err() {
        println!("CO.dat not found, skipping");
        return;
    }

    // Some idiot at the FCC decided that unescaped newlines in the middle of a field were cool
    // Uncle Ted may have had some good ideas after all
    let comments_regex = Regex::new(r"\s*\r\r\n").unwrap();
    let comments = fs::read_to_string("CO.dat").expect("Error reading file");
    fs::write(
        "CO.dat",
        comments_regex.replace_all(&comments, " ").to_string(),
    )
    .expect("Error writing file");

    let comments_file = File::open("CO.dat").unwrap();
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

    if clear_first {
        QueryBuilder::new("DELETE FROM comments")
            .build()
            .execute(&mut transaction)
            .await
            .expect("Error deleting comments");
    }

    let chunk_size = BIND_LIMIT / 8;
    for chunk in &reader.records().chunks(chunk_size) {
        let chunk = chunk.collect::<Result<Vec<StringRecord>, _>>().unwrap();
        let chunk = chunk.iter();

        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(INSERT_COMMENT_SQL);

        query_builder.push_values(chunk, |mut builder, entry| {
            let comment: Comment = entry.deserialize(None).expect("Error deserializing entry");
            builder
                .push_bind(comment.RecordType)
                .push_bind(comment.UniqueSystemIdentifier)
                .push_bind(comment.UlsFileNumber)
                .push_bind(comment.CallSign)
                .push_bind(comment.CommentDate)
                .push_bind(comment.Description)
                .push_bind(comment.StatusCode)
                .push_bind(comment.StatusDate);
        });

        query_builder
            .build()
            .execute(&mut transaction)
            .await
            .expect("Error executing query");
        progress_bar.set_position(progress_bar.position() + chunk_size as u64);
    }

    transaction
        .commit()
        .await
        .expect("Error committing transaction");
    std::fs::remove_file("CO.dat").expect("Error deleting CO.dat");
    progress_bar.finish();
}

pub async fn load_entities(db: &SqlitePool, clear_first: bool) {
    let entities_file = File::open("EN.dat");
    if entities_file.is_err() {
        println!("EN.dat not found, skipping");
        return;
    }
    let entities_file = entities_file.unwrap();
    //let entities_file_meta = fs::metadata("EN.dat").expect("Error getting file metadata");
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

    if clear_first {
        QueryBuilder::new("DELETE FROM entities")
            .build()
            .execute(&mut transaction)
            .await
            .expect("Error deleting entities");
    }

    let chunk_size = BIND_LIMIT / 30;
    for chunk in &reader.records().chunks(chunk_size) {
        let chunk = chunk.collect::<Result<Vec<StringRecord>, _>>().unwrap();
        let chunk = chunk.iter();

        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(INSERT_ENTITY_SQL);

        query_builder.push_values(chunk, |mut builder, entry| {
            let entity: Entity = entry.deserialize(None).expect("Error deserializing entry");
            builder
                .push_bind(entity.RecordType)
                .push_bind(entity.UniqueSystemIdentifier)
                .push_bind(entity.UlsFileNumber)
                .push_bind(entity.EBFNumber)
                .push_bind(entity.CallSign)
                .push_bind(entity.EntityType)
                .push_bind(entity.LicenseeId)
                .push_bind(entity.EntityName)
                .push_bind(entity.FirstName)
                .push_bind(entity.MiddleInitial)
                .push_bind(entity.LastName)
                .push_bind(entity.Suffix)
                .push_bind(entity.Phone)
                .push_bind(entity.Fax)
                .push_bind(entity.Email)
                .push_bind(entity.StreetAddress)
                .push_bind(entity.City)
                .push_bind(entity.State)
                .push_bind(entity.ZipCode)
                .push_bind(entity.POBox)
                .push_bind(entity.AttentionLine)
                .push_bind(entity.SGIN)
                .push_bind(entity.FRN)
                .push_bind(entity.ApplicantTypeCode)
                .push_bind(entity.ApplicantTypeCodeOther)
                .push_bind(entity.StatusCode)
                .push_bind(entity.StatusDate)
                .push_bind(entity.ThreePointSevenGhzLicenseType)
                .push_bind(entity.LinkedUniqueSystemIdentifier)
                .push_bind(entity.LinkedCallsign);
        });

        query_builder
            .build()
            .execute(&mut transaction)
            .await
            .expect("Error executing query");
        progress_bar.set_position(progress_bar.position() + chunk_size as u64);
    }

    transaction
        .commit()
        .await
        .expect("Error committing transaction");
    std::fs::remove_file("EN.dat").expect("Error deleting EN.dat");
    progress_bar.finish();
}

pub async fn load_headers(db: &SqlitePool, clear_first: bool) {
    let headers_file = File::open("HD.dat");
    if headers_file.is_err() {
        println!("HD.dat not found, skipping");
        return;
    }
    let headers_file = headers_file.unwrap();
    // let headers_file_meta = fs::metadata("HD.dat").expect("Error getting file metadata");
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

    if clear_first {
        QueryBuilder::new("DELETE FROM headers")
            .build()
            .execute(&mut transaction)
            .await
            .expect("Error deleting headers");
    }

    let chunk_size = BIND_LIMIT / 60;
    for chunk in &reader.records().chunks(chunk_size) {
        let chunk = chunk.collect::<Result<Vec<StringRecord>, _>>().unwrap();
        let chunk = chunk.iter();

        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(INSERT_HEADER_SQL);

        query_builder.push_values(chunk, |mut builder, entry| {
            let header: Header = entry.deserialize(None).expect("Error deserializing entry");
            builder
                .push_bind(header.RecordType)
                .push_bind(header.UniqueSystemIdentifier)
                .push_bind(header.UlsFileNumber)
                .push_bind(header.EBFNumber)
                .push_bind(header.CallSign)
                .push_bind(header.LicenseStatus)
                .push_bind(header.RadioServiceCode)
                .push_bind(header.GrantDate)
                .push_bind(header.ExpiredDate)
                .push_bind(header.CancellationDate)
                .push_bind(header.EligibilityRuleNumber)
                .push_bind(header.Reserved)
                .push_bind(header.Alien)
                .push_bind(header.AlienGovernment)
                .push_bind(header.AlienCorporation)
                .push_bind(header.AlienOfficers)
                .push_bind(header.AlienControl)
                .push_bind(header.Revoked)
                .push_bind(header.Convicted)
                .push_bind(header.Adjudged)
                .push_bind(header.Reserved2)
                .push_bind(header.CommonCarrier)
                .push_bind(header.NonCommonCarrier)
                .push_bind(header.PrivateComm)
                .push_bind(header.Fixed)
                .push_bind(header.Mobile)
                .push_bind(header.Radiolocation)
                .push_bind(header.Sattelite)
                .push_bind(header.DevelopmentalOrSta)
                .push_bind(header.InterconnectedService)
                .push_bind(header.CertifierFirstName)
                .push_bind(header.CertifierMiddleInitial)
                .push_bind(header.CertifierLastName)
                .push_bind(header.CertifierSuffix)
                .push_bind(header.CertifierTitle)
                .push_bind(header.Female)
                .push_bind(header.BlackOrAfricanAmerican)
                .push_bind(header.NativeAmerican)
                .push_bind(header.Hawaiian)
                .push_bind(header.Asian)
                .push_bind(header.White)
                .push_bind(header.Hispanic)
                .push_bind(header.EffectiveDate)
                .push_bind(header.LastActionDate)
                .push_bind(header.AuctionId)
                .push_bind(header.BroadcastServicesRegulatoryStatus)
                .push_bind(header.BandManagerRegulatoryStatus)
                .push_bind(header.BroadcastServicesTypeOfRadioService)
                .push_bind(header.AlienRuling)
                .push_bind(header.LicenseeNameChange)
                .push_bind(header.WhitespaceIndicator)
                .push_bind(header.OperationRequirementChoice)
                .push_bind(header.OperationRequirementAnswer)
                .push_bind(header.DiscontinuationOfService)
                .push_bind(header.RegulatoryCompliance)
                .push_bind(header.EligibilityCertification900Mhz)
                .push_bind(header.TransitionPlanCertification900Mhz)
                .push_bind(header.ReturnSpectrumCertification900Mhz)
                .push_bind(header.PaymentCertification900Mhz);
        });

        query_builder
            .build()
            .execute(&mut transaction)
            .await
            .expect("Error executing query");
        progress_bar.set_position(progress_bar.position() + chunk_size as u64);
    }

    transaction
        .commit()
        .await
        .expect("Error committing transaction");
    std::fs::remove_file("HD.dat").expect("Error deleting HD.dat");
    progress_bar.finish();
}

pub async fn load_history(db: &SqlitePool, clear_first: bool) {
    let history_file = File::open("HS.dat");
    if history_file.is_err() {
        println!("No HS.dat file found, skipping");
        return;
    }
    let history_file = history_file.unwrap();
    // let history_file_meta = fs::metadata("HS.dat").expect("Error getting file metadata");
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

    if clear_first {
        QueryBuilder::new("DELETE FROM history")
            .build()
            .execute(&mut transaction)
            .await
            .expect("Error deleting history");
    }

    let chunk_size = BIND_LIMIT / 6;
    for chunk in &reader.records().chunks(chunk_size) {
        let chunk = chunk.collect::<Result<Vec<StringRecord>, _>>().unwrap();
        let chunk = chunk.iter();

        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(INSERT_HISTORY_SQL);

        query_builder.push_values(chunk, |mut builder, entry| {
            let history: History = entry.deserialize(None).expect("Error deserializing entry");
            builder
                .push_bind(history.RecordType)
                .push_bind(history.UniqueSystemIdentifier)
                .push_bind(history.UlsFileNumber)
                .push_bind(history.CallSign)
                .push_bind(history.LogDate)
                .push_bind(history.Code);
        });

        query_builder
            .build()
            .execute(&mut transaction)
            .await
            .expect("Error executing query");
        progress_bar.set_position(progress_bar.position() + chunk_size as u64);
    }

    transaction
        .commit()
        .await
        .expect("Error committing transaction");
    std::fs::remove_file("HS.dat").expect("Error deleting HS.dat");
    progress_bar.finish();
}

pub async fn load_license_attachments(db: &SqlitePool, clear_first: bool) {
    let attachments_file = File::open("LA.dat");
    if attachments_file.is_err() {
        println!("No LA.dat file found, skipping");
        return;
    }
    let attachments_file = attachments_file.unwrap();
    // let attachments_file_meta = fs::metadata("LA.dat").expect("Error getting file metadata");
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

    if clear_first {
        QueryBuilder::new("DELETE FROM license_attachments")
            .build()
            .execute(&mut transaction)
            .await
            .expect("Error deleting license_attachments");
    }

    let chunk_size = BIND_LIMIT / 8;
    for chunk in &reader.records().chunks(chunk_size) {
        let chunk = chunk.collect::<Result<Vec<StringRecord>, _>>().unwrap();
        let chunk = chunk.iter();

        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new(INSERT_LICENSE_ATTACHMENT_SQL);

        query_builder.push_values(chunk, |mut builder, entry| {
            let attachment: LicenseAttachment =
                entry.deserialize(None).expect("Error deserializing entry");
            builder
                .push_bind(attachment.RecordType)
                .push_bind(attachment.UniqueSystemIdentifier)
                .push_bind(attachment.CallSign)
                .push_bind(attachment.AttachmentCode)
                .push_bind(attachment.AttachmentDescription)
                .push_bind(attachment.AttachmentDate)
                .push_bind(attachment.AttachmentFileName)
                .push_bind(attachment.ActionPerformed);
        });

        query_builder
            .build()
            .execute(&mut transaction)
            .await
            .expect("Error executing query");
        progress_bar.set_position(progress_bar.position() + chunk_size as u64);
    }

    transaction
        .commit()
        .await
        .expect("Error committing transaction");
    std::fs::remove_file("LA.dat").expect("Error deleting LA.dat");
    progress_bar.finish();
}

pub async fn load_special_conditions(db: &SqlitePool, clear_first: bool) {
    let conditions_file = File::open("SC.dat");
    if conditions_file.is_err() {
        println!("No SC.dat file found, skipping");
        return;
    }
    let conditions_file = conditions_file.unwrap();
    // let conditions_file_meta = fs::metadata("SC.dat").expect("Error getting file metadata");
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

    if clear_first {
        QueryBuilder::new("DELETE FROM special_conditions")
            .build()
            .execute(&mut transaction)
            .await
            .expect("Error deleting special_conditions");
    }

    let chunk_size = BIND_LIMIT / 9;
    for chunk in &reader.records().chunks(chunk_size) {
        let chunk = chunk.collect::<Result<Vec<StringRecord>, _>>().unwrap();
        let chunk = chunk.iter();

        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new(INSERT_SPECIAL_CONDITION_SQL);

        query_builder.push_values(chunk, |mut builder, entry| {
            let condition: SpecialCondition =
                entry.deserialize(None).expect("Error deserializing entry");
            builder
                .push_bind(condition.RecordType)
                .push_bind(condition.UniqueSystemIdentifier)
                .push_bind(condition.UlsFileNumber)
                .push_bind(condition.EBFNumber)
                .push_bind(condition.CallSign)
                .push_bind(condition.SpecialConditionType)
                .push_bind(condition.SpecialConditionCode)
                .push_bind(condition.StatusCode)
                .push_bind(condition.StatusDate);
        });

        query_builder
            .build()
            .execute(&mut transaction)
            .await
            .expect("Error executing query");
        progress_bar.set_position(progress_bar.position() + chunk_size as u64);
    }

    transaction
        .commit()
        .await
        .expect("Error committing transaction");
    std::fs::remove_file("SC.dat").expect("Error deleting SC.dat");
    progress_bar.finish();
}

pub async fn load_special_conditions_free_form(db: &SqlitePool, clear_first: bool) {
    let conditions_file = File::open("SF.dat");
    if conditions_file.is_err() {
        println!("No SF.dat file found, skipping");
        return;
    }
    let conditions_file = conditions_file.unwrap();
    // let conditions_file_meta = fs::metadata("SF.dat").expect("Error getting file metadata");
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

    if clear_first {
        QueryBuilder::new("DELETE FROM special_conditions_free_form")
            .build()
            .execute(&mut transaction)
            .await
            .expect("Error deleting special_conditions_free_form");
    }

    let chunk_size = BIND_LIMIT / 11;
    for chunk in &reader.records().chunks(chunk_size) {
        let chunk = chunk.collect::<Result<Vec<StringRecord>, _>>().unwrap();
        let chunk = chunk.iter();

        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new(INSERT_SPECIAL_CONDITION_FREE_FORM_SQL);

        query_builder.push_values(chunk, |mut builder, entry| {
            let condition: SpecialConditionFreeForm =
                entry.deserialize(None).expect("Error deserializing entry");
            builder
                .push_bind(condition.RecordType)
                .push_bind(condition.UniqueSystemIdentifier)
                .push_bind(condition.UlsFileNumber)
                .push_bind(condition.EBFNumber)
                .push_bind(condition.CallSign)
                .push_bind(condition.LicenseFreeFormType)
                .push_bind(condition.UniqueLicenseFreeFormIdentifier)
                .push_bind(condition.SequenceNumber)
                .push_bind(condition.LicenseFreeFormCondition)
                .push_bind(condition.StatusCode)
                .push_bind(condition.StatusDate);
        });

        query_builder
            .build()
            .execute(&mut transaction)
            .await
            .expect("Error executing query");
        progress_bar.set_position(progress_bar.position() + chunk_size as u64);
    }

    transaction
        .commit()
        .await
        .expect("Error committing transaction");
    std::fs::remove_file("SF.dat").expect("Error deleting SF.dat");
    progress_bar.finish();
}

pub async fn load_special_condition_codes(db: &SqlitePool, clear_first: bool) {
    let codes_file = File::open("special_condition_codes.txt");
    if codes_file.is_err() {
        println!("No special_condition_codes.txt file found, skipping");
        return;
    }
    let codes_file = codes_file.unwrap();
    // let history_file_meta = fs::metadata("special_condition_codes.txt").expect("Error getting file metadata");
    let line_count = std::io::BufReader::new(&codes_file).lines().count();
    drop(codes_file);

    let codes_file = File::open("special_condition_codes.txt").expect("Error opening file");
    let mut transaction = db.begin().await.expect("Error starting transaction");
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'|')
        .quoting(true)
        .from_reader(codes_file);

    let progress_bar = ProgressBar::new(line_count.try_into().unwrap());
    progress_bar.set_style(
        ProgressStyle::with_template(
            "[{elapsed}+{eta}/{duration}] [{bar:40.cyan/blue}] {human_pos}/{human_len} ({per_sec}) {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    progress_bar.set_message("special_condition_codes.txt");

    if clear_first {
        QueryBuilder::new("DELETE FROM special_condition_codes")
            .build()
            .execute(&mut transaction)
            .await
            .expect("Error deleting special_condition_codes");
    }

    let chunk_size = BIND_LIMIT / 4;
    for chunk in &reader.records().chunks(chunk_size) {
        let chunk = chunk.collect::<Result<Vec<StringRecord>, _>>().unwrap();
        let chunk = chunk.iter();

        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new(INSERT_SPECIAL_CONDITION_CODE_SQL);

        query_builder.push_values(chunk, |mut builder, entry| {
            builder
                .push_bind(entry.get(0))
                .push_bind(entry.get(1))
                .push_bind(format!(
                    "{} {} {} {} {}",
                    entry.get(2).unwrap_or_default(),
                    entry.get(3).unwrap_or_default(),
                    entry.get(4).unwrap_or_default(),
                    entry.get(5).unwrap_or_default(),
                    entry.get(6).unwrap_or_default()
                ))
                .push_bind(entry.get(7));
        });

        query_builder
            .build()
            .execute(&mut transaction)
            .await
            .expect("Error executing query");
        progress_bar.set_position(progress_bar.position() + chunk_size as u64);
    }

    transaction
        .commit()
        .await
        .expect("Error committing transaction");
    std::fs::remove_file("special_condition_codes.txt")
        .expect("Error deleting special_condition_codes.txt");
    progress_bar.finish();
}
