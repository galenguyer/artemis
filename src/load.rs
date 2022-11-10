use crate::types::*;
use csv::StringRecord;
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use std::fs::File;
use std::io::BufRead;

const INSERT_AMATEUR_SQL: &str = r"INSERT INTO amateurs (record_type, unique_system_identifier, uls_file_number, ebf_number, call_sign, operator_class, group_code, region_code, trustee_call_sign, trustee_indicator, physician_certification, ve_signature, systematic_call_sign_change, vanity_call_sign_change, vainty_relationship, previous_call_sign, previous_operator_class, trustee_name) ";
const INSERT_COMMENT_SQL: &str = r"INSERT INTO comments (record_type, unique_system_identifier, uls_file_number, call_sign, comment_date, description, status_code, status_date) ";
const INSERT_ENTITY_SQL: &str = r"INSERT INTO entities (record_type, unique_system_identifier, uls_file_number, ebf_number, call_sign, entity_type, licensee_id, entity_name, first_name, mi, last_name, suffix, phone, fax, email, street_address, city, state, zip_code, po_box, attention_line, sgin, frn, applicant_type_code, applicant_type_other, status_code, status_date, lic_category_code, linked_license_id, linked_callsign) ";
const INSERT_HEADER_SQL: &str = r"INSERT INTO headers (record_type, unique_system_identifier, uls_file_number, ebf_number, call_sign, license_status, radio_service_code, grant_date, expired_date, cancellation_date, eligibility_rule_number, reserved, alien, alien_government, alien_corporation, alien_officer, alien_control, revoked, convicted, adjudged, reserved2, common_carrier, non_common_carrier, private_comm, fixed, mobile, radiolocation, satellite, developmental_or_sta, interconnected_service, certifier_first_name, certifier_mi, certifier_last_name, certifier_suffix, certifier_title, gender, african_american, native_american, hawaiian, asian, white, ethnicity, effective_date, last_action_date, auction_id, reg_stat_broad_serv, band_manager, type_serv_broad_serv, alien_ruling, licensee_name_change, whitespace_ind, additional_cert_choice, additional_cert_answer, discontinuation_ind, regulatory_compliance_ind, eligibility_cert_900, transition_plan_cert_900, return_spectrum_cert_900, payment_cert_900) ";
const INSERT_HISTORY_SQL: &str = r"INSERT INTO history (record_type, unique_system_identifier, uls_file_number, call_sign, log_date, code) ";
const INSERT_LICENSE_ATTACHMENT_SQL: &str = r"INSERT INTO license_attachments (record_type, unique_system_identifier, call_sign, attachment_code, attachment_description, attachment_date, attachment_file_name, action_performed) ";
const INSERT_SPECIAL_CONDITION_SQL: &str = r"INSERT INTO special_conditions (record_type, unique_system_identifier, uls_file_number, ebf_number, call_sign, special_conditions_type, special_conditions_code, status_code, status_date) ";
const INSERT_SPECIAL_CONDITION_FREE_FORM_SQL: &str = r"INSERT INTO special_conditions_free_form (record_type, unique_system_identifier, uls_file_number, ebf_number, call_sign, license_free_form_type, unique_license_free_form_identifier, sequence_number, license_free_form_condition, status_code, status_date) ";
const INSERT_SPECIAL_CONDITION_CODES_SQL: &str =
    r"INSERT INTO special_condition_codes (code, service, description, unknown) ";

const BIND_LIMIT: usize = 32766;

pub async fn load_amateurs(db: &SqlitePool) {
    let amateurs_file = File::open("AM.dat").expect("Error opening file");
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

    let chunk_size = BIND_LIMIT / 18;
    for chunk in &reader.records().chunks(chunk_size) {
        let chunk = chunk.collect::<Result<Vec<StringRecord>, _>>().unwrap();
        let chunk = chunk.iter();

        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(INSERT_AMATEUR_SQL);

        query_builder.push_values(chunk, |mut builder, entry| {
            let amateur: Amateur = entry.deserialize(None).expect("Error deserializing entry");
            builder
                .push_bind(amateur.RecordType)
                .push_bind(amateur.UniqueSystemIdentifier)
                .push_bind(amateur.UlsFileNumber)
                .push_bind(amateur.EBFNumber)
                .push_bind(amateur.CallSign)
                .push_bind(amateur.OperatorClass)
                .push_bind(amateur.GroupCode)
                .push_bind(amateur.RegionCode)
                .push_bind(amateur.TrusteeCallSign)
                .push_bind(amateur.TrusteeIndicator)
                .push_bind(amateur.PhysicianCertification)
                .push_bind(amateur.VESignature)
                .push_bind(amateur.SystematicCallSignChange)
                .push_bind(amateur.VanityCallSignChange)
                .push_bind(amateur.VanityRelationship)
                .push_bind(amateur.PreviousCallSign)
                .push_bind(amateur.PreviousOperatorClass)
                .push_bind(amateur.TrusteeName);
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
    progress_bar.finish();
}

pub async fn load_comments(db: &SqlitePool) {
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

    progress_bar.finish();
}

pub async fn load_entities(db: &SqlitePool) {
    let entities_file = File::open("EN.dat").expect("Error opening file");
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
    progress_bar.finish();
}

pub async fn load_headers(db: &SqlitePool) {
    let headers_file = File::open("HD.dat").expect("Error opening file");
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

    progress_bar.finish();
}

pub async fn load_history(db: &SqlitePool) {
    let history_file = File::open("HS.dat").expect("Error opening file");
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

    progress_bar.finish();
}

pub async fn load_license_attachments(db: &SqlitePool) {
    let attachments_file = File::open("LA.dat").expect("Error opening file");
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

    progress_bar.finish();
}

pub async fn load_special_conditions(db: &SqlitePool) {
    let conditions_file = File::open("SC.dat").expect("Error opening file");
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

    progress_bar.finish();
}

pub async fn load_special_conditions_free_form(db: &SqlitePool) {
    let conditions_file = File::open("SF.dat").expect("Error opening file");
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

    progress_bar.finish();
}

pub async fn load_special_condition_codes(db: &SqlitePool) {
    let codes_file = File::open("special_condition_codes.txt").expect("Error opening file");
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

    let chunk_size = BIND_LIMIT / 4;
    for chunk in &reader.records().chunks(chunk_size) {
        let chunk = chunk.collect::<Result<Vec<StringRecord>, _>>().unwrap();
        let chunk = chunk.iter();

        let mut query_builder: QueryBuilder<Sqlite> =
            QueryBuilder::new(INSERT_SPECIAL_CONDITION_CODES_SQL);

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

    progress_bar.finish();
}
