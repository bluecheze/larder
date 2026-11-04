use std::time::{SystemTime, UNIX_EPOCH};

/// Today's date as YYYY-MM-DD in UTC, for conflict copy names. Pulling in a
/// whole date library for one string didn't seem worth it.
pub fn today() -> String {
    let days = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() / 86_400).unwrap_or(0);
    civil(days as i64)
}

// Howard Hinnant's days-to-civil algorithm, the same one most date libs use.
fn civil(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests {
    use super::civil;

    #[test]
    fn known_dates() {
        assert_eq!(civil(0), "1970-01-01");
        assert_eq!(civil(20_396), "2025-11-04");
        assert_eq!(civil(11_016), "2000-02-29");
    }
}
