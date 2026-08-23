#![cfg(not(target_arch = "wasm32"))]

//! [#5769 후속] Shape z 순서 역연산의 저장 바이트 왕복 동일성.
//!
//! `change_shape_z_order_native` 은 실제 변경 시 자기기술 레코드 `moves`(대상+교환
//! 이웃의 before/after)를 돌려주고, `apply_shape_z_order_pairs_native` 이 그 절대
//! 대입을 수행한다. TS `SetZOrderCommand` 가 밟는 순서 — capture → 상대 연산 →
//! old 쌍 대입 → raw 복원 — 를 코어 수준에서 그대로 재현해 저장 바이트가 원본으로
//! 돌아오는지 증명한다. Shape z 대입에는 대입 외 부작용이 없으므로(#5769 선결 규약)
//! 속성쌍 왕복이 참인 역연산이다.
//!
//! 무변경 경계("이미 맨 앞/뒤")는 moves 없이 조기 반환하며 passthrough 도 건드리지
//! 않는다 — phantom 엔트리와 불필요한 raw 무효화가 없음을 함께 고정한다.

use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

/// 도형 n개 문서 — 생성 순서대로 z_order 1..=n (max+1 규약).
fn doc_with_shapes(count: usize) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    let spots: Vec<(u32, u32)> = vec![(0, 0), (12000, 0), (24000, 0)];
    for i in 0..count {
        let (x, y) = spots[i % spots.len()];
        doc.create_shape_control_native(
            0,
            0,
            0,
            4000,
            4000,
            x + (i as u32) * 100,
            y,
            false,
            "InFrontOfText",
            "rectangle",
            false,
            false,
            &[],
        )
        .expect("도형 삽입");
    }
    doc
}

/// 구역 0의 Shape z_order 목록(문단 순).
fn z_orders(doc: &HwpDocument) -> Vec<i32> {
    let mut out = Vec::new();
    for p in &doc.document().sections[0].paragraphs {
        for c in &p.controls {
            if let Control::Shape(shape) = c {
                out.push(shape.z_order());
            }
        }
    }
    out
}

/// 첫 Shape 의 (para_idx, control_idx) — 재로드 문서는 파서 배치가 생성 시와 다르다.
fn first_shape_at(doc: &HwpDocument) -> (usize, usize) {
    for (pi, p) in doc.document().sections[0].paragraphs.iter().enumerate() {
        for (ci, c) in p.controls.iter().enumerate() {
            if matches!(c, Control::Shape(_)) {
                return (pi, ci);
            }
        }
    }
    panic!("구역 0에 Shape 가 없다");
}

/// change_shape_z_order_native 응답에서 moves 를 파싱한다(없으면 빈 벡터).
fn parse_moves(resp: &str) -> Vec<(usize, usize, i32, i64)> {
    let v: Value = serde_json::from_str(resp).expect("응답 JSON");
    v["moves"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|m| {
                    (
                        m["ppi"].as_u64().expect("ppi") as usize,
                        m["ci"].as_u64().expect("ci") as usize,
                        m["before"].as_i64().expect("before") as i32,
                        m["after"].as_i64().expect("after"),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// moves 를 주어진 방향("before"/"after")의 절대 대입 pairs JSON 으로 바꾼다.
fn pairs_json(moves: &[(usize, usize, i32, i64)], dir: &str) -> String {
    let items: Vec<String> = moves
        .iter()
        .map(|(ppi, ci, before, after)| {
            let z = if dir == "before" {
                i64::from(*before)
            } else {
                *after
            };
            format!("{{\"ppi\":{ppi},\"ci\":{ci},\"z\":{z}}}")
        })
        .collect();
    format!("[{}]", items.join(","))
}

#[test]
fn forward_swap_undo_restores_bytes_exact() {
    let mut doc = doc_with_shapes(3);
    let z_initial = z_orders(&doc);
    assert_eq!(z_initial.len(), 3, "도형 3개 준비");
    let before = doc.export_hwp().expect("baseline");

    // undo 경로 전체 — TS SetZOrderCommand 순서 그대로:
    // capture → 상대 연산(forward) → old 쌍 절대 대입 → raw 복원.
    let cap = doc.capture_section_raw_native(0).expect("캡처");
    let resp = doc
        .change_shape_z_order_native(0, 0, 0, "forward")
        .expect("forward");
    let moves = parse_moves(&resp);
    assert_eq!(moves.len(), 2, "교환은 대상+이웃 쌍 — {resp}");
    assert_ne!(doc.export_hwp().expect("변경 후"), before, "변경 실재");

    doc.apply_shape_z_order_pairs_native(0, &pairs_json(&moves, "before"))
        .expect("old 복원");
    assert_eq!(z_orders(&doc), z_initial, "z 값 복원");
    doc.restore_section_raw_native(cap).expect("raw 복원");

    let after = doc.export_hwp().expect("복원 후 export");
    assert_eq!(before.len(), after.len(), "바이트 길이 수렴");
    let diff = before.iter().zip(after.iter()).position(|(a, b)| a != b);
    assert!(
        diff.is_none(),
        "undo 후 원본 바이트 완전 수렴 — 첫 불일치 @{diff:?}"
    );
}

#[test]
fn front_target_only_inverse_restores_bytes_exact() {
    let mut doc = doc_with_shapes(3);
    let before = doc.export_hwp().expect("baseline");

    let cap = doc.capture_section_raw_native(0).expect("캡처");
    let resp = doc
        .change_shape_z_order_native(0, 0, 0, "front")
        .expect("front");
    let moves = parse_moves(&resp);
    assert_eq!(moves.len(), 1, "front 는 이웃 교환이 없다 — {resp}");
    assert_ne!(doc.export_hwp().expect("변경 후"), before, "변경 실재");

    doc.apply_shape_z_order_pairs_native(0, &pairs_json(&moves, "before"))
        .expect("old 복원");
    doc.restore_section_raw_native(cap).expect("raw 복원");

    let after = doc.export_hwp().expect("복원 후 export");
    let diff = before.iter().zip(after.iter()).position(|(a, b)| a != b);
    assert!(
        diff.is_none() && before.len() == after.len(),
        "front undo 수렴 @{diff:?}"
    );
}

#[test]
fn redo_reapplies_after_pairs_to_post_op_bytes() {
    let mut doc = doc_with_shapes(3);
    let cap = doc.capture_section_raw_native(0).expect("캡처");
    let resp = doc
        .change_shape_z_order_native(0, 0, 2, "back")
        .expect("back");
    let moves = parse_moves(&resp);
    let post_op = doc.export_hwp().expect("변경 직후 export");

    // undo → redo(TS execute 재실행 = 새 캡처 + after 대입)
    doc.apply_shape_z_order_pairs_native(0, &pairs_json(&moves, "before"))
        .expect("undo");
    doc.restore_section_raw_native(cap).expect("raw 복원");

    let cap2 = doc.capture_section_raw_native(0).expect("redo 캡처");
    doc.apply_shape_z_order_pairs_native(0, &pairs_json(&moves, "after"))
        .expect("redo");
    doc.restore_section_raw_native(cap2).expect("redo raw 복원");

    let redone = doc.export_hwp().expect("redo 후 export");
    let diff = post_op.iter().zip(redone.iter()).position(|(a, b)| a != b);
    assert!(
        diff.is_none() && post_op.len() == redone.len(),
        "redo 가 변경 상태를 정확히 재현 — 첫 불일치 @{diff:?}"
    );
}

#[test]
fn noop_front_reports_no_moves_and_changes_nothing() {
    let mut doc = doc_with_shapes(1);
    let z_initial = z_orders(&doc);
    let before = doc.export_hwp().expect("baseline");

    // 유일 도형은 이미 맨 앞 — 조기 반환 경로다.
    let resp = doc
        .change_shape_z_order_native(0, 0, 0, "front")
        .expect("front");
    assert!(
        !resp.contains("\"moves\""),
        "무변경은 moves 가 없다 — {resp}"
    );
    assert_eq!(z_orders(&doc), z_initial, "z 불변");

    // 캡처→복원 짝이 노이즈 없이 닫히는지도 확인(무변경 뒤 복원 전제는 Some 요구가 아니라
    // 저널 정합이다 — None 캡처의 복원도 허용된다).
    let cap = doc.capture_section_raw_native(0).expect("캡처");
    doc.restore_section_raw_native(cap).expect("복원");
    let after = doc.export_hwp().expect("이후 export");
    assert_eq!(
        before.iter().zip(after.iter()).position(|(a, b)| a != b),
        None,
        "무변경 front 는 저장 바이트에 아무 흔적도 남기지 않는다"
    );
}

#[test]
fn stale_pair_is_rejected_without_partial_application() {
    let mut doc = doc_with_shapes(2);
    let resp = doc
        .change_shape_z_order_native(0, 0, 0, "forward")
        .expect("forward");
    let mut moves = parse_moves(&resp);

    // 두 번째 항목을 범위 밖으로 오염 — 기록 이후 문서가 바뀐 상황을 흉내낸다.
    let last = moves.len() - 1;
    moves[last].1 = 999;

    let z_before_apply = z_orders(&doc);
    let err = doc
        .apply_shape_z_order_pairs_native(0, &pairs_json(&moves, "before"))
        .expect_err("오염된 pairs 는 거부돼야 한다");
    let _ = err;
    assert_eq!(
        z_orders(&doc),
        z_before_apply,
        "거부 시 부분 적용도 없어야 한다"
    );
}

#[test]
fn empty_pairs_are_a_clean_noop() {
    let mut doc = doc_with_shapes(2);
    let before = doc.export_hwp().expect("baseline");
    let resp = doc
        .apply_shape_z_order_pairs_native(0, "[]")
        .expect("빈 pairs");
    assert!(resp.contains("\"applied\":0"), "applied 0 — {resp}");
    let after = doc.export_hwp().expect("이후 export");
    assert_eq!(
        before.iter().zip(after.iter()).position(|(a, b)| a != b),
        None,
        "빈 pairs 는 passthrough 도 건드리지 않는다"
    );
}

#[test]
fn loaded_file_passthrough_some_stream_converges() {
    // 실파일 경로 — 위 테스트들은 raw_stream=None 합성 문서라 "capture(Some) →
    // 상대 연산(None 화) → old 대입 → restore(Some) → export 원본 바이트" 의
    // Some→Some 왕복을 받아치지 못했다(P3-4). from_bytes 로 내보낸 바이트를
    // 다시 열면 파서가 raw_stream=Some 을 채우므로 공개 API 만으로 실측 가능.
    let exported = doc_with_shapes(3).export_hwp().expect("파일화");
    let mut doc = HwpDocument::from_bytes(&exported).expect("재로드");
    let before = doc.export_hwp().expect("재로드 baseline");
    assert_eq!(before.len(), exported.len(), "왕복 자기동일 전제");
    assert_eq!(z_orders(&doc).len(), 3, "도형 3개 재로드");

    // TS SetZOrderCommand undo 순서 그대로 — 이번엔 캡처가 Some 이다.
    let (pi, ci) = first_shape_at(&doc);
    let cap = doc.capture_section_raw_native(0).expect("캡처");
    let resp = doc
        .change_shape_z_order_native(0, pi, ci, "forward")
        .expect("forward");
    let moves = parse_moves(&resp);
    assert_eq!(moves.len(), 2, "교환 기록 — {resp}");

    doc.apply_shape_z_order_pairs_native(0, &pairs_json(&moves, "before"))
        .expect("old 복원");
    doc.restore_section_raw_native(cap).expect("raw 복원");

    let after = doc.export_hwp().expect("복원 후 export");
    assert_eq!(before.len(), after.len(), "바이트 길이 수렴");
    let diff = before.iter().zip(after.iter()).position(|(a, b)| a != b);
    assert!(
        diff.is_none(),
        "passthrough Some 복원 후 원본 바이트 완전 수렴 — 첫 불일치 @{diff:?}"
    );
}
