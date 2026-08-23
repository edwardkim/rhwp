#![cfg(not(target_arch = "wasm32"))]

//! [#5769] Stage 4 선결 실증 — 페이지/구역 설정 setter 의 속성쌍 역연산 수렴성.
//!
//! set_section_def / set_page_def 는 적용 시 `section.raw_stream = None` 으로
//! passthrough 를 무효화하고, page_def 는 추가로 wrap 폭 변화 시 본문 전체를
//! 재래핑한다([#4956] `reflow_body_paragraphs_in_section` — 저장 line_segs 전면
//! 재작성). 같은 setter 로 old 를 다시 적용하면 부작용도 대칭으로 재실행되므로,
//! "new 적용 → old 복원" 왕복 뒤 저장 바이트가 변경 전으로 돌아오는지가 속성쌍
//! 커맨드(스냅샷 대체 역연산) 가능 여부의 판정이다.
//!
//! 실증 판정(표본 hongbo):
//! - section_def: 속성쌍만으론 **불수렴** — 속성을 하나도 바꾸지 않고 같은 값을
//!   한 번 적용만 해도 동일 델타가 난다(raw 무효화 → IR 재구성 직렬화 경로).
//!   구역 raw 저널(`section_raw_journal.rs`)로 passthrough 를 되돌리면 바이트
//!   완전 수렴한다 → 구역 설정 다이얼로그의 역연산화 근거.
//! - page_def: raw 복원으로도 **불수렴** — 재래핑이 한컴 원본 줄 나눔을 rhwp
//!   조판값으로 교체하기 때문(len 562176→561664 잔류). page_setup·page_margin 은
//!   스냅샷 잔류다(그림 회전 선례 준용, 계획서 task_m100_5769.md Stage 4).
//!
//! 불수렴 단정은 트립와이어다 — 직렬화 충실도 개선(#5890 계열)이나 조판 정합으로
//! 정반대가 되면 이 시험이 깨져 의식적 설계 갱신을 강제한다.

use rhwp::document_core::DocumentCore;

const HONGBO: &str = "samples/20250130-hongbo-no.hwp";

fn load_core() -> DocumentCore {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(HONGBO))
        .expect("표본 로드");
    DocumentCore::from_bytes(&bytes).expect("파싱")
}

fn first_diff(a: &[u8], b: &[u8]) -> Option<usize> {
    let n = a.len().min(b.len());
    (0..n)
        .find(|&i| a[i] != b[i])
        .or((a.len() != b.len()).then_some(n))
}

/// hideHeader 를 뒤집는 최소 변경 JSON — 비-기하 키라 wrap 폭·line_segs 는 불변.
fn flipped_hide_header(core: &mut DocumentCore, sec: usize) -> (String, String) {
    let old_json = core.get_section_def_native(sec).expect("get section def");
    let old: serde_json::Value = serde_json::from_str(&old_json).expect("getter JSON");
    let new_json = format!(
        "{{\"hideHeader\":{}}}",
        !old["hideHeader"].as_bool().expect("hideHeader")
    );
    (old_json, new_json)
}

#[test]
fn section_def_property_pair_without_raw_restore_diverges() {
    let mut core = load_core();
    let before = core.export_hwp_native().expect("변경 전 export");
    let sec = 0;
    let (old_json, new_json) = flipped_hide_header(&mut core, sec);

    core.set_section_def_native(sec, &new_json)
        .expect("변경 적용");
    core.set_section_def_native(sec, &old_json)
        .expect("old 복원");

    let after = core.export_hwp_native().expect("복원 후 export");
    assert!(
        first_diff(&before, &after).is_some(),
        "raw 미복원 속성쌍 왕복이 수렴하면 이 시험의 전제(무효화→재직렬화 델타)가 무너진 것이다 — \
         구역 raw 저널 없이도 역연산화 가능해졌는지 재검토하라"
    );
}

#[test]
fn section_def_property_pair_with_raw_journal_converges_byte_exact() {
    let mut core = load_core();
    let before = core.export_hwp_native().expect("변경 전 export");
    let sec = 0;
    let (old_json, new_json) = flipped_hide_header(&mut core, sec);

    // undo 경로 — TS SetSectionPropsCommand 가 밟을 순서 그대로:
    // capture(변경 전) → apply new → apply old → restore raw.
    let cap = core.capture_section_raw_native(sec).expect("캡처");
    core.set_section_def_native(sec, &new_json)
        .expect("변경 적용");
    core.set_section_def_native(sec, &old_json)
        .expect("old 복원");
    core.restore_section_raw_native(cap).expect("raw 복원");

    let after = core.export_hwp_native().expect("복원 후 export");
    assert_eq!(before.len(), after.len(), "바이트 길이 수렴");
    let diff = first_diff(&before, &after);
    assert!(
        diff.is_none(),
        "raw 저널 복원 후 저장 바이트 완전 수렴 — 첫 불일치 @{diff:?}"
    );
}

#[test]
fn section_def_double_roundtrip_stays_convergent() {
    // redo 재실행은 새 캡처를 만든다(undo 가 저널을 소비하므로). 소비·재캡처
    // 생명주기가 반복 왕복에서도 정합인지 확인한다.
    let mut core = load_core();
    let before = core.export_hwp_native().expect("변경 전 export");
    let sec = 0;
    let (old_json, new_json) = flipped_hide_header(&mut core, sec);

    for _ in 0..2 {
        let cap = core.capture_section_raw_native(sec).expect("캡처");
        core.set_section_def_native(sec, &new_json)
            .expect("변경 적용");
        core.set_section_def_native(sec, &old_json)
            .expect("old 복원");
        core.restore_section_raw_native(cap).expect("raw 복원");
    }

    let after = core.export_hwp_native().expect("복원 후 export");
    let diff = first_diff(&before, &after);
    assert!(diff.is_none(), "2회 왕복 후에도 수렴 — 첫 불일치 @{diff:?}");
}

#[test]
fn section_raw_restore_rejects_when_passthrough_alive() {
    // 전제 검증 — setter 가 무효화한(None) 직후가 아니면 복원을 거부한다.
    // 이미 살아있는(Some) 상태에서의 복원 요청은 이중 undo 같은 배선 버그다.
    let mut core = load_core();
    let sec = 0;
    let (_, new_json) = flipped_hide_header(&mut core, sec);

    let cap1 = core.capture_section_raw_native(sec).expect("캡처 1");
    let cap2 = core.capture_section_raw_native(sec).expect("캡처 2");

    core.set_section_def_native(sec, &new_json)
        .expect("변경 적용");
    core.restore_section_raw_native(cap1)
        .expect("첫 복원은 성공");

    let second = core.restore_section_raw_native(cap2);
    assert!(
        second.is_err(),
        "passthrough 가 살아있는 상태의 중복 복원은 거부돼야 한다"
    );

    // 거부가 캡처를 소비하면 CommandHistory 는 같은 undo 를 다시 시도할 수 없다.
    // setter 로 raw 를 다시 무효화한 뒤 같은 캡처가 복원 가능한지 확인한다.
    core.set_section_def_native(sec, &new_json)
        .expect("재무효화 적용");
    core.restore_section_raw_native(cap2)
        .expect("거부 뒤 살아있는 캡처 복원");
}

#[test]
fn discarded_capture_is_gone() {
    let mut core = load_core();
    let sec = 0;
    let (_, new_json) = flipped_hide_header(&mut core, sec);

    let cap = core.capture_section_raw_native(sec).expect("캡처");
    core.discard_section_raw_native(cap);
    core.set_section_def_native(sec, &new_json)
        .expect("변경 적용");

    let restore = core.restore_section_raw_native(cap);
    assert!(restore.is_err(), "discard 된 캡처로는 복원할 수 없다");
}

#[test]
fn page_def_property_pair_does_not_converge_reflow_remains() {
    // 기하 키 — wrap 폭 변동이 재래핑을 트리거해 한컴 원본 line_segs 가 rhwp
    // 조판값으로 교체되고, old 복원으로도 돌아오지 않는다. page_setup·page_margin
    // 이 스냅샷에 잔류하는 이유의 실증이다. 수렴하게 됐다면(조판 정합·직렬화
    // 충실도 개선) 이 시험이 깨진다 — 그때 역연산화로 설계을 갱신하라.
    let mut core = load_core();
    let before = core.export_hwp_native().expect("변경 전 export");
    let sec = 0;

    let old_json = core.get_page_def_native(sec).expect("get page def");
    let old: serde_json::Value = serde_json::from_str(&old_json).expect("getter JSON");
    let ml = old["marginLeft"].as_u64().unwrap();
    let mr = old["marginRight"].as_u64().unwrap();
    let new_json = format!(
        "{{\"marginLeft\":{},\"marginRight\":{}}}",
        ml + 1500,
        mr + 1500
    );

    core.set_page_def_native(sec, &new_json).expect("변경 적용");
    core.set_page_def_native(sec, &old_json).expect("old 복원");

    let after = core.export_hwp_native().expect("복원 후 export");
    assert!(
        first_diff(&before, &after).is_some(),
        "page_def 속성쌍 왕복이 수렴했다 — reflow 부작용이 사라진 것이므로 \
         page_setup/page_margin 도 역연산화 대상으로 재판정하라"
    );
}
