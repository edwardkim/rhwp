//! 과업을 원자 기준으로 분해하고 각 원자를 다시 판정한다.

use crate::atom::{AtomSpec, Expected};
use crate::envelope::{read_named, Observed};
use crate::field::is_allowed_envelope_field;
use crate::holistic::holistic_would_hide;
use crate::row::DecompRow;
use crate::task::TaskBundle;
use crate::verdict::{AtomVerdict, DecompReport, FailKind};
use serde_json::Value;

/// 봉투 없이 한 원자의 완전성만 본다 (필드 허용·식별자·과업 문장).
pub fn evaluate_atom(spec: &AtomSpec, envelope: Option<&Value>) -> AtomVerdict {
    evaluate_atom_with_bundle(spec, envelope, None)
}

fn evaluate_atom_with_bundle(
    spec: &AtomSpec,
    envelope: Option<&Value>,
    bundle_counts: Option<(u64, u64)>,
) -> AtomVerdict {
    if spec.task.trim().is_empty() {
        return fail(spec, FailKind::EmptyTask);
    }
    if spec.criterion_id.trim().is_empty() {
        return fail(spec, FailKind::EmptyCriterion);
    }
    if !is_allowed_envelope_field(&spec.envelope_field) {
        return fail(spec, FailKind::InventedField);
    }

    let (atom_pass, fail_kind) = match envelope {
        None => (true, None),
        Some(env) => {
            let observed = read_named(env, &spec.envelope_field);
            if observed.is_missing()
                && !matches!(spec.expected, Expected::Absent | Expected::EmptySeq)
            {
                (false, Some(FailKind::MissingField))
            } else if spec.expected.matches(&observed) {
                (true, None)
            } else {
                (false, Some(FailKind::AtomMismatch))
            }
        }
    };

    let (pass_count, total) = bundle_counts.unwrap_or((u64::from(atom_pass), 1));
    AtomVerdict {
        criterion_id: spec.criterion_id.clone(),
        task: spec.task.clone(),
        envelope_field: spec.envelope_field.clone(),
        atom_pass,
        holistic_would_hide: holistic_would_hide(atom_pass, pass_count, total),
        fail_kind,
    }
}

fn fail(spec: &AtomSpec, kind: FailKind) -> AtomVerdict {
    AtomVerdict {
        criterion_id: spec.criterion_id.clone(),
        task: spec.task.clone(),
        envelope_field: spec.envelope_field.clone(),
        atom_pass: false,
        holistic_would_hide: false,
        fail_kind: Some(kind),
    }
}

/// 과업 묶음을 원자로 분해한다. 총점은 만들지 않는다.
pub fn decompose_bundle(bundle: &TaskBundle) -> DecompReport {
    if bundle.is_holistic_only() {
        return DecompReport {
            task: bundle.task.clone(),
            atoms: vec![AtomVerdict {
                criterion_id: String::new(),
                task: bundle.task.clone(),
                envelope_field: String::new(),
                atom_pass: false,
                holistic_would_hide: false,
                fail_kind: Some(FailKind::HolisticOnly),
            }],
            atom_pass_count: 0,
            atom_total: 0,
            hidden_fail_count: 0,
        };
    }

    let prelim: Vec<(AtomSpec, AtomVerdict)> = bundle
        .atoms
        .iter()
        .map(|spec| {
            let v = evaluate_atom(spec, bundle.envelope.as_ref());
            (spec.clone(), v)
        })
        .collect();

    let total = prelim.len() as u64;
    let pass_count = prelim.iter().filter(|(_, v)| v.atom_pass).count() as u64;

    let atoms: Vec<AtomVerdict> = prelim
        .into_iter()
        .map(|(spec, mut v)| {
            if v.fail_kind != Some(FailKind::InventedField)
                && v.fail_kind != Some(FailKind::EmptyTask)
                && v.fail_kind != Some(FailKind::EmptyCriterion)
            {
                v.holistic_would_hide = holistic_would_hide(v.atom_pass, pass_count, total);
            } else {
                v.holistic_would_hide = false;
            }
            let _ = spec;
            v
        })
        .collect();

    let hidden_fail_count = atoms.iter().filter(|a| a.holistic_would_hide).count() as u64;
    DecompReport {
        task: bundle.task.clone(),
        atoms,
        atom_pass_count: pass_count,
        atom_total: total,
        hidden_fail_count,
    }
}

/// 코퍼스 행을 다시 판정한다. 픽스처 `atomPass` 를 믿지 않는다.
pub fn evaluate_row(row: &DecompRow) -> AtomVerdict {
    if row.holistic_only {
        return AtomVerdict {
            criterion_id: row.criterion_id.clone(),
            task: row.task.clone(),
            envelope_field: row.envelope_field.clone(),
            atom_pass: false,
            holistic_would_hide: false,
            fail_kind: Some(FailKind::HolisticOnly),
        };
    }
    if row.task.trim().is_empty() {
        return AtomVerdict {
            criterion_id: row.criterion_id.clone(),
            task: row.task.clone(),
            envelope_field: row.envelope_field.clone(),
            atom_pass: false,
            holistic_would_hide: false,
            fail_kind: Some(FailKind::EmptyTask),
        };
    }
    if row.criterion_id.trim().is_empty() {
        return AtomVerdict {
            criterion_id: row.criterion_id.clone(),
            task: row.task.clone(),
            envelope_field: row.envelope_field.clone(),
            atom_pass: false,
            holistic_would_hide: false,
            fail_kind: Some(FailKind::EmptyCriterion),
        };
    }
    if !is_allowed_envelope_field(&row.envelope_field) {
        return AtomVerdict {
            criterion_id: row.criterion_id.clone(),
            task: row.task.clone(),
            envelope_field: row.envelope_field.clone(),
            atom_pass: false,
            holistic_would_hide: false,
            fail_kind: Some(FailKind::InventedField),
        };
    }
    if row.bundle_total < 1 || row.bundle_pass_count > row.bundle_total {
        return AtomVerdict {
            criterion_id: row.criterion_id.clone(),
            task: row.task.clone(),
            envelope_field: row.envelope_field.clone(),
            atom_pass: false,
            holistic_would_hide: false,
            fail_kind: Some(FailKind::BundleShape),
        };
    }

    let observed = observed_from_row(row);
    let atom_pass = row.expected.matches(&observed);
    let fail_kind = if atom_pass {
        None
    } else if observed.is_missing()
        && !matches!(row.expected, Expected::Absent | Expected::EmptySeq)
    {
        Some(FailKind::MissingField)
    } else {
        Some(FailKind::AtomMismatch)
    };
    let hide = holistic_would_hide(atom_pass, row.bundle_pass_count, row.bundle_total);
    AtomVerdict {
        criterion_id: row.criterion_id.clone(),
        task: row.task.clone(),
        envelope_field: row.envelope_field.clone(),
        atom_pass,
        holistic_would_hide: hide,
        fail_kind,
    }
}

fn observed_from_row(row: &DecompRow) -> Observed {
    match &row.observed {
        None => Observed::Missing,
        Some(Value::Null) => Observed::Null,
        Some(Value::Bool(b)) => Observed::Bool(*b),
        Some(Value::Number(n)) => {
            if let Some(u) = n.as_u64() {
                Observed::U64(u)
            } else if let Some(i) = n.as_i64() {
                Observed::I64(i)
            } else {
                Observed::Text(n.to_string())
            }
        }
        Some(Value::String(s)) => Observed::Text(s.clone()),
        Some(Value::Array(a)) => Observed::Seq(a.clone()),
        Some(Value::Object(_)) => Observed::Map,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atom::Expected;
    use serde_json::json;

    fn spec(id: &str, field: &str, expected: Expected) -> AtomSpec {
        AtomSpec {
            criterion_id: id.into(),
            task: "기획재정부 과업지시서 누름틀을 edit fill-fields --verify 로 검증한다.".into(),
            envelope_field: field.into(),
            expected,
            command: Some("edit fill-fields".into()),
        }
    }

    #[test]
    fn invented_field_fails_without_hide() {
        let v = evaluate_atom(
            &spec("C-x", "holisticScore", Expected::Bool { value: true }),
            Some(&json!({})),
        );
        assert_eq!(v.fail_kind, Some(FailKind::InventedField));
        assert!(!v.atom_pass);
        assert!(!v.holistic_would_hide);
    }

    #[test]
    fn majority_pass_hides_identical_false() {
        let bundle = TaskBundle {
            task: "행정안전부 일반기안문 누름틀 채우기 자기검증".into(),
            envelope: Some(json!({
                "filledCount": 2,
                "notFound": [],
                "ambiguous": [],
                "dryRun": false,
                "verify": {"identical": false, "diffCount": 1},
                "untrustedContent": false
            })),
            atoms: vec![
                spec("C-1", "filledCount", Expected::U64 { value: 2 }),
                spec("C-2", "notFound", Expected::EmptySeq),
                spec("C-3", "ambiguous", Expected::EmptySeq),
                spec("C-4", "verify.identical", Expected::Bool { value: true }),
                spec("C-5", "untrustedContent", Expected::Bool { value: false }),
            ],
            holistic_score: None,
        };
        let r = decompose_bundle(&bundle);
        assert_eq!(r.atom_total, 5);
        assert_eq!(r.atom_pass_count, 4);
        let ident = r
            .atoms
            .iter()
            .find(|a| a.envelope_field == "verify.identical")
            .unwrap();
        assert!(!ident.atom_pass);
        assert!(ident.holistic_would_hide);
        assert_eq!(r.hidden_fail_count, 1);
        assert!(!r.all_atoms_pass());
    }

    #[test]
    fn holistic_only_is_rejected() {
        let bundle = TaskBundle {
            task: "전체적으로 괜찮아 보인다".into(),
            atoms: Vec::new(),
            envelope: None,
            holistic_score: Some(0.91),
        };
        let r = decompose_bundle(&bundle);
        assert_eq!(r.atoms[0].fail_kind, Some(FailKind::HolisticOnly));
        assert_eq!(r.atom_total, 0);
    }

    #[test]
    fn missing_field_is_atom_fail() {
        let v = evaluate_atom(
            &spec("C-m", "identical", Expected::Bool { value: true }),
            Some(&json!({"diffCount": 0})),
        );
        assert_eq!(v.fail_kind, Some(FailKind::MissingField));
        assert!(!v.atom_pass);
    }
}
