//! 관계형: 설치되지 않은 TTF 선언 글꼴과 한컴 기본 대체(한컴바탕)의 글꼴 소유 관계,
//! HFT 기호 `∙` 의 전진과 뒤 본문 원점 사이 간격.
//! 주석 갱신: 2026-11-19T19:41:38+09:00 (새 파일).
//! 측정일 2026-11-19, 제품 코드 upstream/devel `36ae26d6f6` + 이 변경.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only --compat 2022`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣,
//! `threshold_boundary_color_support_v1`, PDF raster pdfium), `RHWP_FONT_PATH` 에 생성 PC 글꼴 모음.
//! 입력 `samples/issue547/online-search-rfp.hwp`;
//! 독립 Print `pdf/issue547/online-search-rfp-2022.pdf`(한컴 2022 12.0.0.4547).
//! 전체 22쪽(독립 Print 22쪽). 전쪽 최저 Native 95.00%(21쪽) / fresh WASM 98.55%(21쪽).
//! 주장 쪽 raw: 4쪽 Native 99.88% / WASM 99.93%, 5쪽 99.80% / 99.80%,
//! 14쪽 99.93% / 99.94%, 15쪽 99.91% / 99.94%.
//! 수정 전 devel `fbf8081db5` Native: 4쪽 85.52%, 5쪽 84.62%, 14쪽 88.99%, 15쪽 89.64%.
//! 증거: `mydocs/pr/assets/issue_547/{native,wasm,native-base}/*_silhouette.tsv`,
//! 4·14쪽 review/overlay `mydocs/pr/assets/issue_547/{native,wasm}/{review,overlay}/`.
//! 남은 차이는 HFT 한양중고딕 본문·∙ 의 표시 글리프 모양(원 HFT 프로그램 없음)이다.
//! 절대 px 좌표는 고정하지 않는다. ∙ 검사는 독립 Print의 두 원점 사이 간격이다.
//!
//! 원인: 본문 `문체부 바탕체`(TTF, 문서 대체 필드 없음)는 독립 Print를 만든 PC에 없었고,
//! 한컴 2022는 그 본문을 `Haansoft Batang`(한컴바탕)으로 출력했다. rhwp 글꼴 계획은 이
//! 원본에 아무 대체 후보도 내지 않아, Native 웹폰트 공급과 `visual_sweep` 이 원장 밖
//! 원본을 일괄 `NotoSansKR-Regular`(고딕)로 치환했다. 4·5쪽 본문 전체가 고딕으로 그려졌다.
//! 14·15쪽은 기호 슬롯 HFT 한양중고딕 `∙` 의 전진을 표시용 HY중고딕 표 결측 때문에
//! 반각으로 재어 뒤 본문이 7px 앞당겨졌다.
//!
//! 독립 기대값: 같은 원본의 한컴 2022 Print `pdf/issue547/online-search-rfp-2022.pdf`
//! 4쪽 본문 span의 글꼴이 모두 `Haansoft Batang`(CID TrueType)이다. 다른 원본 10k 오라클
//! 728의 `HY울릉도L`(TTF, 대체 없음, 이름에 명조·바탕 없음) 제목도 같은 `Haansoft Batang`이다.
//! HFT 선언(`한양중고딕` 등)은 한컴 내장 Type3 로 그려지므로 이 대체 대상이 아니다.
//!
//! 검사: 실제 원본의 글꼴 계획이 TTF·대체 없는 원본에만 한컴 기본 대체를 싣고 HFT·문서
//! 대체·한컴 내장 HFT 이름(80168 `한양신명조`)에는 싣지 않는지, 공급 snapshot 환경이 원
//! 프로그램·문서 대체 뒤에서만 표시 이름으로 치환하고 프로그램 선택은 만들지 않는지,
//! 공급이 만드는 표시 치환을 소비한 4쪽 본문 글자가 한컴바탕으로
//! 그려지고 `  - ` 뒤 본문 원점 간격이 독립 Print와 같은지 확인한다. 실제 글꼴 설치를
//! 가정하지 않는다(공급 snapshot·환경 JSON 의 이름만 쓴다).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::font_environment::FontEnvironment;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use serde_json::{json, Value};

const SAMPLE: &str = "samples/issue547/online-search-rfp.hwp";
const BODY_FACE: &str = "문체부 바탕체";
const HFT_FACE: &str = "한양중고딕";
const SUBSTITUTED_FACE: &str = "-윤고딕120";
const HANCOM_DEFAULT_FAMILY: &str = "Haansoft Batang";

fn load() -> DocumentCore {
    let bytes = std::fs::read(SAMPLE).expect("issue547 원본");
    DocumentCore::from_bytes(&bytes).expect("issue547 원본 파싱")
}

fn program(query: &str, family: &str) -> Value {
    json!({"query": query, "family": family, "programId": format!("issue547-{family}"), "trueType": true})
}

/// 글꼴 계획 요청의 한컴 기본 대체 이름(직렬화 계약 `hancomMissingDefault`).
fn hancom_default(request: &impl serde::Serialize) -> Vec<String> {
    serde_json::to_value(request).unwrap()["hancomMissingDefault"]
        .as_array()
        .map(|names| {
            names
                .iter()
                .filter_map(|name| name.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn select(core: &DocumentCore, programs: Value) -> Value {
    serde_json::from_str(
        &core
            .build_font_environment_from_available_programs(&programs.to_string())
            .expect("글꼴 환경"),
    )
    .expect("글꼴 환경 JSON")
}

#[test]
fn missing_ttf_without_document_substitute_requests_hancom_batang() {
    let core = load();
    let requests = FontEnvironment::document_requests(&core.document().doc_info);
    let body: Vec<_> = requests.iter().filter(|r| r.source == BODY_FACE).collect();
    assert!(!body.is_empty(), "본문 글꼴 요청");
    for request in &body {
        assert!(
            request.substitute.is_empty(),
            "문서 대체 필드가 없는 TTF 선언"
        );
        let names = hancom_default(request);
        assert!(
            names.first().map(String::as_str) == Some("한컴바탕")
                && names.iter().any(|name| name == HANCOM_DEFAULT_FAMILY),
            "원 프로그램과 문서 대체가 없을 때 쓸 한컴 기본 대체: {names:?}"
        );
    }
    // HFT 는 한컴 내장 프로그램이 그리므로 기본 대체를 요청하지 않는다.
    for request in requests.iter().filter(|r| r.source == HFT_FACE) {
        assert!(hancom_default(request).is_empty(), "HFT 선언 {:?}", request);
    }
    // 문서 대체가 있는 TTF 는 문서 대체 필드가 소유한다.
    for request in requests.iter().filter(|r| r.source == SUBSTITUTED_FACE) {
        assert!(request.substitute.iter().any(|name| name == "한컴바탕"));
        assert!(
            hancom_default(request).is_empty(),
            "문서 대체 소유 {:?}",
            request
        );
    }
}

/// 반례: TTF로 선언됐지만 한컴이 내장 HFT로 그리는 `한양신명조`.
/// 독립 한컴 2020 Print `pdf/issue1891/80168_regulatory_analysis-hwpx-2020.pdf` 4쪽은
/// 이 원본의 TTF·대체 없는 `한양신명조` 본문(`<신설>`, `제4조(사업시행예정…`)을
/// Type3 /T10(HFT)으로 출력한다. 한컴 기본 대체(한컴바탕)를 요청하면 안 된다.
#[test]
fn ttf_declared_hancom_builtin_hft_name_does_not_request_hancom_batang() {
    let bytes =
        std::fs::read("samples/issue1891/80168_regulatory_analysis.hwpx").expect("80168 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("80168 원본 파싱");
    let doc_info = &core.document().doc_info;
    let declared = doc_info
        .font_faces
        .iter()
        .flatten()
        .filter(|font| font.name == "한양신명조")
        .collect::<Vec<_>>();
    assert!(!declared.is_empty(), "80168 의 한양신명조 선언");
    assert!(
        declared
            .iter()
            .all(|font| font.alt_type == 1 && font.subst_font.is_none()),
        "전제: TTF 선언이고 문서 대체가 없다"
    );
    let requests = FontEnvironment::document_requests(doc_info);
    let hanyang: Vec<_> = requests
        .iter()
        .filter(|r| r.source == "한양신명조")
        .collect();
    assert!(!hanyang.is_empty());
    for request in hanyang {
        assert!(
            hancom_default(request).is_empty(),
            "한컴 내장 HFT 이름은 한컴 기본 대체 대상이 아님: {request:?}"
        );
    }
}

/// 공급 snapshot 으로 만드는 환경(Native·WASM 공통)은 원 프로그램과 문서 대체가 없을 때만
/// 한컴 기본 대체를 표시 face 이름으로 치환한다. 프로그램 선택(TrueType 전진 소유)은 만들지
/// 않으며 원 프로그램이 공급되면 원 프로그램이 앞선다. HFT 선언은 바꾸지 않는다.
#[test]
fn hancom_default_is_a_display_substitution_after_original_and_document_substitute() {
    let core = load();
    let selected = select(&core, json!([program("한컴바탕", HANCOM_DEFAULT_FAMILY)]));
    assert_eq!(
        selected["substitutions"][SUBSTITUTED_FACE], HANCOM_DEFAULT_FAMILY,
        "문서 대체 필드의 한컴바탕은 종전 프로그램 선택"
    );
    assert_eq!(
        selected["substitutions"][BODY_FACE], "한컴바탕",
        "대체 없는 TTF 원본은 한컴 기본 대체 표시 이름"
    );
    assert!(
        selected["selectedPrograms"].get(BODY_FACE).is_none(),
        "표시 치환은 프로그램 선택이 아님: {selected}"
    );
    assert!(
        selected["substitutions"].get(HFT_FACE).is_none(),
        "HFT 선언은 한컴 기본 대체로 바꾸지 않음: {selected}"
    );
    let original = select(
        &core,
        json!([
            program("한컴바탕", HANCOM_DEFAULT_FAMILY),
            program(BODY_FACE, "MBatang")
        ]),
    );
    assert_eq!(
        original["substitutions"][BODY_FACE], "MBatang",
        "원 프로그램 선택"
    );
    let none = select(&core, json!([program("Batang", "Batang")]));
    assert!(
        none.is_null() || none["substitutions"].get(BODY_FACE).is_none(),
        "공급되지 않은 기본 대체를 선택하지 않음: {none}"
    );
}

fn bullet_runs(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text == "∙ " && run.cell_context.is_some() {
            out.push((node.bbox.x, node.bbox.width));
        }
    }
    for child in &node.children {
        bullet_runs(child, out);
    }
}

/// 기호 슬롯 HFT 한양중고딕 `∙`(U+2219)는 독립 Print에서 1em을 전진한다.
///
/// 독립 기대값: Print 14쪽 `/T14` ∙ 원점 x=250.0pt 와 뒤 `/T13` 본문 원점 x=266.2pt 의
/// 간격 16.2pt(21.6px), 15쪽 ∙ 224.5pt → 본문 244.0pt 의 19.5pt(26.0px). 이 간격은
/// `∙`(/Widths 1000)과 반각 공백(/Widths 500)의 합이다. 수정 전에는 표시용 HY중고딕
/// 표의 U+2219 결측을 반각으로 재어 `∙ ` 런이 14.4px·17.3px 로 좁았다.
#[test]
fn hft_symbol_slot_bullet_advances_one_em() {
    let core = load();
    let doc_info = &core.document().doc_info;
    let symbol = &doc_info.font_faces[5];
    let shape = &doc_info.char_shapes[139];
    let face = &symbol[usize::from(shape.font_ids[5])];
    assert_eq!(face.name, HFT_FACE, "∙ 런의 기호 슬롯 원본");
    assert_eq!(face.alt_type, 2, "HFT 선언");
    for (page, expected) in [(13u32, 21.6f64), (14, 26.0)] {
        let tree = core.build_page_render_tree(page).expect("render tree");
        let mut runs = Vec::new();
        bullet_runs(&tree.root, &mut runs);
        assert!(runs.len() >= 10, "{}쪽 표의 ∙ 런 {}", page + 1, runs.len());
        for (x, width) in runs {
            assert!(
                (width - expected).abs() <= 0.5,
                "{}쪽 x={x:.1}: `∙ ` 폭 {width:.2}px, 독립 Print 원점 간격 {expected:.1}px",
                page + 1
            );
        }
    }
}

/// 한컴 기본 대체 프로그램의 조회 이름(한국어 family 또는 영문 family).
fn is_hancom_default(family: &str) -> bool {
    family == "한컴바탕" || family == HANCOM_DEFAULT_FAMILY
}

/// `font-family` 의 첫 이름을 글자별로 모은다.
fn first_families(svg: &str, glyph: &str) -> Vec<String> {
    let needle = format!(">{glyph}</text>");
    svg.match_indices(&needle)
        .filter_map(|(end, _)| {
            let start = svg[..end].rfind("<text")?;
            let tag = &svg[start..end];
            let family = tag.split("font-family=\"").nth(1)?;
            let family = family.split('"').next()?;
            Some(
                family
                    .split(',')
                    .next()?
                    .trim()
                    .trim_start_matches("&apos;")
                    .trim_end_matches("&apos;")
                    .to_string(),
            )
        })
        .collect()
}

fn text_origins(node: &RenderNode, out: &mut Vec<(String, i64, i64)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push((
            run.text.clone(),
            (node.bbox.x * 100.0).round() as i64,
            (node.bbox.y * 100.0).round() as i64,
        ));
    }
    for child in &node.children {
        text_origins(child, out);
    }
}

/// 공급이 계획의 한컴 기본 대체로 만드는 표시 치환을 소비하면 4쪽 본문 글자는 한컴바탕이
/// 그리고, 원본 표에 없는 `-` 는 한컴바탕 전진으로 재어 독립 Print 원점 간격과 같다.
/// HFT 본문은 그대로다.
#[test]
fn hancom_default_supply_owns_body_paint_and_dash_advance() {
    let mut core = load();
    let request = FontEnvironment::document_requests(&core.document().doc_info)
        .into_iter()
        .find(|r| r.source == BODY_FACE)
        .expect("본문 요청");
    // 공급은 선언 순서의 첫 설치 이름(`한컴바탕`)으로 표시 치환한다.
    let target = hancom_default(&request)
        .first()
        .expect("공급 face 이름")
        .clone();
    assert_eq!(target, "한컴바탕");
    let environment = json!({
        "id": "issue547-hancom-default-supply",
        "substitutions": {BODY_FACE: target},
        "preserveSourceMetrics": true
    });
    core.set_font_environment(Some(
        FontEnvironment::from_json(&environment.to_string()).unwrap(),
    ))
    .unwrap();
    // 독립 Print 4쪽 `  - ` 줄은 줄 시작 75.52px 에서 본문 첫 글자 125.28px 까지
    // 49.76px 을 전진한다(rawdict 원점, 96 DPI). 한컴바탕의 `-` 전진이다.
    // 종전 일반 Noto 공급은 이 간격을 42.66px 로 재어 본문이 7px 앞당겨졌다.
    let mut runs = Vec::new();
    text_origins(&core.build_page_render_tree(3).unwrap().root, &mut runs);
    let gaps: Vec<f64> = runs
        .windows(2)
        .filter(|pair| pair[0].0 == "  - " && pair[0].2 == pair[1].2)
        .map(|pair| (pair[1].1 - pair[0].1) as f64 / 100.0)
        .collect();
    assert!(gaps.len() >= 3, "4쪽 `  - ` 줄 {gaps:?}");
    for gap in &gaps {
        assert!(
            (gap - 49.76).abs() <= 0.5,
            "`  - ` 뒤 본문 원점 간격 {gap:.2}px, 독립 Print 49.76px: {gaps:?}"
        );
    }
    // 4쪽(0-기준 3) 본문 `온라인 수색` 의 글자는 한컴바탕이 그린다.
    let page4 = core.render_page_svg_native(3).unwrap();
    let body = first_families(&page4, "색");
    assert!(body.len() >= 5, "4쪽 본문 `색` {body:?}");
    assert!(
        body.iter().all(|family| is_hancom_default(family)),
        "4쪽 본문은 한컴 기본 대체가 그림: {body:?}"
    );
    // 14쪽(0-기준 13) HFT 한양중고딕 본문은 기존 HFT 대체 체인을 유지한다.
    let page14 = core.render_page_svg_native(13).unwrap();
    let hft = first_families(&page14, "서");
    assert!(!hft.is_empty(), "14쪽 HFT 본문 `서`");
    assert!(
        hft.iter().all(|family| !is_hancom_default(family)),
        "HFT 본문은 한컴 기본 대체로 바꾸지 않음: {hft:?}"
    );
}
