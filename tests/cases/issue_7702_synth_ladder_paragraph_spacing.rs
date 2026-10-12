//! [#7702] 저장 줄이 없는 문단을 합성한 쪽의 세로 위치 — 문단 간격·쪽 맨 위 앞 간격·기준 역산.
//!
//! 입력 `samples/HWP5-nopassword-123456.hwpx` 는 한/글 2024 저장본이다. 일부 문단은 저장 줄이
//! 없어 rhwp 가 로드 때 줄을 합성한다(`TAG_IMPLEMENTATION_PROPERTY`). 기준은 같은 파일의 한/글
//! 2024 PDF `pdf/HWP5-nopassword-123456-hwpx-2024.pdf` 의 글자 기준선(pt × 96/72)이다.
//!
//! 1. 로드 재구성 사다리가 문단 간격을 빠뜨렸다. 그리기가 그 사다리로 스냅해 12쪽 `[HNC Input]`
//!    이 앞 문단 아래 간격(7.6px)만큼 위에 놓였다.
//! 2. 쪽 첫 문단이 합성 줄이면 그리기가 쪽 맨 위의 앞 간격을 버렸다. 한/글은 그 간격을 둔다
//!    (20쪽 제목 `나. 팩스`, 앞 간격 3.8px, `pageBreak="1"` 문단). 쪽 기준점도 그 간격을 빼야
//!    뒤 저장 문단(`•팩스를 송신할 때…`)의 스냅이 간격만큼 되감지 않는다.
//! 3. 기준 리셋 뒤 lazy 기준의 역산식은 트림한 조판 흐름을 전제해, 간격이 흐름에 든 그리기에서는
//!    다음 문단을 끝 줄간격 + 아래 간격(1168 HU = 15.6px)만큼 내렸다(3쪽 표 뒤). 그리기는 앞
//!    문단 마지막 줄의 실제 바닥으로 기준을 맞춘다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

const TOLERANCE_PX: f64 = 0.5;

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/HWP5-nopassword-123456.hwpx");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    let end = start + tag[start..].find('"')?;
    Some(&tag[start..end])
}

/// 글자 `<text>` 의 원점. 장평이 100% 가 아니면 `transform="translate(x,y) scale(..)"` 로 쓴다.
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

/// 쪽(1부터)의 글자 기준선마다 공백 뺀 글을 모은다.
fn baselines(core: &DocumentCore, page: u32) -> Vec<(f64, String)> {
    let svg = core.render_page_svg_native(page - 1).expect("SVG");
    let mut lines: Vec<(f64, Vec<(f64, String)>)> = Vec::new();
    let mut rest = svg.as_str();
    while let Some(open) = rest.find("<text ") {
        rest = &rest[open..];
        let (Some(tag_end), Some(close)) = (rest.find('>'), rest.find("</text>")) else {
            break;
        };
        if let Some((x, y)) = origin(&rest[..tag_end]) {
            let y = (y * 10.0).round() / 10.0;
            let text = rest[tag_end + 1..close].to_string();
            match lines.iter_mut().find(|(line_y, _)| *line_y == y) {
                Some((_, line)) => line.push((x, text)),
                None => lines.push((y, vec![(x, text)])),
            }
        }
        rest = &rest[close..];
    }
    lines
        .into_iter()
        .map(|(y, mut glyphs)| {
            glyphs.sort_by(|a, b| a.0.total_cmp(&b.0));
            let text: String = glyphs.into_iter().map(|(_, t)| t).collect();
            (y, text.chars().filter(|c| !c.is_whitespace()).collect())
        })
        .collect()
}

fn baseline_of(core: &DocumentCore, page: u32, head: &str) -> f64 {
    let lines = baselines(core, page);
    lines
        .iter()
        .find(|(_, text)| text.starts_with(head))
        .map(|(y, _)| *y)
        .unwrap_or_else(|| panic!("{page}쪽 `{head}` 줄이 없다: {lines:?}"))
}

#[test]
fn lines_sit_on_hangul_baselines() {
    let core = core();
    // (쪽, 줄 머리, 한/글 2024 PDF 기준선 px)
    let cases: [(u32, &str, f64); 4] = [
        (12, "[HNCInput]", 236.64),
        (20, "입니다.PC환경에서", 200.0),
        (20, "•팩스를송신할때", 536.6),
        (3, "&lt;하드디스크에설치한", 512.96),
    ];
    let mut mismatched = Vec::new();
    for (page, head, hangul) in cases {
        let y = baseline_of(&core, page, head);
        if (y - hangul).abs() > TOLERANCE_PX {
            mismatched.push(format!(
                "{page}쪽 `{head}`: 기준선 {y:.2}px, 한/글 {hangul:.2}px"
            ));
        }
    }
    assert!(
        mismatched.is_empty(),
        "줄 위치가 한/글과 다르다:\n{}",
        mismatched.join("\n")
    );
}

#[test]
fn page_count_stays_hangul() {
    assert_eq!(core().page_count(), 24, "정본 24쪽");
}
