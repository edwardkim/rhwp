use llm_verifier_repeat_eval::check::ValueKind;
use llm_verifier_repeat_eval::vote::{conservative_pick, VoteTally};

#[test]
fn three_way_exit_picks_worst() {
    let vals = ["0", "3", "4", "0", "3", "4"]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
    let t = VoteTally::from_values(&vals, ValueKind::Exit);
    assert!(t.is_tie);
    assert_eq!(t.plurality, "4");
}

#[test]
fn conservative_text_prefers_fail() {
    let got = conservative_pick(&["pass".into(), "fail".into()], ValueKind::Text);
    assert_eq!(got, "fail");
}

#[test]
fn counts_are_complete() {
    let vals = ["0", "0", "3", "0", "1"]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
    let t = VoteTally::from_values(&vals, ValueKind::Exit);
    assert_eq!(t.counts.get("0").copied(), Some(3));
    assert_eq!(t.counts.get("3").copied(), Some(1));
    assert_eq!(t.counts.get("1").copied(), Some(1));
    assert_eq!(t.majority.as_deref(), Some("0"));
}
