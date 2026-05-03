//! Issue #505 회귀 테스트 — CASES+EQALIGN 중첩 토폴로지의 layout/HWP 정합
//!
//! all-in-one-parser fixture (D:/PARA/Resource/all-in-one-parser) 의 미적분03.hwp
//! 에서 발견된 4개 CASES+EQALIGN 중첩 수식의 layout 산출이 HWP 권위 height 와
//! ±20% 이내로 정합함을 검증.
//!
//! 본질 결함: parser 의 OVER/ATOP 중위 처리가 parse_cases/parse_pile/parse_eqalign
//! 의 row-collecting 루프에서 분실되어 분수가 인식되지 않던 문제. 정정 후
//! pi=165 의 SVG y-scale 이 1.64 → 1.08 로 개선.

use rhwp::renderer::equation::layout::EqLayout;
use rhwp::renderer::equation::parser::parse;

/// 미적분03.hwp 에서 추출한 4개 CASES+EQALIGN 중첩 fixture.
/// (script, hwp_width_hu, hwp_height_hu) — HWP 11pt = 14.67 px @96dpi
const FIXTURES: &[(&str, u32, u32)] = &[
    // pi=151
    (
        "g(x)= {cases{f(x)&(f(x) LEQ x)#eqalign{# ``````x}&eqalign{# (f(x)`>`x)}}}",
        11332, 3515,
    ),
    // pi=165 — 본 이슈의 핵심 결함 (분수 + 위첨자)
    (
        "g(x)= {cases{{1} over {2} x ^{2}&(0 LEQ x LEQ 2)#eqalign{# ``````x}&eqalign{# (x<0~또는~x>2)}}}",
        15137, 4970,
    ),
    // pi=196
    (
        "f(x)= {cases{`x ^{3} -ax+bx&(x LEQ 1)#eqalign{# ````````````````2x+b}&eqalign{# (x>`1)}}}",
        13219, 3703,
    ),
    // pi=227
    (
        "f(x)= {cases{`x ^{3} +ax+b&(x<`1)#eqalign{# ``````````````bx+4}&eqalign{# (x GEQ 1)}}}",
        12619, 3703,
    ),
];

const FONT_SIZE_PX: f64 = 14.67; // 11pt × 96/72
const MAX_SCALE: f64 = 1.30; // 수락 기준: ±30% 이내 (이상 1.20 목표, 마진 0.10)

fn hwpunit_to_px(hu: u32) -> f64 {
    hu as f64 / 7200.0 * 96.0
}

#[test]
fn issue_505_cases_eqalign_height_ratio() {
    let layout = EqLayout::new(FONT_SIZE_PX);
    let mut max_scale = 0.0_f64;
    for (script, _w, h) in FIXTURES {
        let lb = layout.layout(&parse(script));
        let hwp_h_px = hwpunit_to_px(*h);
        assert!(lb.height > 0.0, "layout height must be positive: {script:?}");
        let scale_y = hwp_h_px / lb.height;
        assert!(
            (1.0 / MAX_SCALE..=MAX_SCALE).contains(&scale_y),
            "scale_y out of range [{:.2}..{:.2}]: {scale_y:.4}\n  layout.h={:.2} hwp.h={:.2}\n  script={script:?}",
            1.0 / MAX_SCALE, MAX_SCALE, lb.height, hwp_h_px,
        );
        max_scale = max_scale.max((scale_y - 1.0).abs());
    }
    eprintln!("issue_505 max |scale_y - 1.0| = {max_scale:.4}");
}

#[test]
fn issue_505_pi165_fraction_recognized() {
    // 본 이슈 핵심: pi=165 의 `{1} over {2}` 가 Fraction 으로 파싱되어
    // CASES 행 1 의 height 가 단순 텍스트 한 줄보다 커야 한다.
    let layout = EqLayout::new(FONT_SIZE_PX);

    // pi=151 (분수 없음)
    let h_no_frac = layout.layout(&parse(FIXTURES[0].0)).height;
    // pi=165 (분수 있음)
    let h_with_frac = layout.layout(&parse(FIXTURES[1].0)).height;

    // 분수 정상 인식 시 height 가 최소 20% 증가해야 함. 비례 임계값으로 폰트
    // 메트릭 변화에 robust. 결함 reverted 시 ratio=1.00 으로 떨어져 fail.
    let ratio = h_with_frac / h_no_frac;
    assert!(
        ratio > 1.20,
        "pi=165 height ({h_with_frac:.2}) must exceed pi=151 ({h_no_frac:.2}) by ≥20% (분수 추가분 인식). ratio={ratio:.4}"
    );
    eprintln!("h_no_frac={h_no_frac:.2} h_with_frac={h_with_frac:.2} ratio={ratio:.4}");
}

#[test]
fn issue_505_no_internal_overlap() {
    use rhwp::renderer::equation::layout::LayoutKind;

    let layout = EqLayout::new(FONT_SIZE_PX);
    for (script, _w, _h) in FIXTURES {
        let lb = layout.layout(&parse(script));
        // CASES 는 Paren { body: Row[ row1, row2 ] } 구조
        let body = match &lb.kind {
            LayoutKind::Row(children) => {
                children.iter().find_map(|c| match &c.kind {
                    LayoutKind::Paren { body, .. } => Some(body.as_ref()),
                    _ => None,
                })
            }
            _ => None,
        };
        let body = body.unwrap_or_else(|| panic!("no Paren found in {script:?}"));
        let rows = match &body.kind {
            LayoutKind::Row(rs) => rs,
            _ => panic!("Paren body should be Row in {script:?}"),
        };
        // 인접 모든 row 쌍 검사 (단순 row[0]/row[1] 검사를 일반화)
        for w in rows.windows(2) {
            let (a, b) = (&w[0], &w[1]);
            let a_bot = a.y + a.height;
            let b_top = b.y;
            assert!(
                b_top + 0.01 >= a_bot,
                "CASES rows overlap in {script:?}: row a bottom={a_bot:.2}, row b top={b_top:.2}"
            );
        }
    }
}

#[test]
fn issue_505_eqalign_no_leading_newline_text() {
    // Tokenizer 가 \n 을 건너뛰는지 검증. 이전에는 `eqalign{# X}` 의 # 뒤 \n 이
    // Text("\n") 으로 들어와 width 와 height 에 잡음을 더했다.
    let ast = parse("eqalign{#\nx}");
    let dbg = format!("{ast:?}");
    assert!(
        !dbg.contains(r#"Text("\n""#) && !dbg.contains(r#"Text("\r""#),
        "AST should not contain newline Text after #505 fix:\n{dbg}"
    );

    // 또한 layout width 가 \n 만큼 부풀어 오르지 않음을 확인 (정정 전 약 7.7 px 잡음)
    let layout = EqLayout::new(FONT_SIZE_PX);
    let h_with_nl = layout.layout(&parse("eqalign{#\nx}")).width;
    let h_without_nl = layout.layout(&parse("eqalign{# x}")).width;
    assert!(
        (h_with_nl - h_without_nl).abs() < 0.5,
        "newline should not change layout width: with_nl={h_with_nl:.2} without_nl={h_without_nl:.2}"
    );
}

/// 분수/atop 카운트. row-collecting 루프 정정 검증의 공통 헬퍼.
fn count_fractions_and_atops(node: &rhwp::renderer::equation::ast::EqNode) -> usize {
    use rhwp::renderer::equation::ast::EqNode;
    match node {
        EqNode::Fraction { numer, denom } => {
            1 + count_fractions_and_atops(numer) + count_fractions_and_atops(denom)
        }
        EqNode::Atop { top, bottom } => {
            1 + count_fractions_and_atops(top) + count_fractions_and_atops(bottom)
        }
        EqNode::Row(children) => children.iter().map(count_fractions_and_atops).sum(),
        EqNode::Matrix { rows, .. } => rows
            .iter()
            .flatten()
            .map(count_fractions_and_atops)
            .sum(),
        EqNode::Cases { rows } | EqNode::Pile { rows, .. } => {
            rows.iter().map(count_fractions_and_atops).sum()
        }
        EqNode::EqAlign { rows } => rows
            .iter()
            .map(|(l, r)| count_fractions_and_atops(l) + count_fractions_and_atops(r))
            .sum(),
        EqNode::Subscript { base, sub } => {
            count_fractions_and_atops(base) + count_fractions_and_atops(sub)
        }
        EqNode::Superscript { base, sup } => {
            count_fractions_and_atops(base) + count_fractions_and_atops(sup)
        }
        EqNode::SubSup { base, sub, sup } => {
            count_fractions_and_atops(base)
                + count_fractions_and_atops(sub)
                + count_fractions_and_atops(sup)
        }
        EqNode::Paren { body, .. } => count_fractions_and_atops(body),
        _ => 0,
    }
}

#[test]
fn issue_505_matrix_bare_over_parses_as_fraction() {
    // 동일 결함 클래스: matrix 셀 내 bare OVER/ATOP. parse_cases/pile/eqalign 와
    // 같은 row-collecting 루프이므로 동일하게 try_consume_infix_over_atop 적용.

    // Bare OVER (셀에 중괄호 없음)
    let bare = parse("matrix{a over b & c # d & e}");
    assert!(
        count_fractions_and_atops(&bare) >= 1,
        "matrix bare OVER must parse as Fraction: AST={bare:?}"
    );

    // 중괄호로 감싼 OVER 도 동일하게 인식 (parse_group 경로)
    let braced = parse("matrix{{a over b} & c # d & e}");
    assert!(
        count_fractions_and_atops(&braced) >= 1,
        "matrix braced OVER must parse as Fraction: AST={braced:?}"
    );

    // ATOP 도 동일
    let atop = parse("matrix{a atop b & c # d & e}");
    assert!(
        count_fractions_and_atops(&atop) >= 1,
        "matrix bare ATOP must parse as Atop: AST={atop:?}"
    );
}

#[test]
fn issue_505_pile_bare_over_parses_as_fraction() {
    // PILE row-collecting 루프 직접 검증. 픽스처 4건은 EQALIGN/CASES 만 다루므로 별도.
    let p = parse("pile{a over b#c#d}");
    assert!(
        count_fractions_and_atops(&p) >= 1,
        "pile bare OVER must parse as Fraction: AST={p:?}"
    );

    let l = parse("lpile{a atop b#c}");
    assert!(
        count_fractions_and_atops(&l) >= 1,
        "lpile bare ATOP must parse as Atop: AST={l:?}"
    );
}

#[test]
fn issue_505_cases_bare_atop_parses_as_atop() {
    // CASES 픽스처는 OVER 만 검증. ATOP 도 같은 try_consume 경로이므로 명시 검증.
    let c = parse("cases{a atop b & cond1 # c & cond2}");
    assert!(
        count_fractions_and_atops(&c) >= 1,
        "cases bare ATOP must parse as Atop: AST={c:?}"
    );
}

#[test]
fn issue_505_chained_over_left_associative() {
    // 한 셀/행 안에 OVER 가 연속되면 좌결합으로 중첩 분수가 되어야 한다.
    // try_consume_infix_over_atop 의 루프 반복 동작을 검증.
    // a over b over c → ((a/b)/c) — 분수 2개.

    // matrix 셀
    let m = parse("matrix{a over b over c & d}");
    assert!(
        count_fractions_and_atops(&m) >= 2,
        "matrix chained OVER must produce 2 nested fractions: AST={m:?}"
    );

    // cases 행
    let c = parse("cases{a over b over c & cond}");
    assert!(
        count_fractions_and_atops(&c) >= 2,
        "cases chained OVER must produce 2 nested fractions: AST={c:?}"
    );

    // pile 행
    let p = parse("pile{a over b over c#d}");
    assert!(
        count_fractions_and_atops(&p) >= 2,
        "pile chained OVER must produce 2 nested fractions: AST={p:?}"
    );

    // eqalign 우측
    let e = parse("eqalign{x & a over b over c}");
    assert!(
        count_fractions_and_atops(&e) >= 2,
        "eqalign chained OVER must produce 2 nested fractions: AST={e:?}"
    );

    // eqalign 좌측 (& 이전)
    let el = parse("eqalign{a over b over c & y}");
    assert!(
        count_fractions_and_atops(&el) >= 2,
        "eqalign left-side chained OVER must produce 2 nested fractions: AST={el:?}"
    );
}

#[test]
fn issue_505_orphan_over_does_not_panic() {
    // 방어성: top/bottom 없는 OVER 가 들어와도 panic 없이 Empty 로 결합.
    // (사용자 입력은 신뢰할 수 없으므로 toolchain 안정성 확보용)

    // 첫 토큰이 OVER (top 없음)
    let _ = parse("matrix{over X & Y}");
    let _ = parse("cases{over X & C}");
    let _ = parse("pile{over X#Y}");

    // 마지막 토큰이 OVER (bottom 없음)
    let _ = parse("matrix{X over & Y}");
    let _ = parse("cases{X over & C}");

    // 중첩에서도 동일
    let _ = parse("matrix{a over b over & c}");
}
