//! HWP3 paragraph-list recursion is bounded before entering the large parser frame.
//! Synthetic binary records follow the HWP3 paragraph/control layout; these are
//! parser resource-boundary tests, not visual baselines.
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
    // Empty font/style inventories; the parser supplies its normal defaults.
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
        // Success after error proves the scoped budget was released.
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
    // Explicit policy boundary: root + 15 child lists fit the 16-frame budget.
    parse_hwp3(&document(15, &[15])).expect("last allowed list depth");
    assert!(parse_hwp3(&document(16, &[15])).is_err());
    let prefix = 30 + 128 + 1008 + 14 + 2;
    let shallow = document(1, &[15]);
    let mut siblings = shallow[..prefix].to_vec();
    // Each sibling owns one child list; total paragraph count is not depth.
    for _ in 0..1_000 {
        siblings.extend_from_slice(&shallow[prefix..shallow.len() - 43]);
    }
    siblings.extend([0; 43]);
    let result = parse_hwp3(&siblings).expect("flat siblings after rejected nesting");
    assert_eq!(result.sections[0].paragraphs.len(), 1_000);
}

#[test]
fn normal_saved_hwp3_documents_still_open() {
    for fixture in ["hwp3-sample.hwp", "hwp3-sample16.hwp", "hwp3-sample19.hwp"] {
        let bytes = std::fs::read(format!("samples/{fixture}")).unwrap();
        let document = parse_hwp3(&bytes).unwrap_or_else(|error| panic!("{fixture}: {error}"));
        assert!(document.sections.iter().any(|s| !s.paragraphs.is_empty()));
    }
}
