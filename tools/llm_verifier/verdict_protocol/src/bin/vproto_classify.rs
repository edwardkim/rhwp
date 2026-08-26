//! 관측 JSON(stdin 또는 파일)을 읽어 기계 판정만 낸다. rhwp CLI 가 아니다.

use llm_verifier_verdict_protocol::{classify, load_observation_bytes};
use std::env;
use std::io::{self, Read};
use std::process;

fn main() {
    let mut args = env::args().skip(1);
    let bytes = match args.next() {
        Some(path) if path != "-" => match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("io: {e}");
                process::exit(1);
            }
        },
        _ => {
            let mut buf = Vec::new();
            if let Err(e) = io::stdin().read_to_end(&mut buf) {
                eprintln!("io: {e}");
                process::exit(1);
            }
            buf
        }
    };
    if bytes.is_empty() {
        eprintln!("usage: vproto-classify [observation.json|-]");
        process::exit(2);
    }
    let obs = match load_observation_bytes(&bytes) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("usage: {e}");
            process::exit(2);
        }
    };
    let decision = classify(&obs);
    match serde_json::to_string_pretty(&decision) {
        Ok(s) => {
            println!("{s}");
            let code = match decision.machine_verdict {
                llm_verifier_verdict_protocol::MachineVerdict::Pass => 0,
                llm_verifier_verdict_protocol::MachineVerdict::IoFail => 1,
                llm_verifier_verdict_protocol::MachineVerdict::UsageFail => 2,
                llm_verifier_verdict_protocol::MachineVerdict::JudgmentFail => 3,
                llm_verifier_verdict_protocol::MachineVerdict::PageVerifyFail => 4,
                llm_verifier_verdict_protocol::MachineVerdict::Inconsistent => 3,
            };
            process::exit(code);
        }
        Err(e) => {
            eprintln!("io: {e}");
            process::exit(1);
        }
    }
}
