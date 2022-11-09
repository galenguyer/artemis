.echo on

CREATE INDEX idx_amateurs_unique_system_identifier ON amateurs (unique_system_identifier);
CREATE INDEX idx_amateurs_call_sign ON amateurs (call_sign);
CREATE INDEX idx_amateurs_operator_class ON amateurs (operator_class);

CREATE INDEX idx_comments_unique_system_identifier ON comments (unique_system_identifier);
CREATE INDEX idx_comments_call_sign ON comments (call_sign);

CREATE INDEX idx_entities_unique_system_identifier ON entities (unique_system_identifier);
CREATE INDEX idx_entities_call_sign ON entities (call_sign);
CREATE INDEX idx_entities_entity_name ON entities (entity_name);
CREATE INDEX idx_entities_first_name ON entities (first_name);
CREATE INDEX idx_entities_last_name ON entities (last_name);
CREATE INDEX idx_entities_phone ON entities (phone);
CREATE INDEX idx_entities_email ON entities (email);
CREATE INDEX idx_entities_street_address ON entities (street_address);
CREATE INDEX idx_entities_city ON entities (city);
CREATE INDEX idx_entities_state ON entities (state);
CREATE INDEX idx_entities_zip_code ON entities (zip_code);
CREATE INDEX idx_entities_frn ON entities (frn);

CREATE INDEX idx_headers_unique_system_identifier ON headers (unique_system_identifier);
CREATE INDEX idx_headers_call_sign ON headers (call_sign);
CREATE INDEX idx_headers_license_status ON headers (license_status);
CREATE INDEX idx_headers_grant_date ON headers (grant_date);
CREATE INDEX idx_headers_expired_date ON headers (expired_date);
CREATE INDEX idx_headers_certifier_first_name ON headers (certifier_first_name);
CREATE INDEX idx_headers_certifier_last_name ON headers (certifier_last_name);


CREATE INDEX idx_history_unique_system_identifier ON history (unique_system_identifier);
CREATE INDEX idx_history_call_sign ON history (call_sign);
