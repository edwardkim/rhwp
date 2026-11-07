//! HWP 메모 명령은 UTC FILETIME을 하위·상위 DWORD 순으로 저장한다.
use chrono::{DateTime, Local, Utc};

const UNIX_EPOCH_TICKS: i64 = 116_444_736_000_000_000;
const TICKS_PER_SECOND: i64 = 10_000_000;

pub(crate) fn now() -> u64 {
    let time = Utc::now();
    (UNIX_EPOCH_TICKS
        + time.timestamp() * TICKS_PER_SECOND
        + i64::from(time.timestamp_subsec_nanos() / 100)) as u64
}

pub(crate) fn date(command: &str) -> Option<String> {
    let mut parts = command.strip_prefix("MEMO/")?.split('/').skip(2);
    let low = parts.next()?.parse::<u32>().ok()?;
    let high = parts.next()?.parse::<u32>().ok()?;
    let ticks = i64::try_from(u64::from(high) << 32 | u64::from(low)).ok()?;
    if ticks == 0 {
        return None;
    }
    let unix_ticks = ticks - UNIX_EPOCH_TICKS;
    let date = DateTime::from_timestamp(
        unix_ticks.div_euclid(TICKS_PER_SECOND),
        (unix_ticks.rem_euclid(TICKS_PER_SECOND) * 100) as u32,
    )?;
    // 한글 SaveAs는 현지 시각에 리터럴 Z를 붙인다.
    Some(
        date.with_timezone(&Local)
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string(),
    )
}
