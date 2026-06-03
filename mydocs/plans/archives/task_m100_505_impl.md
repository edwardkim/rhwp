# Task M100 #505 구현 계획서

수행 계획서: `task_m100_505.md`
이슈: #505

---

## Stage 1 — parser OVER/ATOP 중위 연산자 처리 + EqAlign leading `\n` 폐기

**목표**: `parse_cases` / `parse_pile` / `parse_eqalign` 의 per-row token 수집 루프에 OVER/ATOP 중위 연산자 처리 추가.

### 변경 파일

`src/renderer/equation/parser.rs`

### 변경 내용

1. **헬퍼 함수 추출** (line 104-125 의 `parse_expression` OVER/ATOP 처리 로직을 별도 메서드로):

```rust
/// OVER/ATOP 중위 연산자 처리. 현재 토큰이 OVER/ATOP 이면 children 의 마지막 요소를
/// pop 하여 분수/atop 으로 결합한다. 처리했으면 true 반환.
fn try_consume_infix_over_atop(&mut self, children: &mut Vec<EqNode>) -> bool {
    if self.current_type() != TokenType::Command {
        return false;
    }
    let val = self.current_value();
    let is_over = Self::cmd_eq(val, "OVER");
    let is_atop = Self::cmd_eq(val, "ATOP");
    if !is_over && !is_atop {
        return false;
    }
    self.pos += 1;
    let top = children.pop().unwrap_or(EqNode::Empty);
    let bottom = self.parse_element();
    children.push(if is_atop {
        EqNode::Atop { top: Box::new(top), bottom: Box::new(bottom) }
    } else {
        EqNode::Fraction { numer: Box::new(top), denom: Box::new(bottom) }
    });
    true
}
```

2. **parse_expression / parse_group 의 기존 OVER/ATOP 처리 블록을 헬퍼 호출로 대체** (DRY).

3. **parse_cases 의 else 분기에서 헬퍼 호출**:

```rust
} else {
    if self.try_consume_infix_over_atop(&mut current_row) { continue; }
    current_row.push(self.parse_element());
}
```

4. **parse_pile 동일** (line 928).

5. **parse_eqalign 의 element 수집 분기 (current_left / current_right) 동일**.

6. **EqAlign leading `\n` 폐기**: parse_eqalign 내 element 수집 시 newline (`\n`) 으로 시작하는 Text 토큰은 무시. (정확 위치는 parse_eqalign 의 element 수집 루프에 가드 추가)

### 단위 테스트 (`src/renderer/equation/parser.rs` 의 tests 모듈)

- `test_cases_with_over` — `cases{{1} over {2} & cond}` 가 Fraction 으로 파싱
- `test_pile_with_over` — `pile{{a} over {b} # c}` 가 Fraction 으로 파싱
- `test_eqalign_with_over` — `eqalign{{a} over {b} & = & c}` 가 Fraction 으로 파싱
- `test_eqalign_leading_newline_dropped` — `eqalign{# x}` 의 row 1 에 `\n` Text 없음

### 검증

- `cargo test --lib parser` 통과
- `cargo test --lib probe_pi151_vs_pi165 -- --nocapture` 측정값:
  - pi=165 layout height 가 50+ px (분수 추가분 반영)
  - scale_y < 1.40 (이상적 < 1.20)

### 산출물 보고

`mydocs/working/task_m100_505_stage1.md` — 변경 diff + 측정값 + 단위 테스트 결과.

---

## Stage 2 — 회귀 테스트 (fixture 기반 통합)

**목표**: 4 fixture script 의 layout 정합 + PR #396 회귀 0건 확인.

### 변경 파일

신규: `tests/issue_505.rs` (또는 src/renderer/equation/layout.rs 의 tests 모듈에 추가)

### 변경 내용

```rust
// fixture: 미적분 기출문제_03.미분계수와 도함수1-1.hwp
const FIXTURE_PI151: (&str, u32, u32) = (
    "g(x)= {cases{f(x)&(f(x) LEQ x)#eqalign{# ``````x}&eqalign{# (f(x)`>`x)}}}",
    11332, 3515  // HWP size (width, height) HU
);
const FIXTURE_PI165: (&str, u32, u32) = (
    "g(x)= {cases{{1} over {2} x ^{2}&(0 LEQ x LEQ 2)#eqalign{# ``````x}&eqalign{# (x<0~또는~x>2)}}}",
    15137, 4970
);
const FIXTURE_PI196: (&str, u32, u32) = (
    "f(x)= {cases{`x ^{3} -ax+bx&(x LEQ 1)#eqalign{# ````````````````2x+b}&eqalign{# (x>`1)}}}",
    /* extract from dump */
);
const FIXTURE_PI227: (&str, u32, u32) = (
    "f(x)= {cases{`x ^{3} +ax+b&(x<`1)#eqalign{# ``````````````bx+4}&eqalign{# (x GEQ 1)}}}",
    /* extract */
);

#[test]
fn issue_505_cases_eqalign_height_within_20pct() {
    let fs = 14.67;  // 11pt × 96/72
    for (script, hwp_w, hwp_h) in [FIXTURE_PI151, FIXTURE_PI165, FIXTURE_PI196, FIXTURE_PI227] {
        let lb = parse_and_layout(script, fs);
        let hwp_h_px = hwp_h as f64 / 7200.0 * 96.0;
        let scale_y = hwp_h_px / lb.height;
        assert!(scale_y < 1.40,
            "scale_y too extreme: {scale_y:.4} for script {script:?}");
    }
}

#[test]
fn issue_505_cases_no_internal_overlap() {
    // 모든 4 fixture 의 CASES rows 가 y-overlap 없음 (인접 모든 쌍)
    ...
}
```

### 검증

- `cargo test --lib --tests issue_505` 모두 통과
- `cargo test --lib test_cases_korean_no_overlap` 통과 (PR #396 회귀)
- `cargo test --lib test_korean_text_width_not_italic` 통과
- `cargo test --lib` 전체 통과 (1102+ tests)

### 산출물 보고

`mydocs/working/task_m100_505_stage2.md` — 신규 회귀 테스트 결과 + PR #396 회귀 0건 확인.

---

## Stage 3 — SVG y-scale 방어 (defensive clamp)

**목표**: equation layout 정정 후에도 잔존 갭이 있는 경우 SVG 측에서 극단 ratio 방지.

### 변경 파일

`src/renderer/svg.rs` (line 327-348)

### 변경 내용

```rust
let scale_x_raw = if eq.layout_box.width > 0.0 && node.bbox.width > 0.0 {
    node.bbox.width / eq.layout_box.width
} else { 1.0 };
let scale_y_raw = if eq.layout_box.height > 0.0 && node.bbox.height > 0.0 {
    node.bbox.height / eq.layout_box.height
} else { 1.0 };

// 극단 ratio 는 글리프 왜곡을 유발하므로 clamp.
// 1.30 초과: layout-engine 의 height 정확도 문제 가능성. clamp 후 자연 height 사용.
const EQ_SCALE_MAX: f64 = 1.30;
const EQ_SCALE_MIN: f64 = 1.0 / EQ_SCALE_MAX;
let scale_x = scale_x_raw.clamp(EQ_SCALE_MIN, EQ_SCALE_MAX);
let scale_y = scale_y_raw.clamp(EQ_SCALE_MIN, EQ_SCALE_MAX);
```

### 검증

- 페이지 5 미적분03 SVG 재출력 후 scale 추출:
  - pi=151 scale_y < 1.30
  - pi=165 scale_y < 1.30
- svg_snapshot 6/6 통과
- `cargo test --lib issue_418` 통과
- `cargo test --lib issue_501` 통과

### 산출물 보고

`mydocs/working/task_m100_505_stage3.md` — clamp 적용 전후 scale 변화 + 회귀 통과.

---

## Stage 4 — 시각 검증 + 최종 보고서

**목표**: 미적분03 p5 재출력 → 시각 결함 해소 확인 → 스크린샷 비교 + 최종 보고.

### 작업

1. `cargo build --release`
2. `./target/release/rhwp.exe export-svg "..." -p 4 -o output/diag_175_visual_after/`
3. SVG → PNG (Chrome headless)
4. 정정 전(`output/diag_175_visual/`) ↔ 정정 후 비교
5. SVG transform 분석 재실행 — 전체 35 수식의 scale 분포 확인
6. `mydocs/report/task_m100_505_report.md` 작성 — 변경 요약, 측정 비교, 스크린샷, 회귀 결과
7. 임시 모듈 `src/renderer/equation/layout_probe_505.rs` 삭제 (또는 정식 테스트로 승격)
8. `mydocs/orders/20260501.md` 업데이트
9. PR 생성은 작업지시자 승인 후

### 검증

- 시각: `(1/2) x²` squashing 해소
- 정량: pi=165 scale_y 1.64 → 1.30 이하
- 회귀: cargo test --lib + clippy + svg_snapshot 모두 통과

### 산출물 보고

`mydocs/report/task_m100_505_report.md` — 최종 결과 + before/after 스크린샷 + 모든 검증 결과.

---

## 단계 간 의존

Stage 1 → Stage 2 (Stage 1 의 단위 테스트 통과 후 통합 테스트)
Stage 2 → Stage 3 (분리 가능, 정정 효과 검증 후 방어 추가)
Stage 3 → Stage 4 (모두 정정 후 시각 검증)

---

*승인 요청: 본 구현 계획서 검토 후 Stage 1 착수 가능 여부 확인 부탁드립니다.*
