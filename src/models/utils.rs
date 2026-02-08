use serde::Serializer;
use time::OffsetDateTime;

// Custom serializer for OffsetDateTime
pub fn serialize_datetime<S>(dt: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let formatted = dt.format(&time::format_description::parse("[day]-[month]-[year] [hour]:[minute]:[second]")
        .unwrap())
        .unwrap_or_else(|_| "Invalid Date".to_string());
    serializer.serialize_str(&formatted)
}

// Custom serializer for Option<OffsetDateTime>
pub fn serialize_datetime_option<S>(dt: &Option<OffsetDateTime>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match dt {
        Some(date_time) => {
            let formatted = date_time.format(&time::format_description::parse("[day]-[month]-[year] [hour]:[minute]:[second]")
                .unwrap())
                .unwrap_or_else(|_| "Invalid Date".to_string());
            serializer.serialize_str(&formatted)
        }
        None => serializer.serialize_none(),
    }
}

// Password verification utility
pub async fn verify_password_hash(password_hash: &str, password: &str) -> bool {
    bcrypt::verify(password, password_hash).unwrap_or(false)
}
