//! [Issue #5802] 611쪽 연구보고서에서 머리말·꼬리말(쪽번호)이 짝수 쪽 전량(~300쪽)
//! 그려지지 않았다.
//!
//! 근인 두 축:
//! 1) **구역 간 상속 부재** — 짝수 머리말/꼬리말 정의가 구역 3·4에만 있는데 구역별
//!    finalize 가 신선한 선택기로 시작해 구역 7+(485쪽)의 짝수 쪽이 전부 비었다.
//!    렌더 측은 `source_section_index` 해석을 이미 지원했으므로, paginate_pass 말미에
//!    선택기를 구역 순서로 이월해 **비어 있는 쪽만** 채운다.
//! 2) **HF 문단의 TAC 도형 소실** — 꼬리말 내용이 글자처럼 취급 묶음(쪽번호 자동번호
//!    포함)인데 layout_paragraph 를 거치지 않아 inline 좌표 미등록 → #476 가드가
//!    조용히 스킵했다. HF 전용 센티널 구역 키로 등록·조회를 잇는다.
//!
//! 픽스처는 원본 HWP5 의 구역3(정의 문단 14..17)+구역7(0..4) 절단 + BinData 1×1
//! 스텁 축소본(32KB). 1쪽=홀수(자기 구역 홀수 머리말), 2쪽=짝수(구역 간 상속 짝수
//! 머리말 — 결함 시 빈 머리말).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5802/hf_cross_section_inherit.hwp";

fn header_text(svg: &str) -> String {
    // 머리말 영역(y<200)의 글자만 모은다. <text ... translate(x, y) ...>c</text>
    let mut out = String::new();
    for cap in svg.split("<text ").skip(1) {
        let Some(end) = cap.find("</text>") else {
            continue;
        };
        let node = &cap[..end];
        let y = node
            .split_once("translate(")
            .and_then(|(_, rest)| rest.split_once(')'))
            .and_then(|(args, _)| args.split(',').nth(1))
            .and_then(|v| v.trim().parse::<f64>().ok())
            .or_else(|| {
                node.split_once("y=\"")
                    .and_then(|(_, rest)| rest.split_once('"'))
                    .and_then(|(v, _)| v.parse::<f64>().ok())
            });
        if let (Some(y), Some(gt)) = (y, node.rfind('>')) {
            if y < 200.0 {
                out.push_str(&node[gt + 1..]);
            }
        }
    }
    out
}

#[test]
fn issue_5802_even_page_inherits_header_from_earlier_section() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    // 1쪽(홀수): 자기 구역의 홀수 머리말 — 상속 이전에도 그려지던 경로가 유지돼야 한다.
    let p1 = core.render_page_svg_native(0).expect("page 1 svg");
    let h1 = header_text(&p1);
    assert!(
        h1.contains("1장"),
        "1쪽 홀수 머리말(1장…)이 있어야 한다: {h1:?}"
    );

    // 2쪽(짝수): 짝수 머리말 정의는 앞 구역에만 있다 — 구역 간 상속이 채워야 한다.
    let p2 = core.render_page_svg_native(1).expect("page 2 svg");
    let h2 = header_text(&p2);
    assert!(
        h2.contains("농업직업교육체계"),
        "2쪽 짝수 머리말이 앞 구역에서 상속돼야 한다 (결함 시 빈 머리말): {h2:?}"
    );
}
