//! 구형 차트의 자료·표현 아카이브가 공유하는 문자열 해석.

use encoding_rs::EUC_KR;

/// UTF-16 사본이 있으면 이를 정본으로 읽고, 없는 작성기만 CP949로 읽는다.
/// ANSI 사본 뒤의 두 NUL이 경계이며 UTF-16 시작의 짝수 정렬은 요구하지 않는다.
/// 호출자가 길이와 예약 바이트를 분리하며, 표시용 공백 정규화는 여기서 하지 않는다.
pub(super) fn decode(payload: &[u8]) -> Option<String> {
    if let Some(split) = payload.windows(2).position(|pair| pair == [0, 0]) {
        let tail = &payload[split + 2..];
        if tail.len() >= 2 && tail.ends_with(&[0, 0]) && (tail.len() - 2).is_multiple_of(2) {
            let units: Vec<u16> = tail[..tail.len() - 2]
                .chunks_exact(2)
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                .collect();
            return String::from_utf16(&units).ok();
        }
        return decode_ansi(&payload[..split]);
    }
    decode_ansi(payload)
}

fn decode_ansi(payload: &[u8]) -> Option<String> {
    let (text, _, errors) = EUC_KR.decode(payload);
    (!errors).then(|| text.into_owned())
}
