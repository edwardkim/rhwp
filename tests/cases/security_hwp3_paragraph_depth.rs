//! HWP3 문단 목록 재귀는 큰 파서 프레임에 진입하기 전에 제한한다.
//! 합성 바이너리 레코드는 HWP3 문단·제어 구조를 따르며 시각 기준이 아닌
//! 파서 자원 경계 검사에 사용한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::parser::hwp3::parse_hwp3;

fn header(count: u16) -> Vec<u8> {
    let mut bytes = vec![0; 43];
    bytes[0] = 1; // inherit paragraph shape, 31-byte representative character shape
    bytes[1..3].copy_from_slice(&count.to_le_bytes());
    bytes
}

fn document(depth: usize, controls: &[u16]) -> Vec<u8> {
    let mut bytes = vec![0; 30 + 128 + 1008 + 14 + 2];
    bytes[..23].copy_from_slice(b"HWP Document File V3.00");
    // 글꼴·스타일 목록을 비워 파서가 일반 기본값을 사용하게 한다.
    for level in 0..depth {
        let control = controls[level % controls.len()];
        bytes.extend(header(4));
        bytes.extend(control.to_le_bytes());
        bytes.extend([0; 6]); // reserved dword and closing control word
        let info_size = match control {
            15 => 8,
            16 => 10,
            17 => 14,
            _ => unreachable!(),
        };
        bytes.extend(vec![0; info_size]);
    }
    bytes.extend(header(1));
    bytes.extend(0x41u16.to_le_bytes()); // leaf text A
    for _ in 0..=depth {
        bytes.extend([0; 43]);
    }
    bytes
}

#[test]
fn depth_policy_rejects_hidden_header_and_note_recursion() {
    for controls in [&[15][..], &[16][..], &[17][..], &[15, 16, 17][..]] {
        parse_hwp3(&document(1, controls)).expect("normal nested control must parse");
        let error =
            parse_hwp3(&document(20, controls)).expect_err("excess nesting must be rejected");
        assert!(
            error.to_string().contains("paragraph nesting exceeds"),
            "wrong failure: {error}"
        );
        // 오류 후 성공으로 범위별 예산이 해제됐음을 확인한다.
        parse_hwp3(&document(1, controls)).expect("depth state restored after error");
    }
}

#[test]
fn hostile_depth_is_an_error_on_the_default_stack() {
    let error = parse_hwp3(&document(20_000, &[15, 16, 17])).expect_err("hostile depth");
    assert!(error.to_string().contains("paragraph nesting exceeds"));
}

#[test]
fn depth_budget_counts_active_lists_and_restores_after_failure() {
    // 명시적 정책 경계: 루트와 자식 목록 15개는 16개 프레임 예산에 들어간다.
    parse_hwp3(&document(15, &[15])).expect("last allowed list depth");
    assert!(parse_hwp3(&document(16, &[15])).is_err());
    let prefix = 30 + 128 + 1008 + 14 + 2;
    let shallow = document(1, &[15]);
    let mut siblings = shallow[..prefix].to_vec();
    // 각 형제는 자식 목록 하나를 소유하며 전체 문단 수는 깊이가 아니다.
    for _ in 0..1_000 {
        siblings.extend_from_slice(&shallow[prefix..shallow.len() - 43]);
    }
    siblings.extend([0; 43]);
    let result = parse_hwp3(&siblings).expect("flat siblings after rejected nesting");
    assert_eq!(result.sections[0].paragraphs.len(), 1_000);
}

#[test]
fn normal_saved_hwp3_documents_still_open() {
    for fixture in ["hwp3-sample.hwp", "hwp3-sample19.hwp"] {
        let bytes = std::fs::read(format!("samples/{fixture}")).unwrap();
        let document = parse_hwp3(&bytes).unwrap_or_else(|error| panic!("{fixture}: {error}"));
        assert!(document.sections.iter().any(|s| !s.paragraphs.is_empty()));
    }
}
