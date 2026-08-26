//! 다수결·평균 축소. 후보 순위·기준 분해가 아니다.

use llm_verifier_repeat_eval::check::CheckSpec;
use llm_verifier_repeat_eval::exit_class::ExitClass;
use llm_verifier_repeat_eval::reduce::reduce_trials;
use llm_verifier_repeat_eval::report::ReduceKind;
use llm_verifier_repeat_eval::trial::Trial;
use serde_json::json;

fn t(seed: u64, exit: u8, env: serde_json::Value) -> Trial {
    Trial {
        seed,
        exit_class: ExitClass::from_code(exit as i32).unwrap(),
        observed: String::new(),
        envelope: Some(env),
    }
}

#[test]
fn k7_majority_beats_single_flip() {
    let check = CheckSpec::envelope_bool("verify.identical");
    let mut trials = Vec::new();
    for i in 0..7 {
        let ident = i != 4;
        trials.push(t(
            i,
            if ident { 0 } else { 3 },
            json!({"verify": {"identical": ident, "diffCount": if ident { 0 } else { 1 }}}),
        ));
    }
    let r = reduce_trials("fill-a", &check, &trials);
    assert_eq!(r.k, 7);
    assert_eq!(r.votes.majority.as_deref(), Some("true"));
    assert_eq!(r.final_value.value, "true");
    assert!(r.final_value.pass);
    assert!((r.variance.disagreement - (1.0 / 7.0)).abs() < 1e-9);
}

#[test]
fn k_increase_does_not_rank_candidates() {
    let check = CheckSpec::exit_class();
    let small = vec![t(0, 0, json!({})), t(1, 3, json!({})), t(2, 0, json!({}))];
    let r3 = reduce_trials("same", &check, &small);
    let mut big = small.clone();
    for i in 3..11 {
        big.push(t(i, 0, json!({})));
    }
    let r11 = reduce_trials("same", &check, &big);
    assert_eq!(r3.final_value.value, "0");
    assert_eq!(r11.final_value.value, "0");
    assert!(r11.variance.disagreement < r3.variance.disagreement);
}

#[test]
fn mean_filled_count() {
    let check = CheckSpec::envelope_u64("filledCount");
    let trials = vec![
        t(0, 0, json!({"filledCount": 6})),
        t(1, 0, json!({"filledCount": 6})),
        t(2, 0, json!({"filledCount": 8})),
        t(3, 0, json!({"filledCount": 6})),
        t(4, 0, json!({"filledCount": 4})),
    ];
    let r = reduce_trials("ff", &check, &trials);
    assert_eq!(r.final_value.reduce, ReduceKind::Mean);
    assert!((r.final_value.numeric.unwrap() - 6.0).abs() < 1e-9);
}

#[test]
fn pass_fail_majority() {
    let check = CheckSpec::pass_fail();
    let trials = vec![
        t(0, 0, json!({"identical": true, "diffCount": 0})),
        t(1, 0, json!({"identical": true, "diffCount": 0})),
        t(2, 3, json!({"identical": false, "diffCount": 2})),
    ];
    let r = reduce_trials("ir", &check, &trials);
    assert_eq!(r.final_value.value, "pass");
}

#[test]
fn tie_is_fail_closed() {
    let check = CheckSpec::envelope_bool("identical");
    let trials = vec![
        t(0, 0, json!({"identical": true})),
        t(1, 3, json!({"identical": false})),
    ];
    let r = reduce_trials("tie", &check, &trials);
    assert!(r.votes.is_tie);
    assert_eq!(r.final_value.value, "false");
    assert!(!r.final_value.pass);
}

#[test]
fn unused_envelope_fields_do_not_split_criteria() {
    let check = CheckSpec::envelope_u64("diffCount");
    let trials = vec![
        t(
            0,
            3,
            json!({"identical": false, "diffCount": 2, "categories": {"para_count": 2}}),
        ),
        t(
            1,
            3,
            json!({"identical": false, "diffCount": 2, "categories": {"char_offsets": 2}}),
        ),
        t(
            2,
            3,
            json!({"identical": false, "diffCount": 4, "categories": {"table_cell": 4}}),
        ),
    ];
    let r = reduce_trials("ir", &check, &trials);
    assert!((r.final_value.numeric.unwrap() - (8.0 / 3.0)).abs() < 1e-9);
    assert_eq!(r.check, "diffCount");
}
