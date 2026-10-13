//! [#7702] 양쪽 정렬 줄 끝에 걸린 공백은 배분 자리가 아니다 — 저장 줄 없는 재조판 줄도 같다.
//!
//! 입력은 #7816 의 합성 HWPX `samples/issue7816/odd_indent_ladder.hwpx`(저장 줄 없음, 바탕 10pt,
//! 1쪽 문단 전부 양쪽 정렬)다. 기대값은 같은 문서의 한/글 2020 출력
//! `pdf/issue7816/odd_indent_ladder-2020.pdf` 1쪽에서 잰, 문단 마지막 줄이 아닌 줄(양쪽 정렬로
//! 공백이 늘어나는 줄)의 **마지막 가시 글자 원점 x**(px, 96dpi)다.
//!
//! 한/글은 줄 끝에 걸린 공백을 오른쪽 여백 바깥에 두고, 그 앞 글자가 본문 오른쪽 끝에서 끝나도록
//! 내부 공백만 늘린다. 수정 전 rhwp 는 저장 줄 없는 줄에서 이 공백까지 분모(내부 공백 + 1)와 사용
//! 폭에 넣어, 마지막 글자가 6~16px 왼쪽에 그려졌다(Visual Sweep p1 87.46%).
//!
//! 판정은 줄 마지막 글자 x 의 차이다. 허용 1.0px 은 한/글 PDF 글자 원점의 장치 격자 흔들림
//! (±0.16px)과 공백 배분 끝수(한 줄 최대 0.7px)를 합친 값이고, 결함 크기(6px 이상)와 겹치지 않는다.

use rhwp::document_core::DocumentCore;

const TOLERANCE_PX: f64 = 1.0;

/// (줄 끝 세 글자, 한/글 마지막 글자 x px) — 1쪽 양쪽 정렬 배분 줄, 위에서 아래 순.
const HANGUL_JUSTIFIED_LINE_ENDS: &[(&str, f64)] = &[
    ("저장된", 685.55), // 한글문서의문
    ("게적어", 685.55), // 줄정보가없는
    ("저장된", 685.55), // 한글문서의문
    ("게적어", 685.55), // 줄정보가없는
    ("저장된", 685.55), // 한글문서의문
    ("게적어", 685.55), // 줄정보가없는
    ("꾸면서", 685.55), // 한글문서의문
    ("히길게", 685.39), // 저장된줄정보
    ("꾸면서", 685.55), // 한글문서의문
    ("히길게", 685.39), // 저장된줄정보
    ("꾸면서", 685.55), // 한글문서의문
    ("히길게", 685.39), // 저장된줄정보
    ("과함께", 685.55), // 한글문서의문
    ("문장을", 685.55), // 바꾸면서저장
    ("방식과", 685.55), // 한글문서의문
    ("려고이", 685.55), // 함께바꾸면서
    ("있는데", 685.55), // 한글문서의문
    ("누는지", 685.55), // 이값을정렬방
    ("는첫줄", 685.55), // 한글문서의문
    ("한글이", 685.55), // 들여쓰기가있
    ("저장된", 685.55), // 한글문서의문
    ("게적어", 685.55), // 줄정보가없는
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

/// 1쪽 글자 줄: x 순 (x, 글자) 목록, 위에서 아래 순.
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
fn justified_line_hangs_its_end_space_outside_the_distribution() {
    let bytes = std::fs::read("samples/issue7816/odd_indent_ladder.hwpx").expect("고정본");
    let core = DocumentCore::from_bytes(&bytes).expect("파싱");

    // 문단 마지막 줄(마침표로 끝남)과 한 줄 문단(`가나다라마바사아`)은 배분하지 않는 줄이다.
    let distributed: Vec<(String, f64)> = page_glyph_lines(&core, 0)
        .into_iter()
        .filter_map(|line| {
            let text: String = line.iter().map(|(_, t)| t.as_str()).collect();
            if text == "가나다라마바사아" || text.ends_with('.') {
                return None;
            }
            let tail: String = {
                let chars: Vec<char> = text.chars().collect();
                chars[chars.len().saturating_sub(3)..].iter().collect()
            };
            Some((tail, line.last().expect("글자").0))
        })
        .collect();

    assert_eq!(
        distributed.len(),
        HANGUL_JUSTIFIED_LINE_ENDS.len(),
        "1쪽 배분 줄 수가 한/글과 달라 줄 나눔부터 어긋났다: {distributed:?}"
    );
    let mut failures = Vec::new();
    for (i, ((tail, x), (hangul_tail, hangul_x))) in distributed
        .iter()
        .zip(HANGUL_JUSTIFIED_LINE_ENDS)
        .enumerate()
    {
        assert_eq!(
            tail, hangul_tail,
            "{i}번째 배분 줄의 끝 글자가 한/글과 다르다"
        );
        if (x - hangul_x).abs() > TOLERANCE_PX {
            failures.push(format!(
                "{i}번째 줄 «…{tail}» 마지막 글자 x rhwp {x:.2} · 한/글 {hangul_x:.2} (차 {:+.2})",
                x - hangul_x
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "양쪽 정렬 줄 마지막 글자가 본문 오른쪽 끝에 닿지 않는다 — 줄 끝에 걸린 공백이 배분 \
         몫을 가져갔다:\n{}",
        failures.join("\n")
    );
}
