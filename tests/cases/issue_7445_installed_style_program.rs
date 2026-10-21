//! #7445 설치한 동일 프로그램의 전체 이름과 PostScript 이름을 함께 조회한다.
//! KoPub 6개 SFNT의 name1/4/6/16/17과 독립 #7009 Print의 KoPub 공급을 대조했다.
//! 프로그램 선택 API 계약이며 실물 미달 페이지의 좌표나 모양을 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{renderer::font_environment::FontEnvironment, DocumentCore};
use serde_json::json;
#[test]
fn supplied_style_program_precedes_the_document_substitute() {
    let mut errors = vec![];
    for (kind, ko, en, ps) in [
        ("Dotum", "돋움체", "Dotum", "Dotum"),
        ("Batang", "바탕체", "Batang", "Batang"),
    ] {
        for weight in ["Light", "Medium", "Bold"] {
            let postscript = format!("KoPub{ps}{weight}");
            for name in [
                format!("KoPub{ko} {weight}"),
                format!("KoPub{en} {weight}"),
                format!("KoPub {en} {weight}"),
                postscript.clone(),
            ] {
                let mut c = DocumentCore::new_empty();
                c.create_blank_document_native().unwrap();
                let mut d = c.document().clone();
                for f in d.doc_info.font_faces.iter_mut().flatten() {
                    f.name = name.clone();
                    f.alt_type = 1;
                    f.subst_font = Some(rhwp::model::style::SubstFont {
                        face: "HCR Batang".into(),
                        font_type: 1,
                        ..Default::default()
                    });
                }
                let supplied = json!([{"query":postscript,"family":format!("KoPub{kind}"),"programId":format!("same-{kind}-{weight}-font-program"),"trueType":true},{"query":"HCR Batang","family":"HCR Batang","programId":"fallback-program","trueType":true}]);
                if FontEnvironment::from_available_programs(&d.doc_info, &supplied.to_string())
                    .unwrap()
                    .is_some()
                {
                    errors.push(name.clone());
                }
                let wrong_weight = if weight == "Bold" { "Light" } else { "Bold" };
                let wrong_program = json!([{"query":format!("KoPub{ps}{wrong_weight}"),"family":format!("KoPub{kind}"),"programId":"another-weight-program","trueType":true},{"query":"HCR Batang","family":"HCR Batang","programId":"fallback-program","trueType":true}]);
                assert!(
                    FontEnvironment::from_available_programs(
                        &d.doc_info,
                        &wrong_program.to_string()
                    )
                    .unwrap()
                    .is_some(),
                    "다른 굵기의 파일을 같은 프로그램 공급으로 간주하지 않는다"
                );
                let fallback = json!([{"query":"HCR Batang","family":"HCR Batang","programId":"fallback-program","trueType":true}]);
                assert!(
                    FontEnvironment::from_available_programs(&d.doc_info, &fallback.to_string())
                        .unwrap()
                        .is_some(),
                    "실제 원본 프로그램이 없으면 문서의 대체를 유지한다"
                );
            }
        }
    }
    assert!(
        errors.is_empty(),
        "같은 SFNT의 언어/전체 이름·PostScript 이름을 다른 프로그램 결측으로 취급했다: {errors:?}"
    );
}
