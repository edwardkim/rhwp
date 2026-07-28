//! Issue #3492: HWP3 문서를 저장하면 재파싱한 IR 이 달라진다 — `convert --verify` 가 exit 3.
//!
//! `samples/SO-SUEOP.hwp`(HWP 3.0, 46쪽)를 HWP5 로 저장하고 되읽으면 컨트롤 93개가
//! 사라진 것으로 보고됐다. 정체는 **HWP3 개요번호 마커**다.
//!
//! HWP3 파서는 이 마커를 `Control::Field("Outline:kind=..:level=..")` 로 만든 뒤
//! `fixup_hwp3_outline_fields` 에서 읽어 `ParaShape::head_type`·`para_level` 과
//! `Numbering` 으로 옮긴다. 옮긴 뒤에는 아무도 읽지 않는데 공통 IR 에는 그대로 남았다.
//!
//! 이 잔재는 **텍스트 앵커가 없다** — 오브젝트 문자를 남기지 않아 `char_offsets` 어디에도
//! 대응 위치가 없다. 그래서 두 가지가 동시에 깨졌다.
//!
//!   1) HWP5 저장기가 자리 없는 컨트롤에 필드 begin/end(각 8 코드 유닛)를 지어내
//!      문자 수를 부풀리고, 재파싱하면 필드로 복원되지 않아 `--verify` 가 손실로 판정
//!   2) 앵커가 없으면서도 `controls` 앞자리를 차지해, 같은 문단의 **미주 참조 표시가
//!      제 오프셋을 빼앗기고** 줄 끝으로 밀렸다
//!
//! 계약: 마커는 소비 직후 공통 IR 에서 걷어낸다. 번호 정보는 `ParaShape` 에 남는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::style::HeadType;

const SAMPLE: &str = "samples/SO-SUEOP.hwp";
/// 개요번호가 붙는 문단 수 — 마커를 걷어내도 번호는 남아야 한다.
const OUTLINE_PARAGRAPHS: usize = 92;

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn parse_sample() -> rhwp::model::document::Document {
    let data = std::fs::read(sample_path()).expect("샘플 읽기");
    rhwp::parser::parse_document(&data).expect("HWP3 파싱")
}

fn is_outline_marker(control: &Control) -> bool {
    matches!(control, Control::Field(f) if f.command.starts_with("Outline:"))
}

/// HWP3 전용 개요번호 마커는 공통 IR 로 새어 나오지 않는다.
#[test]
fn outline_markers_do_not_leak_into_the_common_ir() {
    let doc = parse_sample();
    let leaked: Vec<String> = doc
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .flat_map(|p| p.controls.iter())
        .filter_map(|c| match c {
            Control::Field(f) if f.command.starts_with("Outline:") => Some(f.command.clone()),
            _ => None,
        })
        .collect();
    assert!(
        leaked.is_empty(),
        "HWP3 개요번호 마커가 공통 IR 에 남았다 ({}건): {:?}",
        leaked.len(),
        &leaked[..leaked.len().min(3)]
    );
}

/// 마커를 걷어내도 번호 정보(`head_type`·`Numbering`)는 그대로다 — 정보 손실이 아니다.
#[test]
fn outline_numbering_survives_the_marker_removal() {
    let doc = parse_sample();
    let numbered = doc
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .filter(|p| {
            doc.doc_info
                .para_shapes
                .get(p.para_shape_id as usize)
                .is_some_and(|s| s.head_type == HeadType::Number)
        })
        .count();
    assert_eq!(
        numbered, OUTLINE_PARAGRAPHS,
        "개요번호 문단 수가 달라졌다 — 마커 제거가 번호까지 지웠는지 확인하라"
    );
    assert!(
        !doc.doc_info.numberings.is_empty(),
        "개요번호 정의(Numbering)가 사라졌다"
    );
}

/// 앵커 없는 마커가 앞자리를 차지하면 뒤 컨트롤이 제 오프셋을 잃는다.
///
/// `char_offsets` 의 8 코드 유닛 공백은 미주 것인데, 마커가 `controls[0]` 이라 그 자리를
/// 가져가고 미주 표시가 줄 끝으로 밀렸다. 마커를 걷어내면 미주가 제 위치로 돌아온다.
#[test]
fn endnote_marks_keep_their_recorded_offset() {
    let doc = parse_sample();
    let mut checked = 0usize;
    for section in &doc.sections {
        for paragraph in &section.paragraphs {
            let Some(endnote_idx) = paragraph
                .controls
                .iter()
                .position(|c| matches!(c, Control::Endnote(_)))
            else {
                continue;
            };
            // 오프셋에 공백이 있는 문단만 — 미주가 본문 중간에 앵커된 경우다.
            let gap = paragraph
                .char_offsets
                .windows(2)
                .position(|w| w[1] - w[0] > 1);
            let Some(gap) = gap else { continue };
            let positions = paragraph.control_text_positions();
            assert_eq!(
                positions.get(endnote_idx).copied(),
                Some(gap + 1),
                "미주 표시가 기록된 오프셋을 벗어났다: controls={} text={:?} offsets={:?} positions={:?}",
                paragraph.controls.len(),
                paragraph.text,
                paragraph.char_offsets,
                positions
            );
            checked += 1;
        }
    }
    assert!(
        checked > 0,
        "미주 앵커 문단을 찾지 못했다 — 표본이 바뀌었는지 확인하라"
    );
}

/// 보고된 계약 그대로: `convert --verify` 가 exit 0.
#[test]
fn convert_verify_reports_no_ir_loss() {
    let out = std::env::temp_dir().join(format!(
        "rhwp-issue3492-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("convert")
        .arg(sample_path())
        .arg(&out)
        .arg("--verify")
        .output()
        .expect("rhwp 실행");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _ = std::fs::remove_file(&out);
    assert_eq!(
        output.status.code(),
        Some(0),
        "convert --verify 가 IR 손실을 보고했다:\n{stdout}"
    );
}
