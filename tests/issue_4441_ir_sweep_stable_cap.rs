use rhwp::diagnostics::ir_field_sweep::{sweep_documents, MAX_DIVERGENCE_EXAMPLES};
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::Paragraph;

const EARLY_PATH: &str = "doc_info.bullet_count";
const LATER_PATH: &str = "sections[].paragraphs[].text";

fn sweep_fixture(
    early_divergence: bool,
    later_divergences: usize,
) -> rhwp::diagnostics::ir_field_sweep::DivergenceReport {
    let paragraph_count = MAX_DIVERGENCE_EXAMPLES + 1;
    assert!(later_divergences <= paragraph_count);

    let source_paragraphs = (0..paragraph_count)
        .map(|_| Paragraph {
            text: "source".to_string(),
            ..Default::default()
        })
        .collect();
    let roundtrip_paragraphs = (0..paragraph_count)
        .map(|index| Paragraph {
            text: if index < later_divergences {
                "roundtrip".to_string()
            } else {
                "source".to_string()
            },
            ..Default::default()
        })
        .collect();

    let mut source = Document::default();
    source.sections.push(Section {
        paragraphs: source_paragraphs,
        ..Default::default()
    });

    let mut roundtrip = Document::default();
    roundtrip.doc_info.bullet_count = u32::from(early_divergence);
    roundtrip.sections.push(Section {
        paragraphs: roundtrip_paragraphs,
        ..Default::default()
    });

    sweep_documents(&source, &roundtrip).expect("synthetic sweep must fit the path budget")
}

#[test]
fn early_improvement_does_not_redistribute_saturated_later_path_count() {
    let before = sweep_fixture(true, MAX_DIVERGENCE_EXAMPLES);
    let after = sweep_fixture(false, MAX_DIVERGENCE_EXAMPLES);

    assert_eq!(before.counts().get(EARLY_PATH), Some(&1));
    assert_eq!(after.counts().get(EARLY_PATH), None);
    assert_eq!(
        before.counts().get(LATER_PATH),
        Some(&MAX_DIVERGENCE_EXAMPLES)
    );
    assert_eq!(
        after.counts().get(LATER_PATH),
        before.counts().get(LATER_PATH)
    );

    assert_eq!(before.total(), MAX_DIVERGENCE_EXAMPLES + 1);
    assert_eq!(
        before.examples().len(),
        MAX_DIVERGENCE_EXAMPLES,
        "payload examples remain bounded while counts stay complete"
    );
}

#[test]
fn later_path_regression_remains_visible_after_early_improvement() {
    let baseline = sweep_fixture(true, MAX_DIVERGENCE_EXAMPLES);
    let regressed = sweep_fixture(false, MAX_DIVERGENCE_EXAMPLES + 1);

    assert_eq!(baseline.counts().get(EARLY_PATH), Some(&1));
    assert_eq!(regressed.counts().get(EARLY_PATH), None);
    assert_eq!(
        baseline.counts().get(LATER_PATH),
        Some(&MAX_DIVERGENCE_EXAMPLES)
    );
    assert_eq!(
        regressed.counts().get(LATER_PATH),
        Some(&(MAX_DIVERGENCE_EXAMPLES + 1)),
        "a genuine later-path increase must not be hidden by the early improvement"
    );
}
