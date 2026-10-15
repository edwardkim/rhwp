//! [#7824] 저장 줄 없는 HWPX 에서 문단 위 간격이 큰 마지막 문단은 쪽 끝에서 다음 쪽으로
//! 넘어간다.
//!
//! 입력 `samples/issue7824/synth_ladder_spacing_before.hwpx` 는 #7816 실험의 합성 문서다
//! (문단 모양 45 를 복제해 문단 위 간격만 0·2000·6000·10000·20000 으로 바꿨다 — 모두 짝수,
//! 최대 133.3px). 저장 줄(`linesegarray`)과 rhwp 내부 표지 `META-INF/rhwp-hwp5-origin` 이 없어
//! rhwp 는 로드 때 줄을 합성한다. 기대값은 같은 문서를 한/글 2020(MCP engine 2020)으로 출력한
//! `pdf/issue7824/synth_ladder_spacing_before-2020.pdf` 다 — 2쪽이고, 마지막 문단(3줄)의 첫 줄만
//! 1쪽에 남고 나머지 두 줄이 2쪽으로 간다.
//!
//! 수정 전 rhwp 는 쪽 맞춤 높이(`height_for_fit`)를 합성 줄 사다리의 범위로 깎아 위 간격
//! (133.3px)을 빼먹고, 마지막 문단을 통째로 1쪽에 넣어 본문 아래로 37.8px 넘쳤다(1쪽).

use rhwp::document_core::DocumentCore;

const FIXTURE: &[u8] = include_bytes!("../../samples/issue7824/synth_ladder_spacing_before.hwpx");

/// 한/글 PDF 2쪽의 글자(공백 제외)
const HANGUL_PAGE2_TEXT: &str =
    "줄정보가없는문서를한글이어떻게다시나누는지살펴보려고이문장을충분히길게적어둔다.";

fn page_text(core: &DocumentCore, page: u32) -> String {
    let svg = core.render_page_svg_native(page).expect("SVG");
    let mut glyphs: Vec<(f64, f64, String)> = Vec::new();
    let mut rest = svg.as_str();
    while let Some(open) = rest.find("<text ") {
        rest = &rest[open..];
        let (Some(tag_end), Some(close)) = (rest.find('>'), rest.find("</text>")) else {
            break;
        };
        let tag = &rest[..tag_end];
        let number = |name: &str| -> Option<f64> {
            let key = format!(" {name}=\"");
            let start = tag.find(&key)? + key.len();
            tag[start..start + tag[start..].find('"')?].parse().ok()
        };
        if let (Some(x), Some(y)) = (number("x"), number("y")) {
            glyphs.push((y, x, rest[tag_end + 1..close].to_string()));
        }
        rest = &rest[close..];
    }
    glyphs.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
    glyphs
        .into_iter()
        .map(|(_, _, text)| text)
        .collect::<String>()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

#[test]
fn large_spacing_before_paragraph_moves_to_next_page_like_hangul() {
    let core = DocumentCore::from_bytes(FIXTURE).expect("시험 문서 열기");
    assert_eq!(core.page_count(), 2, "한/글은 2쪽");
    assert_eq!(
        page_text(&core, 1),
        HANGUL_PAGE2_TEXT,
        "마지막 문단의 둘째·셋째 줄만 2쪽에 온다"
    );
}
