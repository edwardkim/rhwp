//! Issue #505 시각 판정용 fixture HWP 생성기.
//!
//! 저작권 있는 원본 `미적분 기출문제_03.미분계수와 도함수1-1.hwp` 대신,
//! PR #507 회귀 테스트(`tests/issue_505.rs`)의 4 개 CASES+EQALIGN 중첩 수식
//! fixture (pi=151/165/196/227) 을 직접 작성한 HWP 로 패키징한다.
//!
//! 베이스 `samples/equation-lim.hwp` (1 섹션 / 1 문단 / 1 수식) 의 문단을 4 회
//! 복제하여 각 수식 스크립트만 fixture 로 교체. raw_stream/raw_ctrl_data 를
//! 비워 재직렬화를 유도한다. 메타데이터(char_count, char_shapes, line_segs,
//! para_shape_id 등) 는 베이스 그대로 보존되므로 한컴 호환성 위험 최소.
//!
//! Usage:
//!   cargo run --release --example build_issue_505_fixture
//!   ./target/release/rhwp.exe export-svg samples/issue-505-equations.hwp \
//!       -o output/svg/issue-505/

use rhwp::model::control::Control;
use rhwp::parser::parse_hwp;
use rhwp::serializer::serialize_hwp;
use std::fs;

/// (label, script) — `tests/issue_505.rs` FIXTURES 와 동일 텍스트.
const FIXTURES: &[(&str, &str)] = &[
    (
        "pi=151",
        "g(x)= {cases{f(x)&(f(x) LEQ x)#eqalign{# ``````x}&eqalign{# (f(x)`>`x)}}}",
    ),
    (
        "pi=165",
        "g(x)= {cases{{1} over {2} x ^{2}&(0 LEQ x LEQ 2)#eqalign{# ``````x}&eqalign{# (x<0~또는~x>2)}}}",
    ),
    (
        "pi=196",
        "f(x)= {cases{`x ^{3} -ax+bx&(x LEQ 1)#eqalign{# ````````````````2x+b}&eqalign{# (x>`1)}}}",
    ),
    (
        "pi=227",
        "f(x)= {cases{`x ^{3} +ax+b&(x<`1)#eqalign{# ``````````````bx+4}&eqalign{# (x GEQ 1)}}}",
    ),
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_path = "samples/equation-lim.hwp";
    let out_path = "samples/issue-505-equations.hwp";

    let bytes = fs::read(base_path)?;
    let base_doc = parse_hwp(&bytes)?;

    let s0 = base_doc.sections.first().ok_or("base has no sections")?;
    let p0 = s0
        .paragraphs
        .first()
        .ok_or("base has no paragraphs")?
        .clone();
    let template_eq_count = p0
        .controls
        .iter()
        .filter(|c| matches!(c, Control::Equation(_)))
        .count();
    if template_eq_count == 0 {
        return Err("base lacks Equation control".into());
    }
    eprintln!(
        "Base loaded: {} sections, {} paragraphs, template paragraph carries {} Equation",
        base_doc.sections.len(),
        s0.paragraphs.len(),
        template_eq_count,
    );

    let mut doc = base_doc.clone();
    let section = doc
        .sections
        .first_mut()
        .ok_or("clone has no sections")?;
    section.raw_stream = None;
    section.paragraphs.clear();

    for (label, script) in FIXTURES {
        let mut p = p0.clone();
        let mut replaced = 0usize;
        for ctrl in &mut p.controls {
            if let Control::Equation(eq) = ctrl {
                eq.script = (*script).to_string();
                eq.raw_ctrl_data = Vec::new();
                replaced += 1;
            }
        }
        if replaced == 0 {
            return Err(format!("no Equation in template paragraph (label={label})").into());
        }
        section.paragraphs.push(p);
        eprintln!(
            "  + {label}: script set ({} chars, {} equations replaced in cloned paragraph)",
            script.chars().count(),
            replaced,
        );
    }

    let out_bytes = serialize_hwp(&doc)?;
    fs::write(out_path, &out_bytes)?;
    eprintln!("Wrote {out_path} ({} bytes)", out_bytes.len());

    let reread = parse_hwp(&out_bytes)?;
    let mut found: Vec<String> = Vec::new();
    for s in &reread.sections {
        for p in &s.paragraphs {
            for c in &p.controls {
                if let Control::Equation(eq) = c {
                    found.push(eq.script.clone());
                }
            }
        }
    }
    assert_eq!(
        found.len(),
        FIXTURES.len(),
        "round-trip equation count mismatch: got {} expected {}",
        found.len(),
        FIXTURES.len()
    );
    for (i, (label, expect)) in FIXTURES.iter().enumerate() {
        assert_eq!(
            &found[i], expect,
            "round-trip script mismatch at idx {i} ({label})"
        );
        eprintln!("  ✓ {label} round-trip");
    }
    println!(
        "OK — {} (4 equation fixtures, {} bytes)",
        out_path,
        out_bytes.len()
    );
    Ok(())
}
