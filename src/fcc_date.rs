use chrono::NaiveDate;
use serde::{self, Deserialize, Deserializer, Serializer};

const FCC_FORMAT: &str = "%m/%d/%Y";
const SQL_FORMAT: &str = "%Y-%m-%d";

// The signature of a serialize_with function must follow the pattern:
//
//    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
//    where
//        S: Serializer
//
// although it may also be generic over the input types T.
pub fn serialize<S>(date: &Option<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = match date {
        Some(date) => date.format(SQL_FORMAT).to_string(),
        None => "".to_string(),
    };
    serializer.serialize_str(&s)
}

// The signature of a deserialize_with function must follow the pattern:
//
//    fn deserialize<'de, D>(D) -> Result<T, D::Error>
//    where
//        D: Deserializer<'de>
//
// although it may also be generic over the output types T.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s == "" {
        return Ok(None);
    }
    NaiveDate::parse_from_str(&s, FCC_FORMAT)
        .map(|date| Some(date))
        .map_err(serde::de::Error::custom)
}
