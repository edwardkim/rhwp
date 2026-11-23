//! 관계형. 도형 개체 속성의 선(투명도·화살표 채움)·호 테두리·가로 정렬과 채우기 투명도를 검사한다.
//! 선 투명도(HWP 인스턴스 ID 다음 바이트, HWPX `lineShape@alpha`)와 HWPX 채우기 투명도는 한/글
//! 12.30처럼 0~255 정수다. 한/글에서 선 투명도 50%·채우기 30%를 준 사각형은 HWPX에
//! `lineShape alpha="128"`·`winBrush alpha="77"`로, HWP에 같은 바이트로 저장된다.

use rhwp::document_core::DocumentCore;

/// 빈 문서 첫 문단의 첫 도형(구역·단 정의 다음) 컨트롤 번호.
fn shape_control(core: &DocumentCore) -> usize {
    core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|control| matches!(control, rhwp::model::control::Control::Shape(_)))
        .expect("도형")
}

fn shape_props(core: &DocumentCore) -> serde_json::Value {
    let control = shape_control(core);
    serde_json::from_str(
        &core
            .get_shape_properties_native(0, 0, control)
            .expect("도형 속성"),
    )
    .expect("JSON")
}

fn rectangle_with_alpha() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("빈 문서");
    core.create_shape_control_native(
        0,
        0,
        0,
        9000,
        6000,
        20000,
        20000,
        false,
        "InFrontOfText",
        "rectangle",
        false,
        false,
        &[],
    )
    .expect("사각형");
    let control = shape_control(&core);
    core.set_shape_properties_native(
        0,
        0,
        control,
        r#"{"lineAlpha":128,"fillType":"solid","fillBgColor":255,"fillAlpha":77}"#,
    )
    .expect("투명도");
    core
}

#[test]
fn line_and_fill_alpha_survive_hwp_and_hwpx() {
    let core = rectangle_with_alpha();
    assert_eq!(shape_props(&core)["lineAlpha"], 128);
    for bytes in [
        core.export_hwp_native().expect("HWP"),
        core.export_hwpx_native().expect("HWPX"),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).expect("다시 열기");
        let props = shape_props(&reopened);
        assert_eq!(props["lineAlpha"], 128);
        assert_eq!(props["fillAlpha"], 77);
    }
}

#[test]
fn hwpx_writes_alpha_as_hancom_integers() {
    let bytes = rectangle_with_alpha().export_hwpx_native().expect("HWPX");
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip");
    let mut xml = String::new();
    std::io::Read::read_to_string(
        &mut archive.by_name("Contents/section0.xml").expect("section0"),
        &mut xml,
    )
    .expect("읽기");
    assert!(
        xml.contains(r#"outlineStyle="NORMAL" alpha="128""#),
        "{xml}"
    );
    assert!(xml.contains(r#"alpha="77"/></hc:fillBrush>"#), "{xml}");
}

#[test]
fn hancom_integer_alpha_is_a_byte() {
    use rhwp::parser::hwpx::utils::parse_alpha_byte;
    assert_eq!(parse_alpha_byte("77"), Some(77));
    assert_eq!(parse_alpha_byte("128"), Some(128));
    assert_eq!(parse_alpha_byte("0"), Some(0));
    // 예전 rhwp 저장본의 비율 값
    assert_eq!(parse_alpha_byte("0.30196078431372547"), Some(77));
    assert_eq!(parse_alpha_byte("x"), None);
}

#[test]
fn line_alpha_draws_only_the_stroke_transparent() {
    let core = rectangle_with_alpha();
    let svg = core.render_page_svg_native(0).expect("SVG");
    let rect = svg
        .lines()
        .find(|line| line.starts_with("<rect") && line.contains("#ff0000"))
        .expect("사각형");
    assert!(rect.contains(r#"stroke-opacity="0.498""#), "{rect}");
    assert!(rect.contains(r#"fill-opacity="0.698""#), "{rect}");
    assert!(!rect.contains(" opacity="), "{rect}");
}

fn shape_core(kind: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("빈 문서");
    core.create_shape_control_native(
        0,
        0,
        0,
        9000,
        6000,
        20000,
        20000,
        false,
        "InFrontOfText",
        kind,
        false,
        false,
        &[],
    )
    .expect("도형");
    core
}

fn set(core: &mut DocumentCore, json: &str) {
    let control = shape_control(core);
    core.set_shape_properties_native(0, 0, control, json)
        .expect("속성");
}

/// 한컴 12.30 HWP는 시작 화살표 채움을 bit 30, 끝을 bit 31에 두고, HWPX는 headfill·tailfill로 쓴다.
/// 채운 마름모 시작·빈 원 끝 직선은 HWP 0x52051041, HWPX headfill="1" tailfill="0"이다.
#[test]
fn arrow_fill_keeps_start_and_end_through_hwp_and_hwpx() {
    let mut core = shape_core("line");
    set(
        &mut core,
        r#"{"arrowStart":4,"arrowEnd":5,"arrowStartFill":true,"arrowEndFill":false}"#,
    );
    let props = shape_props(&core);
    assert_eq!(props["arrowStartFill"], true);
    assert_eq!(props["arrowEndFill"], false);
    assert_eq!(
        props["borderAttr"].as_u64().unwrap() & 0xC000_0000,
        0x4000_0000
    );
    for bytes in [
        core.export_hwp_native().expect("HWP"),
        core.export_hwpx_native().expect("HWPX"),
    ] {
        let props = shape_props(&DocumentCore::from_bytes(&bytes).expect("다시 열기"));
        assert_eq!(
            (&props["arrowStart"], &props["arrowEnd"]),
            (&serde_json::json!(4), &serde_json::json!(5))
        );
        assert_eq!(props["arrowStartFill"], true);
        assert_eq!(props["arrowEndFill"], false);
    }
}

/// 한컴 개체 속성 > 호 테두리의 부채꼴은 HWPX `<hp:arc type="PIE">`다.
#[test]
fn arc_border_kind_is_read_and_set() {
    let mut core = shape_core("arc");
    assert_eq!(shape_props(&core)["arcType"], 0);
    set(&mut core, r#"{"arcType":1}"#);
    assert_eq!(shape_props(&core)["arcType"], 1);
    let hwpx = core.export_hwpx_native().expect("HWPX");
    for bytes in [core.export_hwp_native().expect("HWP"), hwpx] {
        let props = shape_props(&DocumentCore::from_bytes(&bytes).expect("다시 열기"));
        assert_eq!(props["arcType"], 1);
    }
}

/// 한/글 개체 속성 > 배치의 가로 정렬 안쪽·바깥쪽을 도형·그림 속성으로 바꾸고 HWP·HWPX에 남긴다.
#[test]
fn horizontal_inside_and_outside_alignment_is_set() {
    let mut core = shape_core("rectangle");
    set(&mut core, r#"{"horzRelTo":"Page","horzAlign":"Outside"}"#);
    assert_eq!(shape_props(&core)["horzAlign"], "Outside");
    for bytes in [
        core.export_hwp_native().expect("HWP"),
        core.export_hwpx_native().expect("HWPX"),
    ] {
        let props = shape_props(&DocumentCore::from_bytes(&bytes).expect("다시 열기"));
        assert_eq!(props["horzAlign"], "Outside");
    }
    set(&mut core, r#"{"horzAlign":"Inside"}"#);
    assert_eq!(shape_props(&core)["horzAlign"], "Inside");
}

/// 한/글 12.30은 채운 마름모 시작 화살표를 `headStyle="EMPTY_DIAMOND" headfill="1"`로 저장하고
/// `FILLED_DIAMOND`는 읽지 못해 화살표를 그리지 않는다.
#[test]
fn hwpx_writes_filled_arrow_as_hancom_does() {
    let mut core = shape_core("line");
    set(
        &mut core,
        r#"{"arrowStart":4,"arrowStartFill":true,"arrowEnd":5,"arrowEndFill":false}"#,
    );
    let bytes = core.export_hwpx_native().expect("HWPX");
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip");
    let mut xml = String::new();
    std::io::Read::read_to_string(
        &mut archive.by_name("Contents/section0.xml").expect("section0"),
        &mut xml,
    )
    .expect("읽기");
    assert!(
        xml.contains(
            r#"headStyle="EMPTY_DIAMOND" tailStyle="EMPTY_CIRCLE" headfill="1" tailfill="0""#
        ),
        "{xml}"
    );
    assert!(!xml.contains("FILLED_"), "{xml}");
}

fn zip_entry(bytes: &[u8], name: &str) -> Option<String> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip");
    let mut text = String::new();
    std::io::Read::read_to_string(&mut archive.by_name(name).ok()?, &mut text).expect("읽기");
    Some(text)
}

/// `skip` 엔트리를 뺀 같은 패키지. 한/글은 다시 저장할 때 모르는 META-INF 엔트리를 버린다.
fn zip_without(bytes: &[u8], skip: &str) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip");
    let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let file = archive.by_index_raw(index).expect("엔트리");
        if file.name().map_or(true, |name| name != skip) {
            out.raw_copy_file(file).expect("복사");
        }
    }
    out.finish().expect("zip 닫기").into_inner()
}

/// 예전 rhwp(`devel` ca231d64f)가 채우기·선 투명도 255 사각형 HWP를 `export-hwpx`로 저장한 파일.
/// 그 저장기는 alpha 를 0.0~1.0 비율로 써서 완전 투명 채우기가 `winBrush alpha="1"`이 됐고,
/// 선 투명도는 늘 `alpha="0"`으로 썼다. HWP 원본 표지 `META-INF/rhwp-hwp5-origin`이 있다.
const OLD_RHWP_TRANSPARENT_HWPX: &[u8] =
    include_bytes!("../fixtures/shape_line_fill_alpha/old_rhwp_transparent_fill.hwpx");

#[test]
fn old_rhwp_hwpx_keeps_its_transparent_fill() {
    let section = zip_entry(OLD_RHWP_TRANSPARENT_HWPX, "Contents/section0.xml").expect("구역");
    assert!(
        section.contains(r#"alpha="1"/></hc:fillBrush>"#),
        "{section}"
    );
    assert!(zip_entry(OLD_RHWP_TRANSPARENT_HWPX, "META-INF/rhwp-hwp5-origin").is_some());

    let reopened = DocumentCore::from_bytes(OLD_RHWP_TRANSPARENT_HWPX).expect("열기");
    assert_eq!(shape_props(&reopened)["fillAlpha"], 255);
    // 예전 저장기는 선 투명도를 남기지 않았다 — 저장된 불투명 선을 그대로 그린다.
    assert_eq!(shape_props(&reopened)["lineAlpha"], 0);

    // 새 rhwp로 다시 저장하면 정수 255와 정수 alpha 표지를 쓰고, 다시 열어도 투명하다.
    let resaved = reopened.export_hwpx_native().expect("HWPX");
    assert!(zip_entry(&resaved, "META-INF/rhwp-byte-alpha").is_some());
    let section = zip_entry(&resaved, "Contents/section0.xml").expect("구역");
    assert!(
        section.contains(r#"alpha="255"/></hc:fillBrush>"#),
        "{section}"
    );
    let again = DocumentCore::from_bytes(&resaved).expect("다시 열기");
    assert_eq!(shape_props(&again)["fillAlpha"], 255);
}

/// rhwp 원본 표지가 없는 패키지(한/글이 다시 저장한 파일)의 정수 `alpha="1"`은 한/글처럼
/// 거의 불투명한 바이트 1이다.
#[test]
fn hancom_alpha_one_without_rhwp_marker_stays_nearly_opaque() {
    let hancom = zip_without(OLD_RHWP_TRANSPARENT_HWPX, "META-INF/rhwp-hwp5-origin");
    let core = DocumentCore::from_bytes(&hancom).expect("열기");
    assert_eq!(shape_props(&core)["fillAlpha"], 1);
}

/// 새 rhwp가 HWP에서 만든 HWPX는 정수 alpha 표지를 싣고, 바이트 1을 바이트 1로 다시 읽는다.
#[test]
fn new_rhwp_hwpx_from_hwp_reads_alpha_one_as_a_byte() {
    let mut core = rectangle_with_alpha();
    set(&mut core, r#"{"fillAlpha":1}"#);
    let hwp = core.export_hwp_native().expect("HWP");
    let from_hwp = DocumentCore::from_bytes(&hwp).expect("HWP 열기");
    let hwpx = from_hwp.export_hwpx_native().expect("HWPX");
    assert!(zip_entry(&hwpx, "META-INF/rhwp-hwp5-origin").is_some());
    assert!(zip_entry(&hwpx, "META-INF/rhwp-byte-alpha").is_some());
    let reopened = DocumentCore::from_bytes(&hwpx).expect("HWPX 열기");
    assert_eq!(shape_props(&reopened)["fillAlpha"], 1);
}

/// 그려진 타원의 위끝(px).
fn ellipse_top(core: &DocumentCore) -> f64 {
    let svg = core.render_page_svg_native(0).expect("SVG");
    let ellipse = svg
        .lines()
        .find(|line| line.starts_with("<ellipse"))
        .expect("타원");
    let attr = |name: &str| -> f64 {
        let start = ellipse.find(&format!(" {name}=\"")).expect(name) + name.len() + 3;
        ellipse[start..start + ellipse[start..].find('"').unwrap()]
            .parse()
            .unwrap()
    };
    attr("cy") - attr("ry")
}

/// 한/글 12.30 Mac에서 #8324 타원(문단 기준·위 정렬, 세로 -876)은 '쪽 영역 안으로 제한'이면
/// 글 앞·글 뒤 모두 문단 위끝에 그려지고, 제한을 끄면 876 HU(11.68px) 위에 그려진다.
#[test]
fn restricted_front_and_behind_shape_ignores_negative_para_offset() {
    for wrap in ["InFrontOfText", "BehindText"] {
        let top = |restrict: bool, offset: i32| {
            let mut core = shape_core("ellipse");
            set(
                &mut core,
                &format!(
                    r#"{{"textWrap":"{wrap}","vertRelTo":"Para","vertAlign":"Top","restrictInPage":{restrict},"vertOffset":{}}}"#,
                    offset as u32
                ),
            );
            ellipse_top(&core)
        };
        let anchor = top(true, 0);
        assert!((top(true, -876) - anchor).abs() < 0.01, "{wrap}");
        assert!((top(false, -876) - (anchor - 11.68)).abs() < 0.01, "{wrap}");
    }
}
