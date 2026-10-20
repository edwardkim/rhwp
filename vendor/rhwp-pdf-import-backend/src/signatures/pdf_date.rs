//! PDF date-string parsing (ISO 32000-1 §7.9.4 "Dates").
//!
//! PDF dates look like `D:YYYYMMDDHHmmSSOHH'mm'` where `O` is the UTC
//! offset sign (`+`, `-`, or `Z` for UTC). Trailing components after
//! the year are optional; missing ones default to zero/UTC.
//!
//! This lives alongside the signature code because the /M entry on a
//! signature dictionary is a PDF date and every binding's `Signature`
//! surface wants a numeric timestamp rather than the raw string.

/// Parse a PDF date string into a Unix timestamp (seconds since epoch).
///
/// Returns `None` if the string doesn't match the PDF date grammar.
/// Leading `D:` prefix is optional (as seen in some producers). Time
/// components are optional and default to zero. Timezone defaults to
/// UTC when absent.
pub fn parse_pdf_date_to_epoch(s: &str) -> Option<i64> {
    let raw = s.strip_prefix("D:").unwrap_or(s);
    let bytes = raw.as_bytes();
    if bytes.len() < 4 {
        return None;
    }

    let year = parse_digits(bytes, 0, 4)?;
    let month = parse_digits(bytes, 4, 2).unwrap_or(1);
    let day = parse_digits(bytes, 6, 2).unwrap_or(1);
    let hour = parse_digits(bytes, 8, 2).unwrap_or(0);
    let minute = parse_digits(bytes, 10, 2).unwrap_or(0);
    let second = parse_digits(bytes, 12, 2).unwrap_or(0);

    // Offset handling starts after the seconds field.
    let tz_offset = parse_offset(bytes, 14)?;

    let days = days_from_civil(year, month, day)?;
    let seconds_utc =
        days * 86_400 + i64::from(hour) * 3600 + i64::from(minute) * 60 + i64::from(second);

    // PDF offsets are local - UTC; subtract to convert the local wall
    // time into UTC epoch seconds.
    Some(seconds_utc - tz_offset)
}

fn parse_digits(bytes: &[u8], start: usize, len: usize) -> Option<u32> {
    if start + len > bytes.len() {
        return None;
    }
    let mut value: u32 = 0;
    for b in &bytes[start..start + len] {
        if !b.is_ascii_digit() {
            return None;
        }
        value = value * 10 + u32::from(*b - b'0');
    }
    Some(value)
}

fn parse_offset(bytes: &[u8], start: usize) -> Option<i64> {
    if start >= bytes.len() {
        return Some(0);
    }
    match bytes[start] {
        b'Z' => Some(0),
        sign @ (b'+' | b'-') => {
            let hours = parse_digits(bytes, start + 1, 2)?;
            // Minutes can be `mm'` or `mm'mm'` variants; we skip the
            // apostrophe and read two more digits if present.
            let minutes_start = start + 3;
            let minutes_start = if bytes.get(minutes_start) == Some(&b'\'') {
                minutes_start + 1
            } else {
                minutes_start
            };
            let minutes = parse_digits(bytes, minutes_start, 2).unwrap_or(0);
            let total = i64::from(hours) * 3600 + i64::from(minutes) * 60;
            Some(if sign == b'-' { -total } else { total })
        },
        _ => Some(0),
    }
}

/// Days from 1970-01-01 to the given civil date, following the
/// "date algorithms" by Howard Hinnant (public domain).
fn days_from_civil(year: u32, month: u32, day: u32) -> Option<i64> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let y = if month <= 2 {
        i64::from(year) - 1
    } else {
        i64::from(year)
    };
    let era = y.div_euclid(400);
    let yoe = (y - era * 400) as u64;
    let m = i64::from(month);
    let doy = ((153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(day) - 1) as u64;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe as i64 - 719_468)
}

/// Civil date `(year, month, day)` from days since 1970-01-01 — the
/// exact inverse of [`days_from_civil`] (Howard Hinnant, public
/// domain). Leap-year-correct for all dates.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
                                                   // Parens make the cast apply to the whole `if` (both arms already
                                                   // share a type, so this is a readability fix, not a behaviour one).
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32; // [1, 12]
    (y + i64::from(m <= 2), m, d)
}

/// Format a Unix timestamp (seconds since 1970-01-01 UTC) as a
/// PDF date string `D:YYYYMMDDHHmmSSZ` (ISO 32000-1 §7.9.4). Pure
/// and leap-year-correct — replaces the prior buggy code that
/// hard-coded month/day to `0101` and approximated the year as
/// `1970 + days/365`.
pub(crate) fn pdf_date_from_unix_secs(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let tod = secs % 86_400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "D:{:04}{:02}{:02}{:02}{:02}{:02}Z",
        y,
        m,
        d,
        tod / 3600,
        (tod % 3600) / 60,
        tod % 60
    )
}

/// Current UTC time as a PDF date string. The single correct source
/// for signature `/M` and document-timestamp dates (DRY — replaces
/// two divergent buggy copies in `sign_bytes.rs` / `signer.rs`).
pub(crate) fn format_pdf_date_utc() -> String {
    use std::time::SystemTime;
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    pdf_date_from_unix_secs(secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    // 2026-04-21 12:00:00 UTC → 1_776_772_800 s since epoch.
    const APR21_NOON_UTC: i64 = 1_776_772_800;
    // 2026-04-21 00:00:00 UTC → 1_776_729_600 s since epoch.
    const APR21_MIDNIGHT_UTC: i64 = 1_776_729_600;

    #[test]
    fn utc_z_suffix() {
        assert_eq!(parse_pdf_date_to_epoch("D:20260421120000Z"), Some(APR21_NOON_UTC));
    }

    #[test]
    fn no_d_prefix_still_parses() {
        assert_eq!(parse_pdf_date_to_epoch("20260421120000Z"), Some(APR21_NOON_UTC));
    }

    #[test]
    fn positive_offset_converts_to_utc() {
        // 14:00 +02 = 12:00 UTC.
        assert_eq!(parse_pdf_date_to_epoch("D:20260421140000+02'00'"), Some(APR21_NOON_UTC));
    }

    #[test]
    fn negative_offset_converts_to_utc() {
        // 08:00 -04 = 12:00 UTC.
        assert_eq!(parse_pdf_date_to_epoch("D:20260421080000-04'00'"), Some(APR21_NOON_UTC));
    }

    #[test]
    fn partial_date_defaults_zero_midnight_utc() {
        assert_eq!(parse_pdf_date_to_epoch("D:20260421"), Some(APR21_MIDNIGHT_UTC));
    }

    #[test]
    fn epoch_itself() {
        assert_eq!(parse_pdf_date_to_epoch("D:19700101000000Z"), Some(0));
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(parse_pdf_date_to_epoch(""), None);
        assert_eq!(parse_pdf_date_to_epoch("D:not-a-date"), None);
        assert_eq!(parse_pdf_date_to_epoch("D:"), None);
    }

    #[test]
    fn rejects_invalid_month_day() {
        assert_eq!(parse_pdf_date_to_epoch("D:20260099"), None);
        assert_eq!(parse_pdf_date_to_epoch("D:20260400"), None);
    }

    // ── format_pdf_date latent-bug fix (#235 / README) ───────────────

    #[test]
    fn unix_epoch_formats_exactly() {
        assert_eq!(pdf_date_from_unix_secs(0), "D:19700101000000Z");
    }

    #[test]
    fn formats_real_date_not_hardcoded_jan_1() {
        // The bug hard-coded month/day to "0101" and approximated the
        // year as 1970 + days/365. Assert a real mid-year date with a
        // time-of-day round-trips through the module's own inverse.
        for s in [
            "D:20260516123456Z",
            "D:20240229080000Z", // leap day — old code could never emit 0229
            "D:20250301000001Z", // day after non-leap Feb
            "D:19991231235959Z",
            "D:20000101000000Z", // leap-century boundary
        ] {
            let epoch = parse_pdf_date_to_epoch(s).expect("parse fixture");
            assert!(epoch >= 0);
            assert_eq!(
                pdf_date_from_unix_secs(epoch as u64),
                s,
                "format/parse round-trip must be exact for {s}"
            );
        }
    }

    #[test]
    fn format_pdf_date_utc_is_well_formed_and_current() {
        let d = format_pdf_date_utc();
        assert!(d.starts_with("D:") && d.ends_with('Z') && d.len() == 17);
        // Year is the real current year, not a leap-drifted
        // 1970+days/365. Derive the expected year from the system
        // clock (independently of the code under test) so this never
        // becomes a hard-coded calendar time-bomb and tolerates a
        // year-boundary / clock-skew race with a ±1 window.
        let yr: i64 = d[2..6].parse().expect("year digits");
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock before UNIX epoch")
            .as_secs();
        // 31_556_952 = mean Gregorian year length in seconds.
        let approx_year = 1970 + (now / 31_556_952) as i64;
        assert!(
            (approx_year - 1..=approx_year + 1).contains(&yr),
            "year {yr} in {d} is not within ±1 of the current year (~{approx_year})"
        );
        // Month/day are real (not the old constant 0101 unless it truly is).
        let mo: u32 = d[6..8].parse().unwrap();
        let dy: u32 = d[8..10].parse().unwrap();
        assert!((1..=12).contains(&mo) && (1..=31).contains(&dy));
        // Must round-trip back to the same instant.
        let e = parse_pdf_date_to_epoch(&d).expect("self-parse");
        assert_eq!(pdf_date_from_unix_secs(e as u64), d);
    }
}
