//! [#7702] 양쪽 정렬 줄 끝에 걸린 공백은 배분 자리가 아니다 — 저장 줄 없는 줄도 같다.
//!
//! 입력은 저장 줄이 없는 실문서 HWPX `samples/issue2470/36341511_masked.hwpx` 다. 기준은 같은
//! 문서의 한/글 2022 출력 `pdf/issue2470/36341511_masked-2022.pdf` 이고, 값은 그 PDF 에서 잰
//! 양쪽 정렬 배분 줄(문단 마지막 줄이 아닌 줄)의 **마지막 가시 글자 원점 x**(px, 96dpi)다.
//!
//! 한/글은 줄 끝에 걸린 공백을 오른쪽 여백 바깥에 두고, 그 앞 글자가 본문 오른쪽 끝에서
//! 끝나도록 내부 공백만 늘린다. 수정 전 rhwp 는 저장 줄 없는 줄에서 이 공백까지 분모(내부 공백
//! + 1)와 사용 폭에 넣어, 마지막 글자가 6.6 ~ 31px 왼쪽에 그려졌다.
//!
//! 허용 2.0px 은 이 문서의 다른 줄에도 공통으로 있는 글자 전진 차(줄 끝 +1 ~ +2.5px, 이 수정과
//! 무관)를 덮는 값이고, 결함 크기(6px 이상)와 겹치지 않는다. 줄 머리 공백의 신장(5쪽
//! `-원활한…` 줄 첫 글자 +84px)은 별도 규칙이라 여기서 보지 않는다.

use rhwp::document_core::DocumentCore;

const TOLERANCE_PX: f64 = 2.0;

/// (줄 머리 글, 줄 끝 글, 한/글 마지막 글자 x px) — 저장 줄 없이 다시 나뉘는 양쪽 정렬 배분 줄.
/// 머리·끝 글이 같은 줄이 여럿이면 모두 같은 값이어야 한다(7쪽 «▶입찰참가등록전일부터» 두 줄).
const HANGUL_JUSTIFIED_LINE_ENDS: &[(&str, &str, f64)] = &[
    (
        "-원활한협잡물제거및근무자환경개선을위해",
        "경개선을위해",
        709.12,
    ), // 5쪽
    (
        "▶입찰공고일전일부터입찰일(낙찰자는계약체결일)까",
        "996)또는",
        710.40,
    ), // 7쪽
    (
        "▶입찰참가등록전일부터법인등기부상본점소재지(개인",
        "사업자등록증",
        710.24,
    ), // 7쪽 ×2
];

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    let end = start + tag[start..].find('"')?;
    Some(&tag[start..end])
}

fn origin(tag: &str) -> Option<(f64, f64)> {
    if let (Some(x), Some(y)) = (attr(tag, "x"), attr(tag, "y")) {
        return Some((x.parse().ok()?, y.parse().ok()?));
    }
    let transform = attr(tag, "transform")?;
    let args = transform.strip_prefix("translate(")?;
    let args = &args[..args.find(')')?];
    let (x, y) = args.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}

/// 한 쪽의 글자 줄: x 순 (x, 글자) 목록, 위에서 아래 순.
fn page_glyph_lines(core: &DocumentCore, page: u32) -> Vec<Vec<(f64, String)>> {
    let svg = core.render_page_svg_native(page).expect("SVG");
    let mut lines: Vec<(f64, Vec<(f64, String)>)> = Vec::new();
    let mut rest = svg.as_str();
    while let Some(open) = rest.find("<text ") {
        rest = &rest[open..];
        let (Some(tag_end), Some(close)) = (rest.find('>'), rest.find("</text>")) else {
            break;
        };
        let tag = &rest[..tag_end];
        let text = rest[tag_end + 1..close].trim().to_string();
        if let (Some((x, y)), false) = (origin(tag), text.is_empty()) {
            match lines
                .iter_mut()
                .find(|(line_y, _)| (*line_y - y).abs() < 2.0)
            {
                Some((_, line)) => line.push((x, text)),
                None => lines.push((y, vec![(x, text)])),
            }
        }
        rest = &rest[close..];
    }
    lines.sort_by(|a, b| a.0.total_cmp(&b.0));
    lines
        .into_iter()
        .map(|(_, mut line)| {
            line.sort_by(|a, b| a.0.total_cmp(&b.0));
            line
        })
        .collect()
}

#[test]
fn justified_synthetic_line_hangs_its_end_space_outside_the_distribution() {
    let bytes = std::fs::read("samples/issue2470/36341511_masked.hwpx").expect("표본");
    let core = DocumentCore::from_bytes(&bytes).expect("파싱");

    let lines: Vec<Vec<(f64, String)>> = (0..core.page_count())
        .flat_map(|page| page_glyph_lines(&core, page))
        .collect();
    let mut failures = Vec::new();
    for (head, tail, hangul_x) in HANGUL_JUSTIFIED_LINE_ENDS {
        let matched: Vec<f64> = lines
            .iter()
            .filter(|line| {
                let text: String = line.iter().map(|(_, t)| t.as_str()).collect();
                text.starts_with(head) && text.ends_with(tail)
            })
            .map(|line| line.last().expect("글자").0)
            .collect();
        assert!(
            !matched.is_empty(),
            "«{head}…{tail}» 줄을 찾지 못했다 — 줄 나눔부터 어긋났다"
        );
        for x in matched {
            if (x - hangul_x).abs() > TOLERANCE_PX {
                failures.push(format!(
                    "«{head}…{tail}» 마지막 글자 x rhwp {x:.2} · 한/글 {hangul_x:.2} (차 {:+.2})",
                    x - hangul_x
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "양쪽 정렬 줄 마지막 글자가 본문 오른쪽 끝에 닿지 않는다 — 줄 끝에 걸린 공백이 배분 \
         몫을 가져갔다:\n{}",
        failures.join("\n")
    );
}
