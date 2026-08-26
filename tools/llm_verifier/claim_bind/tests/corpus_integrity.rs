//! 코퍼스 무결성 — 유일성·라벨·한국어 주장·패딩 금지를 한 번에 본다.

use llm_verifier_claim_bind::bind::bind_row;
use llm_verifier_claim_bind::loader::{corpus_dir, load_manifest, load_shards};
use llm_verifier_claim_bind::verdict::{FailKind, Verdict};
use llm_verifier_claim_bind::REQUIRED_COORD_FIELDS;
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
    assert_eq!(man.schema_version, "v-bind.1.0");
    assert_eq!(man.axis, "claim-coords");
    assert!(
        man.record_count >= 100_000,
        "record_count {} < 100000",
        man.record_count
    );
    assert!(man.shard_count >= 8);
    assert_eq!(man.pass_count + man.fail_count, man.record_count);
    assert!(man.pass_count > 0 && man.fail_count > 0);
    assert_eq!(
        man.required_fields,
        REQUIRED_COORD_FIELDS
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>()
    );
}

#[test]
fn every_row_is_distinct_and_label_matches_recomputed_bind() {
    let rows = load_shards(&corpus_dir()).expect("shards");
    assert!(rows.len() >= 100_000);

    let mut hangul = 0usize;
    let mut pass = 0usize;
    let mut fail = 0usize;
    let mut kinds: HashSet<String> = HashSet::new();
    let mut files: HashSet<String> = HashSet::new();
    let mut quotes: HashSet<String> = HashSet::new();

    for row in &rows {
        let lower = row.claim_text.to_ascii_lowercase();
        for marker in PADDING_MARKERS {
            assert!(
                !lower.contains(marker),
                "{} looks like padding: {}",
                row.row_id,
                row.claim_text
            );
        }
        if row.claim_text.chars().any(|c| ('가'..='힣').contains(&c)) {
            hangul += 1;
        }
        if !row.claim_text.trim().is_empty() {
            assert!(
                row.claim_text.chars().count() >= 24,
                "{} claim too short to be a document sentence: {}",
                row.row_id,
                row.claim_text
            );
        }
        assert!(
            row.field_set_consistent(),
            "{} fieldSet {:?} != locator",
            row.row_id,
            row.field_set
        );

        let got = bind_row(row);
        assert_eq!(got.verdict, row.verdict, "{}", row.row_id);
        assert_eq!(got.fail_kind, row.fail_kind, "{}", row.row_id);
        assert_eq!(got.coords_present, row.coords_present, "{}", row.row_id);
        assert_eq!(got.field_set, row.field_set, "{}", row.row_id);

        match row.verdict {
            Verdict::Pass => {
                pass += 1;
                assert!(row.coords_present, "{}", row.row_id);
                for key in REQUIRED_COORD_FIELDS {
                    assert!(
                        row.field_set.iter().any(|k| k == key),
                        "{} pass missing {key}",
                        row.row_id
                    );
                }
                assert!(row.fail_kind.is_none(), "{}", row.row_id);
                assert!(row.invented_keys.is_empty(), "{}", row.row_id);
            }
            Verdict::Fail => {
                fail += 1;
                assert!(row.fail_kind.is_some(), "{}", row.row_id);
                if row.fail_kind == Some(FailKind::Unbound) {
                    assert!(!row.coords_present, "{}", row.row_id);
                }
                if row.fail_kind == Some(FailKind::IncompleteCoords) {
                    assert!(!row.coords_present, "{}", row.row_id);
                    assert!(
                        REQUIRED_COORD_FIELDS
                            .iter()
                            .any(|k| !row.field_set.iter().any(|f| f == k)),
                        "{} incomplete but fieldSet {:?}",
                        row.row_id,
                        row.field_set
                    );
                }
            }
        }
        kinds.insert(row.envelope_kind.clone());
        if let Some(f) = &row.file {
            files.insert(f.clone());
        }
        if let Some(q) = &row.quote {
            if !q.is_empty() {
                quotes.insert(q.clone());
            }
        }
    }

    assert_eq!(pass + fail, rows.len());
    assert!(
        hangul >= rows.len() * 9 / 10,
        "too few Korean claims: {hangul}"
    );
    assert!(kinds.contains("search"));
    assert!(kinds.contains("extract-data"));
    assert!(
        files.len() >= 200,
        "document file names too few: {}",
        files.len()
    );
    assert!(
        quotes.len() >= 1_000,
        "quotes too few to be distinct claims: {}",
        quotes.len()
    );
}
