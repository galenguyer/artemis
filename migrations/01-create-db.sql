CREATE TABLE migrations (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE entities (
  record_type varchar(2) not null,
  unique_system_identifier integer not null,
  uls_file_number varchar(14),
  ebf_number varchar(30),
  call_sign varchar(10),
  entity_type varchar(2),
  licensee_id varchar(9),
  entity_name varchar(200),
  first_name varchar(20),
  mi varchar(1),
  last_name varchar(20),
  suffix varchar(3),
  phone varchar(10),
  fax varchar(10),
  email varchar(50),
  street_address varchar(60),
  city varchar(20),
  state varchar(2),
  zip_code varchar(9),
  po_box varchar(20),
  attention_line varchar(35),
  sgin varchar(3),
  frn varchar(10),
  applicant_type_code varchar(1),
  applicant_type_other varchar(40),
  status_code varchar(1),
  status_date datetime null,
  lic_category_code varchar(1),
  linked_license_id integer null,
  linked_callsign varchar(10)
);
INSERT INTO migrations (name)
VALUES ('01-create-db.sql');