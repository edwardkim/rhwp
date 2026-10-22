//! WMF의 원본 바이트 셀 전진을 해독한 문자열의 원점으로 연결한다.

use crate::wmf::parser::{CharacterSet, ExtTextOutOptions, META_EXTTEXTOUT};
use unicode_segmentation::UnicodeSegmentation;

pub(super) struct ExplicitTextAdvances<'a> {
    pub(super) cells: Vec<(&'a str, i32)>,
    pub(super) total: i32,
}

pub(super) fn explicit_advances<'a>(
    record: &META_EXTTEXTOUT,
    charset: CharacterSet,
    decoded: &'a str,
) -> Option<ExplicitTextAdvances<'a>> {
    if record.dx.is_empty()
        || record.dx.len() != record.string.len()
        || charset == CharacterSet::SYMBOL_CHARSET
        || record.fw_opts.contains(&ExtTextOutOptions::ETO_PDY)
        || record.fw_opts.contains(&ExtTextOutOptions::ETO_GLYPH_INDEX)
    {
        return None;
    }
    let encoding: &'static encoding_rs::Encoding = charset.into();
    let mut offset = 0;
    let mut advance = 0;
    let mut cells = Vec::new();
    for grapheme in decoded.graphemes(true) {
        // into_utf8가 제거한 NUL도 원본 Dx 셀의 순번은 차지한다.
        while record.string.get(offset) == Some(&0) {
            advance += i32::from(record.dx[offset]);
            offset += 1;
        }
        let (bytes, _, errors) = encoding.encode(grapheme);
        let end = offset.checked_add(bytes.len())?;
        if errors || record.string.get(offset..end)? != bytes.as_ref() {
            return None;
        }
        cells.push((grapheme, advance));
        advance += record.dx[offset..end]
            .iter()
            .map(|&dx| i32::from(dx))
            .sum::<i32>();
        offset = end;
    }
    while record.string.get(offset) == Some(&0) {
        advance += i32::from(record.dx[offset]);
        offset += 1;
    }
    (offset == record.string.len()).then_some(ExplicitTextAdvances {
        cells,
        total: advance,
    })
}
