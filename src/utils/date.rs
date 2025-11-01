use chrono::{DateTime, NaiveDate};

pub fn format_date(date_str: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        dt.date().naive_local().to_string() // "YYYY-MM-DD"
    } else if let Ok(naive) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        naive.to_string()
    } else {
        date_str.to_string() // fallback if parsing fails
    }
}
