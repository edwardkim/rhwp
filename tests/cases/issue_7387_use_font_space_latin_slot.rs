//! #7387: `CharShape.useFontSpace` 가 켜진 run 의 공백은 반각이 아니라
//! **영문 슬롯 글꼴이 선언한 공백 전진폭**이다.
//!
//! 종전에는 `useFontSpace` 를 렌더러가 아예 읽지 않아 공백이 언제나 `em/2` 였다.
//!
//! # 왜 절대 좌표로 재는가
//!
//! 이 검사는 `em` 으로 정규화하지 않는다. 앞선 PR(#7389)은 `exam_eng` 를 앵커로
//! 잡고 `폭 / 글꼴크기` 를 정본의 `폭 / 글꼴크기` 와 비교했는데, **그 문서는 정본과
//! rhwp 의 글꼴 크기가 6.43% 다르다**(#7398). 분모가 서로 달라, 고쳐진 뒤에도 남아 있던
//! 5.4% 폭 부족이 상쇄되어 "일치"로 보였다. 그래서 그 PR 을 닫았다.
//!
//! 여기서는 글꼴 크기 confound 가 없는 문서(`scripts/oracle_comparability.py` 판정
//! `fontScale` 0.996)를 앵커로 쓰고, **px 절대 폭**을 본다.
//!
//! # 기대값
//!
//! 한컴 정본 `pdf/hwpctl_API_v2.4-2022.pdf` 를 `mutool draw -F stext` 로 읽어, 같은
//! 글자열 줄의 첫 글자 `x0` 부터 마지막 글자 `x1` 까지를 96dpi px 로 잰 값이다.
//!
//! ```text
//!   쪽   정본 잉크폭   수정 전 점유폭   수정 후 점유폭
//!   14      416.66      432.50 (+15.8)   416.60 (-0.1)
//!   49      430.42      454.20 (+23.8)   432.30 (+1.9)
//!   88      302.42      322.10 (+19.7)   306.20 (+3.8)
//! ```
//!
//! rhwp 쪽은 run 의 **점유폭**이라 마지막 글자의 전진폭까지 포함한다 — 정본의 잉크폭보다
//! 한 글자 몫 안쪽에서 조금 크다. 그래서 등식이 아니라 `±8px` 구간으로 판정한다.
//! 수정 전 값은 전부 그 구간 위에 있다.
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

/// 정본 PDF 의 쪽 너비(595.0pt)를 96dpi px 로 환산한 값. 기대값을 rhwp 쪽 상자에 맞춘다.
const ORACLE_PAGE_PX: f64 = 595.0 * 96.0 / 72.0;
const TOLERANCE_PX: f64 = 8.0;

struct Run {
    text: String,
    x: f64,
    width: f64,
    y: f64,
    font_size: f64,
}

fn collect(node: &RenderNode, out: &mut Vec<Run>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push(Run {
            text: run.text.clone(),
            x: node.bbox.x,
            width: node.bbox.width,
            y: node.bbox.y,
            font_size: run.style.font_size,
        });
    }
    for child in &node.children {
        collect(child, out);
    }
}

fn open(name: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("samples/{name}"));
    DocumentCore::from_bytes(&std::fs::read(path).expect("공개 회귀 문서")).expect("문서 파싱")
}

fn strip(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 같은 baseline 의 run 을 모아 글자열이 `needle` 인 줄의 점유폭과 쪽 너비를 낸다.
fn line_span(doc: &DocumentCore, page: u32, needle: &str) -> Option<(f64, f64)> {
    let tree = doc.build_page_render_tree(page - 1).ok()?;
    let mut runs = Vec::new();
    collect(&tree.root, &mut runs);
    let mut baselines: Vec<f64> = runs.iter().map(|r| r.y).collect();
    baselines.sort_by(|a, b| a.partial_cmp(b).unwrap());
    baselines.dedup_by(|a, b| (*a - *b).abs() < 0.5);
    for baseline in baselines {
        let line: Vec<&Run> = runs
            .iter()
            .filter(|r| (r.y - baseline).abs() < 0.5)
            .collect();
        let joined: String = {
            let mut sorted: Vec<&&Run> = line.iter().collect();
            sorted.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
            strip(&sorted.iter().map(|r| r.text.as_str()).collect::<String>())
        };
        if joined == needle {
            let left = line.iter().fold(f64::MAX, |a, r| a.min(r.x));
            let right = line.iter().fold(f64::MIN, |a, r| a.max(r.x + r.width));
            return Some((right - left, tree.root.bbox.width));
        }
    }
    None
}

#[test]
fn use_font_space_line_width_matches_hancom_in_absolute_px() {
    let doc = open("hwpctl_API_v2.4.hwp");
    for (page, needle, oracle_ink_px) in [
        (
            14u32,
            "되고,대화상자가닫힌후에는사용자가지정한값들이담겨돌아온다.",
            416.66_f64,
        ),
        (
            49,
            "option:다음과같은옵션을지정할수있다.0을지정하면모두off이다.",
            430.42,
        ),
        (88, "ToolBarID:미리정의된툴바의이름이올수있다.", 302.42),
    ] {
        let (span, page_px) =
            line_span(&doc, page, needle).unwrap_or_else(|| panic!("{page}쪽의 `{needle}` 줄"));
        // 정본 쪽 상자에 맞춰 환산한다. 두 쪽 너비는 0.05% 안이라 보정은 미미하지만
        // 기대값이 어느 좌표계의 값인지 남겨 둔다.
        let expected = oracle_ink_px * page_px / ORACLE_PAGE_PX;
        assert!(
            (span - expected).abs() <= TOLERANCE_PX,
            "{page}쪽 `{needle}` 줄 점유폭 {span:.2}px 이 정본 {expected:.2}px 에서 \
             {:.2}px 벗어났다(허용 {TOLERANCE_PX}px). useFontSpace 를 무시하고 공백을 \
             반각으로 전진시키면 공백마다 넓어진다.",
            span - expected,
        );
    }
}

/// 반례 가드 — 규칙은 상수가 아니라 **영문 슬롯 글꼴의 값**이다.
///
/// `1382000_domestic_violence_survey.hwp` 의 charPr 26 은 `useFontSpace="1"` 인데
/// 영문 슬롯이 `휴먼명조`(대체 없이 그대로 쓰인다)이고 그 글꼴의 공백은 256/512 =
/// 0.5 em 이다. 정본 p19 에서 이 charPr 의 공백은 n=38 관측 최빈 **0.489 em** 으로,
/// 같은 쪽 다른 `useFontSpace="1"` charPr 들의 0.337 em 과 뚜렷이 갈린다.
///
/// 이 검사는 수정 전에도 통과한다(수정 전 391.40px / 수정 후 392.00px) — 결함 검출
/// 증거가 아니라, 뒤에 이 갈래를 상수로 눌러 버리는 변경을 막는 경계다. 공백 6개짜리
/// 줄이라 0.25 em 으로 누르면 `6 × 0.25 × 14.667px = 22px` 가 빠져 약 370px 이 되고
/// 아래 구간을 벗어난다.
#[test]
fn latin_slot_with_half_em_space_keeps_half_width() {
    let doc = open("task2430/1382000_domestic_violence_survey.hwp");
    // 줄 앞의 `7.` 은 문단 자동 번호라 별도 run 이다. 줄 전체를 짝짓는다.
    let needle = "7.만일이연구에참여하지않는다면불이익이있습니까?";
    let (span, _) = line_span(&doc, 17, needle).expect("17쪽의 7번 문항 줄");
    assert!(
        (382.0..402.0).contains(&span),
        "17쪽 7번 문항 점유폭 {span:.2}px 이 382~402px 밖이다. \
         영문 슬롯이 0.5 em 공백을 선언한 run 까지 좁히면 안 된다.",
    );
}

/// 폭 표가 **메트릭 DB 밖**에 있는 영문 슬롯 — KoPub.
///
/// KoPub 은 메트릭 DB 가 아니라 전용 표(`ttfs/kopub` 의 `hmtx` 직독)가 폭을 소유한다.
/// 공백폭 해석이 DB 만 보던 동안 `useFontSpace` 가 켜진 KoPub run 은 반각으로 남았다.
///
/// 정본 `pdf/2025 행정업무운영 편람(최종)-2020-kopub.pdf`(Hwp 2020 11.0.0.9136 ·
/// Hancom PDF 1.3.0.550, KoPub 서브셋 내장)의 377쪽 「문서관리카드」 상자다. 같은 폴더의
/// `-hwp-kopub-2020.pdf` 는 cairo 산출이라 가로 폭 기준으로 쓰지 않는다. 두 줄의 공백은 charPr 874
/// (`useFontSpace=1`, 영문 슬롯 `KoPub돋움체 Light` 10pt — 글꼴 공백 290/1000)이다.
/// 기대값은 그 줄 첫 공백의 원점부터 마지막 `]` 의 오른쪽 끝까지를 96dpi px 로 잰 값이다.
///
/// ```text
///   줄                         공백  정본 폭   수정 전          수정 후
///   "  [과제카드명]"              2    69.40    73.6 (+4.2)     68.3 (-1.1)
///   " [열람범위]  …  [열람제한]"  19   175.80   217.6 (+41.8)   168.6 (-7.2)
/// ```
///
/// 정본의 연속 공백 덩어리는 `13 / 18` 칸 모두 공백당 3.89~3.90px(= 0.29 em)이다.
/// 수정 후에도 공백당 약 0.35px 이 모자란다 — 이 charPr 은 한글 슬롯만 장평 98%·
/// 자간 -6% 이고 영문 슬롯은 100%·0 인데, rhwp 는 공백에 run(한글 슬롯)의 장평·자간을
/// 적용한다. 그 축은 이 검사의 범위 밖이라 허용폭으로 둔다(공백당 0.5px).
#[test]
fn use_font_space_kopub_latin_slot_uses_kopub_space_glyph() {
    const ORACLE_PAGE_PT: f64 = 754.0;
    const CARD_LINES: [&str; 2] = ["  [과제카드명]", " [열람범위]                  [열람제한]"];
    let doc = open("2025 행정업무운영 편람(최종).hwp");
    // 정본은 이 상자를 377쪽에 둔다. rhwp 의 이 문서 전체 쪽수는 정본(383)과 달라
    // (앞쪽 표 분할 축, 이 검사의 범위 밖) 같은 상자가 몇 쪽 앞에 놓인다. 폭은 쪽 번호와
    // 무관하므로 쪽 번호로 고정하지 않고 두 줄을 함께 품은 쪽을 내용으로 찾는다.
    let (tree, runs) = (0..doc.page_count())
        .rev()
        .find_map(|page| {
            let tree = doc.build_page_render_tree(page).ok()?;
            let mut runs = Vec::new();
            collect(&tree.root, &mut runs);
            CARD_LINES
                .iter()
                .all(|text| runs.iter().any(|r| r.text == *text))
                .then_some((tree, runs))
        })
        .expect("「문서관리카드」 두 줄을 함께 품은 쪽");
    let page_px = tree.root.bbox.width;
    for (text, spaces, oracle_px) in [
        (CARD_LINES[0], 2.0_f64, 69.40_f64),
        (CARD_LINES[1], 19.0, 175.80),
    ] {
        let run = runs
            .iter()
            .find(|r| r.text == text)
            .unwrap_or_else(|| panic!("「문서관리카드」의 `{text}` run"));
        let expected = oracle_px * page_px / (ORACLE_PAGE_PT * 96.0 / 72.0);
        let tolerance = 1.5 + 0.5 * spaces;
        assert!(
            (run.width - expected).abs() <= tolerance,
            "「문서관리카드」 `{text}` 점유폭 {:.2}px 이 정본 {expected:.2}px 에서 {:.2}px 벗어났다\
             (허용 {tolerance:.1}px, 글꼴 {:.2}px). useFontSpace 영문 슬롯이 KoPub 이면 \
             공백은 반각이 아니라 KoPub 공백 글리프(290/1000 em)다.",
            run.width,
            run.width - expected,
            run.font_size,
        );
    }
}
