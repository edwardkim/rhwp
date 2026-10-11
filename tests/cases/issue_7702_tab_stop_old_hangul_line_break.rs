//! [#7702] 줄 나눔은 탭을 문단 탭 정지까지 전진하고, 옛한글 자모 묶음을 한 글자로 잰다 —
//! 그리기와 같은 규칙이다.
//!
//! 입력 `samples/issue7702/tab_stop_old_hangul_line_break.hwpx` 는 HWP5-nopassword 14쪽 문단 하나를
//! 남긴 HWPX 다. 문단에는 저장 줄(`linesegarray`)이 없어 한/글도 줄을 새로 나눈다.
//! 문단 모양은 왼쪽 여백 949HU(12.65px) · 탭 정지 3328HU 이고, 첫 글자 앞에 탭 하나와
//! 옛한글 `ᄒᆞᆫ`(초성·중성·종성 자모 셋)이 있다.
//!
//! 기대값은 같은 문서를 한/글 2020(MCP engine 2020)으로 출력한
//! `pdf/issue7702/tab_stop_old_hangul_line_break-2020.pdf` 다.
//! 첫 줄은 `…컴퓨터를 이용한 프레젠테이션` 으로 끝나고 둘째 줄은 `을 할 수` 로 시작한다.
//! 탭 뒤 `ᄒᆞᆫ` 다음 글자 `글` 의 x 는 169.70px(96dpi)다.
//!
//! 수정 전 rhwp 의 줄 나눔은 두 가지에서 그리기와 달랐다.
//! - 탭을 줄 시작부터 기본 간격으로 쟀다: 47.9px. 그리기는 탭 정지까지 31.7px 를 전진한다.
//! - `ᄒᆞᆫ` 의 자모 셋을 각각 한 글자 폭으로 쟀다. 그리기는 한 글자로 그린다.
//!
//! 그래서 첫 줄을 `프레젠` 에서 끊었다(탭만 고치면 `프레젠테`). 둘 다 고쳐야 한/글과 같다.

use rhwp::document_core::DocumentCore;

const TOLERANCE_PX: f64 = 0.5;

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

/// 첫 쪽 글자 줄들(기준선 y 순, 줄마다 x 순 글자들).
fn glyph_lines(core: &DocumentCore) -> Vec<Vec<(f64, String)>> {
    let svg = core.render_page_svg_native(0).expect("SVG");
    let mut lines: Vec<(f64, Vec<(f64, String)>)> = Vec::new();
    let mut rest = svg.as_str();
    while let Some(open) = rest.find("<text ") {
        rest = &rest[open..];
        let (Some(tag_end), Some(close)) = (rest.find('>'), rest.find("</text>")) else {
            break;
        };
        let text = rest[tag_end + 1..close].trim().to_string();
        if let (Some((x, y)), false) = (origin(&rest[..tag_end]), text.is_empty()) {
            let y = (y * 10.0).round() / 10.0;
            match lines.iter_mut().find(|(line_y, _)| *line_y == y) {
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
fn tab_stop_and_old_hangul_cluster_break_first_line_like_hangul() {
    let bytes = include_bytes!("../../samples/issue7702/tab_stop_old_hangul_line_break.hwpx");
    let core = DocumentCore::from_bytes(bytes).expect("문서 열기");
    let lines = glyph_lines(&core);
    let text = |line: &[(f64, String)]| line.iter().map(|(_, t)| t.as_str()).collect::<String>();
    let first = lines
        .iter()
        .position(|line| text(line).contains("문서의배경에"))
        .unwrap_or_else(|| {
            panic!(
                "첫 줄을 찾지 못했다: {:?}",
                lines.iter().map(|l| text(l)).collect::<Vec<_>>()
            )
        });
    let line1 = text(&lines[first]);
    let line2 = lines.get(first + 1).map(|l| text(l)).unwrap_or_default();

    assert!(
        line1.ends_with("이용한프레젠테이션") && line2.starts_with("을할수"),
        "한/글 첫 줄은 `…이용한프레젠테이션`, 둘째 줄은 `을할수…`: rhwp 첫 줄 {line1:?} / 둘째 줄 {line2:?}"
    );

    // 대조군: 그리기의 탭·자모 묶음 폭은 수정 전에도 한/글과 같았다.
    let glyph = lines[first]
        .iter()
        .find(|(_, t)| t == "글")
        .expect("`글` 글자");
    assert!(
        (glyph.0 - 169.70).abs() <= TOLERANCE_PX,
        "`글` x: rhwp {:.2} / 한/글 169.70",
        glyph.0
    );
}
