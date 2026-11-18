//! 고정형·관계형: SO-SUEOP의 HWP3 사적 문장부호 decode와 저장 소스 보존.
//! 주석 갱신: 2026-11-09T18:16:58+09:00. 직전 파일 변경: 3c2c5fdf1bea13912b2eefaf4008ea974a2b4fd9 2026-11-08T23:26:50+09:00.
//! 시각 측정: 2026-11-09 KST, pdf/SO-SUEOP-hwp-2020.pdf, 96 DPI, print profile, compat 2022.
//! 전체 46쪽 silhouette_raw_match_percent 최저: Native Skia 61.69802%,
//! fresh WASM Canvas2D 61.97235%. 기존 미달이 남아 수용 보류이며 PASS로 표현하지 않는다.
//! 전쪽 TSV·producer·직접 판독: mydocs/pr/assets/chrisryugj-20261109/README.md.
//! 아래 검사는 Unicode/PUA와 실제 본문·각주의 저장 소스 계약이다. px·golden·쪽수를 고정하지 않는다.
//! 독립 samples/SO-SUEOP.hwpx의 XML text/tail을 모두 세어 문자 보존을 비교한다.

use rhwp::parser::hwp3::johab::decode_johab;

#[test]
fn so_sueop_private_punctuation_decodes() {
    let measured: &[(u16, char)] = &[
        (0x3062, '“'),
        (0x3063, '”'),
        (0x3066, '‧'),
        (0x2014, '—'),
        (0x2223, '∣'),
        (0x21B3, '↳'),
    ];
    for &(code, expected) in measured {
        assert_eq!(
            decode_johab(code),
            expected,
            "0x{code:04X} 가 다시 버려진다"
        );
    }
}

/// 책괄호 두 코드는 변환본이 한컴 PUA(F0854·F0855, 흰 겹낫표 반각)로 저장한다 —
/// 렌더러의 공통 한컴 PUA 표가 표시값을 맡는 계약(0x3366·0x37C0~)과 같다.
#[test]
fn so_sueop_book_brackets_keep_the_hancom_pua() {
    assert_eq!(decode_johab(0x30BB), '\u{F0854}');
    assert_eq!(decode_johab(0x30BD), '\u{F0855}');
}

fn document_punctuation_counts(bytes: &[u8]) -> std::collections::BTreeMap<char, usize> {
    use std::io::Read;
    let mut counts: std::collections::BTreeMap<_, _> =
        ['“', '”', '‧', '—', '∣', '↳', '\u{F0854}', '\u{F0855}']
            .into_iter()
            .map(|c| (c, 0))
            .collect();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("HWPX ZIP");
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).expect("ZIP entry");
        let name = file.name().expect("ZIP entry name").into_owned();
        if !name.starts_with("Contents/section") || !name.ends_with(".xml") {
            continue;
        }
        let mut xml = String::new();
        file.read_to_string(&mut xml).expect("section XML");
        let doc = roxmltree::Document::parse(&xml).expect("독립 XML");
        // hp:t의 첫 text만 세면 혼합 내용의 tail·각주를 잃는다. XML text node를 한 번씩 센다.
        for node in doc.descendants().filter(|n| n.is_text()) {
            for c in node.text().unwrap_or_default().chars() {
                if let Some(count) = counts.get_mut(&c) {
                    *count += 1;
                }
            }
        }
    }
    counts
}

#[test]
fn actual_document_keeps_punctuation_and_note_book_brackets_in_saved_source() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read(root.join("samples/SO-SUEOP.hwp")).expect("원 HWP3");
    let reference = std::fs::read(root.join("samples/SO-SUEOP.hwpx")).expect("독립 한컴 변환본");
    let doc = rhwp::parser::hwp3::parse_hwp3(&source).expect("HWP3 파싱");
    let saved = rhwp::serializer::hwpx::serialize_hwpx(&doc).expect("HWPX 저장");
    let expected = document_punctuation_counts(&reference);
    assert!(
        expected.values().all(|&n| n > 0),
        "독립 원본에 8종이 모두 존재"
    );
    assert_eq!(
        document_punctuation_counts(&saved),
        expected,
        "본문·각주의 문자 보존"
    );
}
