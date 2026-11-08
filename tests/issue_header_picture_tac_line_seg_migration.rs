//! 머리말/꼬리말 그림 treatAsChar 토글 시 앵커 문단 line_segs 미갱신 회귀 가드.
//!
//! 본문 그림(set_picture_properties_native)은 [Task #1151 v2] 마이그레이션으로
//! treatAsChar false→true 시 앵커 line_segs[0].line_height 를 그림 높이로 갱신한다.
//! 머리말/꼬리말 경로(set_header_footer_picture_properties_native)는 이 마이그레이션이
//! 누락되어 있었다. 줄 점유는 그림 본체와 위·아래 바깥여백을 함께 포함한다.
//! 독립 Hancom2020 재저장/Print: Stage26 기록(3016=2448+284+284HU).

use rhwp::model::control::Control;
use std::fs;
use std::path::Path;

fn find_header_picture(hwp: &str) -> Option<(usize, usize, usize, usize, usize)> {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(hwp);
    let data = fs::read(&path).ok()?;
    let doc = rhwp::parser::hwp3::parse_hwp3(&data).ok()?;
    for (si, sec) in doc.sections.iter().enumerate() {
        for (bi, para) in sec.paragraphs.iter().enumerate() {
            for (hi, ctrl) in para.controls.iter().enumerate() {
                if let Control::Header(h) = ctrl {
                    for (ipi, ipara) in h.paragraphs.iter().enumerate() {
                        for (ici, ictrl) in ipara.controls.iter().enumerate() {
                            if matches!(ictrl, Control::Picture(_)) {
                                return Some((si, bi, hi, ipi, ici));
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[test]
fn header_picture_tac_on_updates_anchor_line_height() {
    use rhwp::document_core::DocumentCore;

    let repo_root = env!("CARGO_MANIFEST_DIR");
    let p = Path::new(repo_root).join("samples/hwp3-sample11.hwp");
    let data = fs::read(&p).expect("read");
    let mut core = DocumentCore::from_bytes(&data).expect("parse");
    let body_text = core
        .document()
        .sections
        .iter()
        .map(|s| {
            s.paragraphs
                .iter()
                .map(|p| p.text.clone())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let (si, bi, hi, ipi, ici) =
        find_header_picture("samples/hwp3-sample11.hwp").expect("header picture must exist");

    // 샘플의 초기 tac 상태와 무관하게 false→true 전환을 강제로 재현한다:
    // 먼저 false 로 만들어 floating 상태로 정규화한 뒤, true 로 토글해 마이그레이션을
    // 실제로 발동시킨다 (샘플 그림이 이미 tac=true 이면 두 번째 호출이 no-op 이 되어
    // 마이그레이션 미실행을 놓치는 것을 방지).
    core.set_header_footer_picture_properties_native(
        si,
        bi,
        hi,
        ipi,
        ici,
        r#"{"treatAsChar":false}"#,
    )
    .expect("set_header_footer_picture_properties_native(treatAsChar:false)");

    let (pic_height, top, bottom, header_text) =
        match &core.document().sections[si].paragraphs[bi].controls[hi] {
            Control::Header(h) => match &h.paragraphs[ipi].controls[ici] {
                Control::Picture(p) => (
                    p.common.height as i32,
                    p.common.margin.top as i32,
                    p.common.margin.bottom as i32,
                    h.paragraphs
                        .iter()
                        .map(|p| p.text.clone())
                        .collect::<Vec<_>>(),
                ),
                _ => panic!("expected picture"),
            },
            _ => panic!("expected header"),
        };

    core.set_header_footer_picture_properties_native(
        si,
        bi,
        hi,
        ipi,
        ici,
        r#"{"treatAsChar":true}"#,
    )
    .expect("set_header_footer_picture_properties_native(treatAsChar:true)");

    assert_eq!(
        core.document()
            .sections
            .iter()
            .map(|s| {
                s.paragraphs
                    .iter()
                    .map(|p| p.text.clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        body_text,
        "토글은 본문 구역/문단의 내용과 순서를 보존"
    );

    // 그림 본체만으로 줄 점유를 정하면 여백이 다음 내용과 겹친다.
    // 같은 편집 저장본을 한컴2020에서 재저장한 원본 줄은 본체+위/아래 여백과
    // 85% 기준선을 보존한다. renderer helper의 반환값을 기대값으로 쓰지 않는다.
    let expected_height = pic_height + top + bottom;
    assert!(top + bottom > 0, "여백이 있는 원본 경계를 실제로 검증");
    let Control::Header(header) = &core.document().sections[si].paragraphs[bi].controls[hi] else {
        panic!("expected header");
    };
    assert_eq!(
        header
            .paragraphs
            .iter()
            .map(|p| p.text.clone())
            .collect::<Vec<_>>(),
        header_text,
        "토글은 머리말 문단/본문을 추가·삭제·재배열하지 않는다"
    );
    let para = &header.paragraphs[ipi];
    let Control::Picture(pic) = &para.controls[ici] else {
        panic!("expected picture");
    };
    assert!(pic.common.treat_as_char);
    assert_eq!(pic.common.height as i32, pic_height, "본체 크기는 보존");
    assert_eq!(
        (
            pic.common.margin.top as i32,
            pic.common.margin.bottom as i32
        ),
        (top, bottom)
    );
    let seg = para
        .line_segs
        .first()
        .expect("inline picture owns an actual line");
    assert_eq!(
        seg.line_height, expected_height,
        "머리말 줄은 그림의 전체 세로 점유를 수용"
    );
    assert_eq!(
        seg.text_height, expected_height,
        "같은 구성의 텍스트/개체 점유"
    );
    assert_eq!(
        seg.baseline_distance,
        (f64::from(expected_height) * 0.85).round() as i32,
        "독립 재저장본의 같은 점유 기준선"
    );
}
