//! #7881 변환 원본의 현재 본문 프레임·문단 간격·문자 출처를 보호한다.
//! 같은 원본/독립 Print 전46쪽에서 Native94.65583/WASM94.55536%를 확인했다.
//! Print에서 확인한 a로케트·Pro Medium의 Haansoft Batang 대체 환경을 명시했다.
//! 12·26쪽은 양쪽98% 이상, 19쪽은 양쪽100%이며 review/overlay를 직접 판독했다.
//! 선의 px 기대값은 PDF19쪽 72.281pt를 96CSSpx로 환산한 독립 좌표다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::document::Document;
use serde_json::Value;
use std::{fs, path::PathBuf, process::Command, sync::OnceLock};

const INPUT: &str = "samples/hwpx/hancom-hwp/[2027] 온새미로 1 본교재.hwp";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn binary() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_rhwp")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_rhwp")))
}
fn output(label: &str) -> PathBuf {
    let path = root().join("output").join(format!(
        "issue7881-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
fn source() -> &'static Document {
    static DOC: OnceLock<Document> = OnceLock::new();
    DOC.get_or_init(|| rhwp::parser::parse_hwp(&fs::read(root().join(INPUT)).unwrap()).unwrap())
}
fn text(node: &Value) -> String {
    let mut result = node["text"].as_str().unwrap_or_default().to_owned();
    if let Some(children) = node["children"].as_array() {
        for child in children {
            result.push_str(&text(child));
        }
    }
    result
}
fn body_lines<'a>(node: &'a Value, pi: u64, result: &mut Vec<&'a Value>) {
    if node["type"] == "MasterPage" {
        return;
    }
    if node["type"] == "TextLine" && node["pi"] == pi {
        result.push(node);
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            body_lines(child, pi, result);
        }
    }
}
fn page(number: usize) -> Value {
    let dir = output("tree");
    let result = Command::new(binary())
        .arg("export-render-tree")
        .arg(root().join(INPUT))
        .args([
            "--compat",
            "2022",
            "--page",
            &(number - 1).to_string(),
            "-o",
        ])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "실행 실패: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let files: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|x| x.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    assert_eq!(files.len(), 1);
    let value = serde_json::from_slice(&fs::read(&files[0]).unwrap()).unwrap();
    fs::remove_dir_all(dir).unwrap();
    value
}
fn y(node: &Value) -> f64 {
    node["bbox"]["y"].as_f64().unwrap()
}
fn height(node: &Value) -> f64 {
    node["bbox"]["h"].as_f64().unwrap()
}
fn compact(value: &str) -> String {
    value.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn current_single_row_and_following_origin_share_the_same_frame() {
    let tree = page(12);
    let doc = source();
    for pi in [143usize, 148] {
        let original = &doc.sections[1].paragraphs[pi];
        assert_eq!(
            original.line_segs.len(),
            2,
            "입력은 예전 두 줄 캐시를 보존한다"
        );
        let mut rows = Vec::new();
        body_lines(&tree, pi as u64, &mut rows);
        assert_eq!(
            rows.len(),
            1,
            "현재 폭에 들어가는 한 줄을 예전 두 줄로 점유하면 안 된다"
        );
        assert_eq!(
            compact(&text(rows[0])),
            compact(&original.text),
            "문장 전체의 소속과 순서를 보존한다"
        );
        let mut next = Vec::new();
        body_lines(&tree, (pi + 1) as u64, &mut next);
        assert_eq!(next.len(), 1);
        assert_eq!(
            compact(&text(next[0])),
            compact(&doc.sections[1].paragraphs[pi + 1].text)
        );
        let advance =
            f64::from(original.line_segs[0].line_height + original.line_segs[0].line_spacing)
                / 75.0;
        assert!(
            (y(next[0]) - y(rows[0]) - advance).abs() <= 2.0,
            "다음 원점은 현재 한 줄의 원본 높이·줄간격을 소비한다"
        );
    }
}

#[test]
fn inline_picture_row_keeps_the_next_paragraphs_declared_band() {
    let tree = page(26);
    let doc = source();
    let prev = &doc.sections[2].paragraphs[124];
    let next = &doc.sections[2].paragraphs[125];
    assert!(prev
        .controls
        .iter()
        .any(|c| matches!(c,rhwp::model::control::Control::Picture(p) if p.common.treat_as_char)));
    let mut a = Vec::new();
    let mut b = Vec::new();
    body_lines(&tree, 124, &mut a);
    body_lines(&tree, 125, &mut b);
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
    assert!(text(a[0]).contains("이번 주 목표"));
    let before =
        f64::from(doc.doc_info.para_shapes[next.para_shape_id as usize].spacing_before) / 2.0;
    let after =
        f64::from(doc.doc_info.para_shapes[prev.para_shape_id as usize].spacing_after) / 2.0;
    let advance = (f64::from(prev.line_segs[0].line_height + prev.line_segs[0].line_spacing)
        + before
        + after)
        / 75.0;
    assert!(before > 0.0);
    assert!(
        (y(b[0]) - y(a[0]) - advance).abs() <= 2.0,
        "줄 안 그림 높이가 문단의 앞 간격을 없애면 안 된다"
    );
    assert!(
        y(b[0]) >= y(a[0]) + height(a[0]) + before / 75.0 - 2.0,
        "점유 줄 상자와 다음 간격을 함께 보존한다"
    );
}

#[test]
fn marker_row_owns_its_declared_font_and_following_advance() {
    let tree = page(11);
    let doc = source();
    let paragraph = &doc.sections[1].paragraphs[115];
    let shape = &doc.doc_info.para_shapes[paragraph.para_shape_id as usize];
    let numbering = &doc.doc_info.numberings[shape.numbering_id as usize - 1];
    let head = &numbering.heads[shape.para_level as usize];
    let marker_size_hu = doc.doc_info.char_shapes[head.char_shape_id as usize].base_size;
    assert!(marker_size_hu > paragraph.line_segs[0].line_height);
    let mut lines = Vec::new();
    body_lines(&tree, 115, &mut lines);
    assert_eq!(lines.len(), 1);
    let line = lines[0];
    assert!(text(line).starts_with("01"));
    assert!(text(line).contains("보기 어려운 것은?"));
    let marker_size = f64::from(marker_size_hu) / 75.0;
    assert!(
        height(line) + 0.1 >= marker_size,
        "원본 번호 글자 모양의 선언 크기를 줄 상자가 점유한다"
    );
    let mut following = Vec::new();
    body_lines(&tree, 116, &mut following);
    assert_eq!(following.len(), 1);
    let next = &doc.sections[1].paragraphs[116];
    let next_shape = &doc.doc_info.para_shapes[next.para_shape_id as usize];
    assert_eq!(
        shape.line_spacing_type,
        rhwp::model::style::LineSpacingType::Percent
    );
    let advance = marker_size * f64::from(shape.line_spacing) / 100.0
        + f64::from(shape.spacing_after + next_shape.spacing_before) / 150.0;
    assert!(
        (y(following[0]) - y(line) - advance).abs() <= 2.0,
        "다음 원점도 번호의 실제 줄 높이와 원본 문단 간격을 소비한다"
    );
}

#[test]
fn save_keeps_literal_figure_spaces_distinct_from_fixed_blank_elements() {
    let dir = output("literal");
    let file = dir.join("copy.hwpx");
    let result = Command::new(binary())
        .arg("export-hwpx")
        .arg(root().join(INPUT))
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "저장 실패: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let actual = rhwp::parser::hwpx::parse_hwpx(&fs::read(&file).unwrap()).unwrap();
    let footer = |doc: &Document| {
        let shape = match &doc.sections[0].section_def.master_pages[1].paragraphs[0].controls[0] {
            rhwp::model::control::Control::Shape(shape) => shape,
            _ => panic!("원본 꼬리말 글상자"),
        };
        shape
            .drawing()
            .unwrap()
            .text_box
            .as_ref()
            .unwrap()
            .paragraphs[0]
            .clone()
    };
    let a = footer(source());
    let b = footer(&actual);
    assert_eq!(a.text, b.text);
    let expected: Vec<_> = a
        .text
        .chars()
        .enumerate()
        .filter_map(|(i, c)| (c == '\u{2007}').then_some(i))
        .collect();
    assert_eq!(expected.len(), 2);
    assert_eq!(
        b.literal_figure_spaces, expected,
        "원본 Unicode 리터럴을 fwSpace 요소로 강등하면 안 된다"
    );
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(feature = "native-skia")]
#[test]
fn print_hairline_survives_at_the_independent_pdf_pixel_position() {
    // 독립 PDF19쪽의 72.281pt 가로 괘선. Native/WASM 양쪽100%·직접 판독 완료.
    let dir = output("hairline");
    let result = Command::new(binary())
        .arg("export-png")
        .arg(root().join(INPUT))
        .args([
            "--compat",
            "2022",
            "--profile",
            "print",
            "--dpi",
            "96",
            "--page",
            "18",
            "-o",
        ])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "PNG 출력 실패: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let files: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "png"))
        .collect();
    assert_eq!(files.len(), 1);
    let image = image::load_from_memory(&fs::read(&files[0]).unwrap())
        .unwrap()
        .to_rgb8();
    let center = (72.281_f64 * 96.0 / 72.0).round() as u32;
    for x in [100, 500] {
        assert!(
            (center - 2..=center + 2).any(|y| image.get_pixel(x, y).0.iter().all(|v| *v <= 225)),
            "PDF 괘선의 ±2CSSpx 안에서 연한 단일 실선의 실제 잉크를 보존한다"
        );
    }
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn literal_space_origin_survives_scalar_edits_split_and_merge() {
    use rhwp::model::paragraph::Paragraph;
    // 첫 U+2007은 리터럴, 둘째는 HWP 고정폭 빈칸인 혼용 원문이다.
    let mut paragraph = Paragraph {
        text: "A\u{2007}B\u{2007}C".into(),
        char_offsets: vec![0, 1, 2, 3, 4],
        char_count: 6,
        control_mask: 1u32 << 31,
        literal_figure_spaces: vec![1],
        ..Default::default()
    };
    paragraph.insert_text_at(0, "🙂");
    assert_eq!(paragraph.literal_figure_spaces, vec![2]);
    paragraph.insert_text_at(3, "\u{2007}");
    assert_eq!(paragraph.literal_figure_spaces, vec![2, 3]);
    paragraph.delete_text_at(0, 1);
    assert_eq!(paragraph.literal_figure_spaces, vec![1, 2]);
    let right = paragraph.split_at(2);
    assert_eq!(paragraph.literal_figure_spaces, vec![1]);
    assert_eq!(right.literal_figure_spaces, vec![0]);
    paragraph.merge_from(&right);
    assert_eq!(paragraph.literal_figure_spaces, vec![1, 2]);
    paragraph.delete_text_at(1, 1);
    assert_eq!(paragraph.literal_figure_spaces, vec![1]);
    assert_eq!(paragraph.text, "A\u{2007}B\u{2007}C");
}
