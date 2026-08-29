//! [#6350] 수식 속성 setter 순수화 — getter 봉지 재적용이 저장 바이트에 항등인가.
//!
//! 종전 setter 는 봉지 반영 뒤 두 가지를 무조건 했다.
//!
//! - 자동 크기(`intrinsic_size_hwp`)를 `common.width/height` 에 덧써 명시 크기를 무시했다.
//! - `raw_ctrl_data.clear()` 로 원본 CTRL_HEADER 를 파괴했다. #4495 봉인이 들어온 뒤로는
//!   중복이다 — 저장기가 `common` 봉인 불일치를 보고 이미 IR 합성으로 내려간다.
//!
//! 결과로 값을 하나도 바꾸지 않은 재적용조차 저장 바이트를 바꿨고, 속성 봉지만으로
//! 되돌릴 수 없었다. 여기서는 **공개 API 만으로** 그 계약을 고정한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

/// 한컴 편집기가 만든 수식 문서들. 저장 크기와 rhwp 자동 계산 크기가 서로 다르므로
/// (예: equation-lim 9096x2715 vs 9327x2684) 자동 크기 덧쓰기가 곧 손실로 드러난다.
const SAMPLES: [&str; 3] = [
    "samples/equation-lim.hwp",
    "samples/atop-equation-01.hwp",
    "samples/issue-505-equations.hwp",
];

fn read_fixture(path: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|e| panic!("read {path}: {e}"))
}

fn find_equation(core: &DocumentCore) -> (usize, usize, usize) {
    for (si, section) in core.document().sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                if matches!(ctrl, Control::Equation(_)) {
                    return (si, pi, ci);
                }
            }
        }
    }
    panic!("샘플에 수식 컨트롤이 없다");
}

fn equation_at(
    core: &DocumentCore,
    si: usize,
    pi: usize,
    ci: usize,
) -> &rhwp::model::control::Equation {
    match &core.document().sections[si].paragraphs[pi].controls[ci] {
        Control::Equation(eq) => eq,
        other => panic!("수식이 아니다: {other:?}"),
    }
}

/// getter 가 낸 봉지를 값 변경 없이 그대로 먹이면 저장 바이트가 같아야 한다(get∘set = 항등).
///
/// 기준선도 구역 패스스루(`raw_stream`)를 비워 setter 와 같은 조건에서 비교한다 — 판정
/// 대상은 수식 CTRL_HEADER 이고 구역 스트림 봉인은 #4488 이 따로 다룬다.
///
/// 수정 전에는 세 샘플 모두 불일치했다: `raw_ctrl_data`(48/54/48B)가 0B 로 파괴되고
/// 자동 크기가 `common.width/height` 를 덧썼다.
#[test]
fn getter_bag_reapplied_keeps_saved_bytes_identical() {
    for sample in SAMPLES {
        let bytes = read_fixture(sample);

        let mut base = DocumentCore::from_bytes(&bytes).expect("파싱");
        let (si, pi, ci) = find_equation(&base);
        {
            let eq = equation_at(&base, si, pi, ci);
            assert!(
                !eq.raw_ctrl_data.is_empty() && eq.raw_ctrl_seal.is_some(),
                "{sample}: 전제 — 파스본 수식은 봉인된 raw 를 가진다"
            );
        }
        let bag = base
            .get_equation_properties_native(si, pi, ci, None, None)
            .expect("getter");
        base.document_mut().sections[si].raw_stream = None;
        let baseline = base.export_hwp_native().expect("기준 저장");

        let mut edited = DocumentCore::from_bytes(&bytes).expect("파싱");
        edited
            .set_equation_properties_native(si, pi, ci, None, None, &bag)
            .expect("setter");
        let roundtripped = edited.export_hwp_native().expect("편집 저장");

        assert_eq!(
            baseline, roundtripped,
            "{sample}: getter 봉지를 그대로 먹인 뒤 저장 바이트가 달라졌다"
        );
    }
}

/// 봉인된 raw 는 setter 가 파괴하지 않는다 — 저장기가 봉인 불일치로 IR 합성에 내려가므로
/// 지울 이유가 없고, 지우지 않으면 되돌릴 때 원본 바이트가 살아난다.
#[test]
fn script_edit_keeps_sealed_raw_ctrl_data() {
    for sample in SAMPLES {
        let bytes = read_fixture(sample);
        let mut core = DocumentCore::from_bytes(&bytes).expect("파싱");
        let (si, pi, ci) = find_equation(&core);
        let original_raw = equation_at(&core, si, pi, ci).raw_ctrl_data.clone();

        core.set_equation_properties_native(si, pi, ci, None, None, r#"{"script":"x^2 + 1"}"#)
            .expect("스크립트 편집");

        let eq = equation_at(&core, si, pi, ci);
        assert_eq!(eq.script, "x^2 + 1", "{sample}: 편집이 반영됐다");
        assert_eq!(
            eq.raw_ctrl_data, original_raw,
            "{sample}: 봉인된 raw_ctrl_data 를 setter 가 지우지 말아야 한다"
        );
    }
}

/// 종전 `raw_ctrl_data.clear()` 가 지키던 계약은 그대로다 — 실제 크기 편집은 봉인
/// 불일치로 저장기를 IR 합성으로 내려 저장·재파싱에서 살아남는다.
#[test]
fn size_edit_survives_save_and_reparse() {
    for sample in SAMPLES {
        let bytes = read_fixture(sample);
        let mut core = DocumentCore::from_bytes(&bytes).expect("파싱");
        let (si, pi, ci) = find_equation(&core);

        core.set_equation_properties_native(
            si,
            pi,
            ci,
            None,
            None,
            r#"{"width":5000,"height":4000}"#,
        )
        .expect("크기 편집");
        let saved = core.export_hwp_native().expect("저장");

        let reparsed = DocumentCore::from_bytes(&saved).expect("재파싱");
        let eq = equation_at(&reparsed, si, pi, ci);
        assert_eq!(
            (eq.common.width, eq.common.height),
            (5000, 4000),
            "{sample}: 크기 편집이 저장·재파싱에서 살아남지 않았다"
        );
    }
}

/// 자동 크기 파생은 봉지가 지정하지 않은 축에만 적용한다.
///
/// 종전에는 무조건 덧써서 명시된 width/height 가 조용히 무시됐다. 변경 키만 담는 부분
/// 봉지(스튜디오 다이얼로그 계약)는 두 축 모두 파생값을 받아 종전대로 동작한다.
#[test]
fn explicit_size_wins_over_intrinsic_derivation() {
    let bytes = read_fixture(SAMPLES[0]);

    // 양축 명시 — 파생이 덧쓰지 않는다.
    let mut both = DocumentCore::from_bytes(&bytes).expect("파싱");
    let (si, pi, ci) = find_equation(&both);
    both.set_equation_properties_native(
        si,
        pi,
        ci,
        None,
        None,
        r#"{"script":"x","width":5000,"height":4000}"#,
    )
    .expect("양축 명시");
    let eq = equation_at(&both, si, pi, ci);
    assert_eq!((eq.common.width, eq.common.height), (5000, 4000));

    // 크기를 담지 않은 부분 봉지 — 두 축 모두 파생값을 받는다.
    let mut auto = DocumentCore::from_bytes(&bytes).expect("파싱");
    auto.set_equation_properties_native(si, pi, ci, None, None, r#"{"script":"x"}"#)
        .expect("부분 봉지");
    let derived = {
        let eq = equation_at(&auto, si, pi, ci);
        (eq.common.width, eq.common.height)
    };
    assert_ne!(
        derived,
        (5000, 4000),
        "크기를 지정하지 않으면 스크립트에서 파생된 크기가 와야 한다"
    );

    // 한 축만 지정 — 지정한 축은 그대로, 나머지는 파생값.
    let mut half = DocumentCore::from_bytes(&bytes).expect("파싱");
    half.set_equation_properties_native(si, pi, ci, None, None, r#"{"script":"x","width":5000}"#)
        .expect("한 축 명시");
    let eq = equation_at(&half, si, pi, ci);
    assert_eq!(eq.common.width, 5000, "명시한 축은 유지된다");
    assert_eq!(eq.common.height, derived.1, "나머지 축은 파생값을 받는다");
}
