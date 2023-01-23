DROP INDEX IF EXISTS idx_amateurs_unique_system_identifier;
DROP INDEX IF EXISTS idx_amateurs_call_sign;
DROP INDEX IF EXISTS idx_amateurs_operator_class;

DROP INDEX IF EXISTS idx_comments_unique_system_identifier;
DROP INDEX IF EXISTS idx_comments_call_sign;

DROP INDEX IF EXISTS idx_entities_unique_system_identifier;
DROP INDEX IF EXISTS idx_entities_call_sign;
DROP INDEX IF EXISTS idx_entities_entity_name;
DROP INDEX IF EXISTS idx_entities_first_name;
DROP INDEX IF EXISTS idx_entities_last_name;
DROP INDEX IF EXISTS idx_entities_phone;
DROP INDEX IF EXISTS idx_entities_email;
DROP INDEX IF EXISTS idx_entities_street_address;
DROP INDEX IF EXISTS idx_entities_city;
DROP INDEX IF EXISTS idx_entities_state;
DROP INDEX IF EXISTS idx_entities_zip_code;
DROP INDEX IF EXISTS idx_entities_frn;

DROP INDEX IF EXISTS idx_headers_unique_system_identifier;
DROP INDEX IF EXISTS idx_headers_call_sign;
DROP INDEX IF EXISTS idx_headers_license_status;
DROP INDEX IF EXISTS idx_headers_grant_date;
DROP INDEX IF EXISTS idx_headers_expired_date;
DROP INDEX IF EXISTS idx_headers_certifier_first_name;
DROP INDEX IF EXISTS idx_headers_certifier_last_name;

DROP INDEX IF EXISTS idx_history_unique_system_identifier;
DROP INDEX IF EXISTS idx_history_call_sign;