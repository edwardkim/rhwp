# Task #539 Stage 2 완료 보고서

**제목**: prev_has_overlay_shape 가드 완화 (treat_as_char Shape 제외)
**브랜치**: `local/task539`
**이슈**: https://github.com/edwardkim/rhwp/issues/539

---

## 1. 코드 변경

`src/renderer/layout.rs:1443-1462` (prev_has_overlay_shape 검사):

```rust
Control::Shape(s) => {
    let cm = s.common();
    // [Task #539] tac=true Shape 는 paragraph 의 LINE_SEG vpos 에
    // 통합되어 누적되므로, overlay 가 vpos 에 별도 영향을 주지 않는다.
    // 따라서 prev_has_overlay_shape 가드 제외 — 그렇지 않으면
    // tac=true InFrontOfText/BehindText 글박스 호스트 paragraph
    // 직후의 vpos correction 이 skipped 되어 trailing-ls drift
    // 716 HU 가 잔존 (#539: 21_언어_기출 7p pi=146, 9p pi=182).
    if cm.treat_as_char {
        return false;
    }
    matches!(cm.text_wrap, TextWrap::InFrontOfText | TextWrap::BehindText)
        || (matches!(cm.text_wrap, TextWrap::TopAndBottom)
            && matches!(cm.vert_rel_to, VertRelTo::Para)
            && !cm.treat_as_char)
}
```

LOC: +9 / -0 (주석 + 가드 1줄).

## 2. 검증 결과

### 2.1 단위 테스트
```
test test_539_paragraph_after_overlay_shape_host       ... ok
test test_539_partial_paragraph_after_overlay_shape    ... ok
test test_537_first_answer_after_tac_table_line_spacing ... ok
```
TDD: Red → **Green** ✅. Task #537 테스트도 회귀 없음.

### 2.2 전체 테스트
```
test result: ok. 1119 passed; 0 failed; 1 ignored
```
1117 (이전) + 2 (Task #539 신규) = 1119. 회귀 0건.

### 2.3 정량 검증

**페이지 7 (pi=145 → pi=146 "르포르는...")**:
- '르' baseline y: **569.81 → 579.36** (+9.55 px = 716 HU = 1 ls)
- gap from pi=145 last line: 14.67 → **24.21 px** (IR 기대 정합) ✓

**페이지 9 (pi=181 → pi=182 "더불어 수피즘...")**:
- '더' baseline y: **333.75 → 343.29** (+9.54 px ≈ 1 ls)
- gap from pi=181 last line: 14.67 → **24.21 px** ✓

**Task #537 직접 대상 11곳**: 모두 IR 정합 유지 (페이지 2 q3 ①→② = 72.64 px ✓).

### 2.4 VPOS_CORR 변화

수정 전:
```
                                    ← pi=146 entry 누락 (skipped)
VPOS_CORR pi=147 base=716 ...        (drift 동결)
                                    ← pi=182 entry 누락 (skipped)
```

수정 후:
```
VPOS_CORR pi=146 base=0 applied=true   ← 신규 진입, drift 없음
VPOS_CORR pi=147 base=0 applied=true   ← Task #537 fix 도 같이 작동
VPOS_CORR pi=182 base=0 applied=true   ← 신규 진입
```

가드 완화로 vpos correction 이 정상 작동하며, Task #537 의 lazy_base trailing-ls 보정도 함께 효과를 내어 base=0 으로 수렴.

## 3. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` | prev_has_overlay_shape 가드 완화 (+9 / -0) |
| `mydocs/working/task_m100_539_stage2.md` | 본 보고서 |

## 4. 다음 단계 (Stage 3)

광범위 회귀 검증:
- Task #537 의 7개 핵심 샘플 동일하게 비교
- 한컴 PDF 시각 비교 권고
- 그룹 A 별도 task 분리 결정 (한컴 환경 검증 후)

## 5. 승인 요청

Stage 2 완료. Stage 3 진행 승인 요청.
