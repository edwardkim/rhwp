# Task #539 Stage 1 완료 보고서

**제목**: 정밀 진단 + TDD 단위테스트
**브랜치**: `local/task539`
**이슈**: https://github.com/edwardkim/rhwp/issues/539

---

## 1. 가설 D 검증 결과 (확정)

### 1.1 RHWP_VPOS_DEBUG 출력 분석

```
VPOS_CORR pi=145 prev_pi=144 base=0 applied=true
                                        ← pi=146 출력 누락 (vpos correction skip)
VPOS_CORR pi=147 prev_pi=146 base=716 applied=true
VPOS_CORR pi=181 prev_pi=180 base=0 applied=true
                                        ← pi=182 출력 누락
VPOS_CORR pi=183 prev_pi=182 base=0 applied=true
```

pi=146 / pi=182 는 vpos correction 분기에 진입조차 안 함 (가드에 의해 skipped).

### 1.2 가드 원인

`src/renderer/layout.rs:1443-1462` 의 `prev_has_overlay_shape` 검사:
```rust
Control::Shape(s) => {
    let cm = s.common();
    matches!(cm.text_wrap, TextWrap::InFrontOfText | TextWrap::BehindText)  // ← treat_as_char 무관 true
        || (matches!(cm.text_wrap, TextWrap::TopAndBottom)
            && matches!(cm.vert_rel_to, VertRelTo::Para)
            && !cm.treat_as_char)
}
```

dump 결과:
- pi=145 controls=1: `Shape ci=0 wrap=InFrontOfText tac=true` (글박스)
- pi=181 controls=1: `Shape ci=0 wrap=InFrontOfText tac=true`

→ `tac=true` 인데도 첫 분기 `matches!(cm.text_wrap, TextWrap::InFrontOfText | TextWrap::BehindText)` 에서 true.
→ `prev_has_overlay_shape = true`
→ vpos correction 전체 블록 skipped.

### 1.3 정량 측정 (수정 전, Stage 1 종료 시점)

**페이지 7 (pi=145 → pi=146 "르포르는...")**:
- pi=145 마지막 line baseline y ≈ 555.15 (col 0)
- pi=146 첫 line baseline y = 569.81
- gap = **14.67 px** (= 1100 HU = 1 lh)
- IR 기대 = 24.21 px (= 1816 HU = 1 lh + 1 ls)
- **drift: −9.55 px (1 ls 부족)**

**페이지 9 (pi=181 → pi=182 "더불어 수피즘...")**:
- pi=181 마지막 line baseline y = 319.08 (col 0)
- pi=182 첫 line baseline y = 333.75
- gap = **14.67 px**
- IR 기대 = 24.21 px
- **drift: −9.55 px (1 ls 부족)**

두 케이스 모두 동일 패턴 — 정확히 1 ls 만큼 짧음.

## 2. TDD 단위 테스트 추가

`src/renderer/layout/integration_tests.rs` 에 두 통합 테스트 추가:

1. `test_539_paragraph_after_overlay_shape_host`: 페이지 7 pi=145 → pi=146 gap == 24.21 px
2. `test_539_partial_paragraph_after_overlay_shape`: 페이지 9 pi=181 → pi=182 gap == 24.21 px

```
test renderer::layout::integration_tests::tests::test_539_paragraph_after_overlay_shape_host ... FAILED
test renderer::layout::integration_tests::tests::test_539_partial_paragraph_after_overlay_shape ... FAILED

test result: FAILED. 1117 passed; 2 failed; 1 ignored
```

기존 1117 (Task #537 포함) 통과 + 신규 2건 실패. TDD: Red ✅.

## 3. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/integration_tests.rs` | 두 통합 테스트 추가 (+155 LOC) |
| `mydocs/plans/task_m100_539.md` | 수행계획서 |
| `mydocs/plans/task_m100_539_impl.md` | 구현계획서 |
| `mydocs/working/task_m100_539_stage1.md` | 본 보고서 |

## 4. Stage 2 수정안 (확정)

`layout.rs:1443-1462` 의 `prev_has_overlay_shape` 검사를 다음과 같이 변경:

```rust
let prev_has_overlay_shape = paragraphs.get(prev_pi).map(|p| {
    use crate::model::shape::{TextWrap, VertRelTo};
    p.controls.iter().any(|c| match c {
        Control::Shape(s) => {
            let cm = s.common();
            // [Task #539] tac=true Shape 는 vpos 누적에 통합 → overlay 가드 제외
            if cm.treat_as_char {
                return false;
            }
            matches!(cm.text_wrap, TextWrap::InFrontOfText | TextWrap::BehindText)
                || (matches!(cm.text_wrap, TextWrap::TopAndBottom)
                    && matches!(cm.vert_rel_to, VertRelTo::Para)
                    && !cm.treat_as_char)
        }
        Control::Picture(pic) => { /* 동일 로직 */ }
        _ => false,
    })
}).unwrap_or(false);
```

근거: `treat_as_char=true` Shape 는 paragraph 의 line_seg vpos 에 통합 → overlay 의 추가 vpos 누적 위험 없음. 본 가드의 본래 의도(InFrontOfText/BehindText 의 별도 vpos 누적으로 인한 base 산출 오류 방지)와 무관.

## 5. 승인 요청

Stage 1 완료. Stage 2 진행 승인 요청.
