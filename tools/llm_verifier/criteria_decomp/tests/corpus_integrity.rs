//! 코퍼스 무결성 — 유일성·라벨·한국어 과업·패딩 금지.

use llm_verifier_criteria_decomp::decomp::evaluate_row;
use llm_verifier_criteria_decomp::field::{is_allowed_envelope_field, INVENTED_FIELDS};
use llm_verifier_criteria_decomp::holistic::holistic_would_hide;
use llm_verifier_criteria_decomp::loader::{corpus_dir, load_manifest, load_shards};
use llm_verifier_criteria_decomp::verdict::FailKind;
use std::collections::HashSet;

const PADDING_MARKERS: &[&str] = &[
    "lorem",
    "ipsum",
    "asdf",
    "qwerty",
    "padding",
    "foo bar",
    "xxx",
    "placeholder text",
];

#[test]
fn corpus_manifest_exists_and_meets_size_floor() {
    let dir = corpus_dir();
    let man = load_manifest(&dir).expect("manifest");
    assert_eq!(man.schema_version, "v-decomp.1.0");
    assert_eq!(man.axis, "criteria-decomp");
    assert!(
        man.record_count >= 100_000,
        "record_count {} < 100000",
        man.record_count
    );
    assert!(man.shard_count >= 8);
    assert_eq!(man.atom_pass_count + man.atom_fail_count, man.record_count);
    assert!(man.atom_pass_count > 0 && man.atom_fail_count > 0);
    assert!(man.hidden_fail_count > 0);
    assert_eq!(
        man.tuple_fields,
        [
            "task",
            "criterionId",
            "envelopeField",
            "atomPass",
            "holisticWouldHide"
        ]
    );
}

#[test]
fn every_row_is_distinct_and_label_matches_recomputed_atom() {
    let rows = load_shards(&corpus_dir()).expect("shards");
    assert!(rows.len() >= 100_000);

    let mut hangul = 0usize;
    let mut pass = 0usize;
    let mut fail = 0usize;
    let mut hidden = 0usize;
    let mut fields: HashSet<String> = HashSet::new();
    let mut commands: HashSet<String> = HashSet::new();
    let mut files: HashSet<String> = HashSet::new();
    let mut fail_kinds: HashSet<String> = HashSet::new();

    for row in &rows {
        let blob = format!("{} {}", row.task, row.criterion_id).to_ascii_lowercase();
        for marker in PADDING_MARKERS {
            assert!(
                !blob.contains(marker),
                "{} looks like padding: {}",
                row.row_id,
                row.task
            );
        }
        if row.task.chars().any(|c| ('가'..='힣').contains(&c)) {
            hangul += 1;
        }
        if !row.task.trim().is_empty() {
            assert!(
                row.task.chars().count() >= 28,
                "{} task too short: {}",
                row.row_id,
                row.task
            );
        }
        assert!(
            row.field_allowed_or_invented_fail(),
            "{} field {} neither allowed nor invented-fail",
            row.row_id,
            row.envelope_field
        );

        let got = evaluate_row(row);
        assert_eq!(got.atom_pass, row.atom_pass, "{}", row.row_id);
        assert_eq!(
            got.holistic_would_hide, row.holistic_would_hide,
            "{}",
            row.row_id
        );
        assert_eq!(got.fail_kind, row.fail_kind, "{}", row.row_id);
        assert_eq!(got.envelope_field, row.envelope_field, "{}", row.row_id);
        assert_eq!(got.criterion_id, row.criterion_id, "{}", row.row_id);

        if row.atom_pass {
            pass += 1;
            assert!(row.fail_kind.is_none(), "{}", row.row_id);
            assert!(!row.holistic_would_hide, "{}", row.row_id);
            assert!(
                is_allowed_envelope_field(&row.envelope_field),
                "{}",
                row.row_id
            );
        } else {
            fail += 1;
            assert!(row.fail_kind.is_some(), "{}", row.row_id);
            if row.holistic_would_hide {
                hidden += 1;
                assert!(
                    holistic_would_hide(false, row.bundle_pass_count, row.bundle_total),
                    "{}",
                    row.row_id
                );
            }
        }
        if row.fail_kind == Some(FailKind::InventedField) {
            assert!(
                INVENTED_FIELDS.iter().any(|k| *k == row.envelope_field)
                    || !is_allowed_envelope_field(&row.envelope_field),
                "{}",
                row.row_id
            );
            assert!(!row.holistic_would_hide, "{}", row.row_id);
        }
        fields.insert(row.envelope_field.clone());
        commands.insert(row.command.clone());
        if let Some(f) = &row.file {
            files.insert(f.clone());
        }
        if let Some(k) = row.fail_kind {
            fail_kinds.insert(k.as_str().into());
        }
    }

    assert_eq!(pass + fail, rows.len());
    assert!(
        hangul >= rows.len() * 9 / 10,
        "too few Korean tasks: {hangul}"
    );
    assert!(
        fields.len() >= 20,
        "too few distinct envelope fields: {}",
        fields.len()
    );
    assert!(
        commands.len() >= 8,
        "too few distinct commands: {}",
        commands.len()
    );
    assert!(
        files.len() >= 200,
        "document file names too few: {}",
        files.len()
    );
    assert!(
        fail_kinds.contains("invented_field"),
        "missing invented_field rows"
    );
    assert!(
        fail_kinds.contains("atom_mismatch"),
        "missing atom_mismatch rows"
    );
    assert!(hidden >= 1_000, "too few holistic-hide rows: {hidden}");
    assert!(
        !fields
            .iter()
            .any(|f| f == "bestOfN" && is_allowed_envelope_field(f)),
        "bestOfN must not be an allowed envelope field"
    );
    assert!(
        !fields
            .iter()
            .any(|f| f == "processReward" && is_allowed_envelope_field(f)),
        "processReward must not be an allowed envelope field"
    );
}
