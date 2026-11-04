//! 관계형: 그룹 자식 도형의 원본 데이터와 저장 출력의 소유·바이트 보존.
//! 주석 갱신: 2026-10-31T01:36:11+09:00; 직전 파일 커밋: 2026-10-25T23:58:51+09:00 (d9c8a71b642fad1ee8221f4036817a23dfe15f10).
//! 실제 HWP의 그룹 자식 CTRL_DATA payload·레코드 소유와 저장 왕복을 비교한다. 조판 출력이나 px 좌표를 검사하지 않으므로 시각 수치는 비해당이다. 입력별 저장·편집 검증은 아래 검토 기록에 연결한다.
//! 근거: mydocs/pr/pr_8233_review.md.
//!
//! [#8225] 묶음 안 자식 도형의 `CTRL_DATA` 가 구역을 다시 쓰면 빠진다.
//!
//! 한컴은 묶음(`$con`) 안 자식 도형마다 `SHAPE_COMPONENT` 바로 아래에 `CTRL_DATA` 를 둔다
//! (`samples/basic/sungeo.hwp` 에는 개체 설명 글 "｢가｣항 내용 …" 같은 ParameterSet 이 실려 있다).
//! 파서는 컨트롤마다 첫 `CTRL_DATA` 하나만 `Paragraph.ctrl_data_records` 에 담고 자식 것은 읽지 않아,
//! 구역을 다시 직렬화하면 자식 도형의 설명 글이 통째로 사라졌다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

const HWPTAG_CTRL_DATA: u16 = 0x010 + 71;
const HWPTAG_SHAPE_COMPONENT: u16 = 0x010 + 60;

fn records(buf: &[u8]) -> Vec<(u16, u16, Vec<u8>)> {
    let mut out = Vec::new();
    let mut off = 0usize;
    while off + 4 <= buf.len() {
        let header = u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]]);
        off += 4;
        let tag = (header & 0x3FF) as u16;
        let level = ((header >> 10) & 0x3FF) as u16;
        let mut size = ((header >> 20) & 0xFFF) as usize;
        if size == 0xFFF {
            size =
                u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]]) as usize;
            off += 4;
        }
        let end = (off + size).min(buf.len());
        out.push((tag, level, buf[off..end].to_vec()));
        off = end;
    }
    out
}

/// 묶음 안 자식 도형(부모도 SHAPE_COMPONENT 인 SHAPE_COMPONENT) 바로 아래의 CTRL_DATA 들 (문서 차례)
fn group_child_ctrl_data(hwp: &[u8]) -> Vec<Vec<u8>> {
    let mut cfb = cfb::CompoundFile::open(std::io::Cursor::new(hwp.to_vec())).expect("CFB 열기");
    let compressed = {
        let mut stream = cfb.open_stream("/FileHeader").expect("FileHeader");
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut stream, &mut buf).expect("FileHeader 읽기");
        u32::from_le_bytes([buf[36], buf[37], buf[38], buf[39]]) & 1 == 1
    };
    let mut paths: Vec<String> = cfb
        .walk()
        .filter(|e| e.is_stream())
        .map(|e| e.path().to_string_lossy().replace('\\', "/"))
        .filter(|p| p.contains("BodyText/Section"))
        .collect();
    paths.sort_by_key(|p| {
        p.rsplit("Section")
            .next()
            .and_then(|n| n.parse::<u32>().ok())
    });
    let mut out = Vec::new();
    for path in paths {
        let mut raw = Vec::new();
        std::io::Read::read_to_end(&mut cfb.open_stream(&path).expect("구역"), &mut raw)
            .expect("구역 읽기");
        let body = if compressed {
            let mut decoded = Vec::new();
            std::io::Read::read_to_end(
                &mut flate2::read::DeflateDecoder::new(&raw[..]),
                &mut decoded,
            )
            .expect("압축 해제");
            decoded
        } else {
            raw
        };
        let mut stack: Vec<u16> = Vec::new();
        for (tag, level, data) in records(&body) {
            stack.truncate(level as usize);
            let parent = level
                .checked_sub(1)
                .and_then(|l| stack.get(l as usize))
                .copied();
            let grand = level
                .checked_sub(2)
                .and_then(|l| stack.get(l as usize))
                .copied();
            if tag == HWPTAG_CTRL_DATA
                && parent == Some(HWPTAG_SHAPE_COMPONENT)
                && grand == Some(HWPTAG_SHAPE_COMPONENT)
            {
                out.push(data.clone());
            }
            stack.resize(level as usize, 0);
            stack.push(tag);
        }
    }
    out
}

/// 구역 스트림의 CTRL_DATA 레코드 수 (자리와 상관없이)
fn ctrl_data_count(hwp: &[u8]) -> usize {
    let mut cfb = cfb::CompoundFile::open(std::io::Cursor::new(hwp.to_vec())).expect("CFB 열기");
    let mut header = Vec::new();
    std::io::Read::read_to_end(
        &mut cfb.open_stream("/FileHeader").expect("FileHeader"),
        &mut header,
    )
    .expect("FileHeader 읽기");
    let compressed = u32::from_le_bytes([header[36], header[37], header[38], header[39]]) & 1 == 1;
    let paths: Vec<String> = cfb
        .walk()
        .filter(|e| e.is_stream())
        .map(|e| e.path().to_string_lossy().replace('\\', "/"))
        .filter(|p| p.contains("BodyText/Section"))
        .collect();
    paths
        .iter()
        .map(|p| {
            let mut raw = Vec::new();
            std::io::Read::read_to_end(&mut cfb.open_stream(p).expect("구역"), &mut raw)
                .expect("구역 읽기");
            let body = if compressed {
                let mut d = Vec::new();
                std::io::Read::read_to_end(
                    &mut flate2::read::DeflateDecoder::new(&raw[..]),
                    &mut d,
                )
                .expect("압축 해제");
                d
            } else {
                raw
            };
            records(&body)
                .iter()
                .filter(|(t, _, _)| *t == HWPTAG_CTRL_DATA)
                .count()
        })
        .sum()
}

/// 구역을 모두 다시 직렬화한 HWP 바이트 (편집기에서 글을 고친 뒤 저장한 것과 같은 경로)
fn rewritten(bytes: &[u8]) -> Vec<u8> {
    let mut doc = rhwp::parse_document(bytes).expect("문서 읽기");
    for section in &mut doc.sections {
        section.raw_stream = None;
    }
    rhwp::serializer::serialize_document(&doc).expect("저장")
}

#[test]
fn group_child_ctrl_data_survives_section_rewrite() {
    let original =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/basic/sungeo.hwp"))
            .expect("samples/basic/sungeo.hwp");
    let expected = group_child_ctrl_data(&original);
    assert_eq!(
        expected.len(),
        24,
        "시험 문서의 묶음 자식 CTRL_DATA 수가 바뀌었다"
    );
    let text: String = expected
        .iter()
        .map(|d| {
            String::from_utf16_lossy(
                &d.chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert!(
        text.contains("항 내용"),
        "자식 CTRL_DATA 에 개체 설명 글이 있어야 한다"
    );

    let once = rewritten(&original);
    assert_eq!(
        group_child_ctrl_data(&once),
        expected,
        "다시 쓴 구역에서 자식 도형 CTRL_DATA 가 달라졌다"
    );

    // 저장본을 다시 열어 또 저장해도 그대로 (모델에 남는다)
    let twice = rewritten(&once);
    assert_eq!(group_child_ctrl_data(&twice), expected);
}

/// 반례: 묶음 자신에게는 CTRL_DATA 가 없고 첫 자식 그림에만 있다(하이퍼링크). 컨트롤 몫으로
/// 자식 것을 가져가면 묶음 아래와 자식 아래에 두 번 쓰인다.
#[test]
fn child_only_ctrl_data_is_not_written_twice() {
    let original = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue6542/156678235_mid_para_vpos_rewind.hwp"),
    )
    .expect("samples/issue6542/156678235_mid_para_vpos_rewind.hwp");
    assert_eq!(
        group_child_ctrl_data(&original).len(),
        1,
        "자식 그림 하나에만 CTRL_DATA 가 있는 문서여야 한다"
    );
    let out = rewritten(&original);
    assert_eq!(
        ctrl_data_count(&out),
        ctrl_data_count(&original),
        "CTRL_DATA 수가 원본과 달라졌다"
    );
    assert_eq!(
        group_child_ctrl_data(&out),
        group_child_ctrl_data(&original)
    );
}

/// 다른 한컴 문서에서도 묶음 자식 CTRL_DATA 가 남는다 (이슈에 적힌 1개 손실 문서)
#[test]
fn group_child_ctrl_data_survives_in_multi_section_document() {
    let original =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp-multi-001.hwp"))
            .expect("samples/hwp-multi-001.hwp");
    let expected = group_child_ctrl_data(&original);
    assert!(
        !expected.is_empty(),
        "묶음 자식 CTRL_DATA 가 있는 문서여야 한다"
    );
    let out = rewritten(&original);
    assert_eq!(group_child_ctrl_data(&out), expected);
    assert_eq!(ctrl_data_count(&out), ctrl_data_count(&original));
}
