# Task #540 Stage 1 완료 보고서

**제목**: 가설 H2 TDD 단위테스트 + 진단
**브랜치**: `local/task540`
**이슈**: https://github.com/edwardkim/rhwp/issues/540

---

## 1. fix 위치 진단

### 1.1 RHWP_VPOS_DEBUG 결과 (페이지 2)

```
VPOS_CORR pi=46 prev_pi=45 prev_vpos=1816 prev_lh=1100 prev_ls=-440
                vpos_end=2476 base=0 col_y=209.76
                y_in=239.09 end_y=242.77 applied=true
```

- `prev_pi=45` 의 `prev_ls=-440` (60% line spacing 빈 paragraph)
- `vpos_end=2476` (pi=46 의 IR first vpos = 1816 + 1100 + (-440))
- end_y = col_y + (vpos_end - base) / 75 = 209.76 + 33.01 = 242.77
- applied=true → y_offset 가 IR vpos 위치로 강제 → 음수 ls 영향 그대로 반영

### 1.2 fix 위치 후보

**A안**: vpos correction 의 `vpos_end` 보정
- prev_pi 가 빈 paragraph + 음수 ls 인 경우, vpos_end 에 `(-prev_ls)` 만큼 더함
- 영향: vpos correction 만 변경 → 좁은 fix

**B안**: `LINE_SEG.vertical_pos` 자체 보정 (parser 단)
- 영향: IR 단계 자체에서 음수 ls 반영 제거 → 광범위

**C안**: `prev_vpos_end` 산출 시 `seg.line_spacing.max(0)` 적용
- 영향: prev_vpos_end 만 변경 → vpos correction 의 base 산출에 영향

A안 채택 (Stage 2 에서 정밀화).

## 2. TDD 통합 테스트 추가

`integration_tests.rs` 에 `test_540_empty_paragraph_negative_ls_floor` 추가.

### 2.1 테스트 내용

페이지 2 col 1 의 `[` (pi=44 [4~6]) 와 다음 본문 line (pi=46 지문 첫 글자) 의 baseline gap 측정.

기대값: 38.88 px (가설 H2: 빈 paragraph 음수 ls floor → advance = lh = 1100 HU).
버그(현재): 33.01 px (IR 음수 ls=-440 그대로 반영).

### 2.2 결과

```
test test_540_empty_paragraph_negative_ls_floor ... FAILED

test result: FAILED. 1119 passed; 1 failed; 1 ignored.
```

기존 1119 (Task #537 + #539 포함) 통과 + 신규 1 실패. TDD Red ✅.

## 3. Stage 2 fix 안

A안 정밀 구현:

```rust
// layout.rs vpos correction 의 vpos_end 산출 후
let prev_neg_ls_compensation: i32 = paragraphs.get(prev_pi)
    .and_then(|p| {
        // 빈 paragraph + 음수 ls 인 경우만 보정
        let is_empty = p.text.chars().all(|c| c.is_whitespace() || c.is_control());
        if !is_empty { return None; }
        // 모든 line_segs 의 음수 ls 합 (절댓값)
        Some(p.line_segs.iter()
            .filter_map(|s| if s.line_spacing < 0 { Some(-s.line_spacing) } else { None })
            .sum())
    }).unwrap_or(0);

let vpos_end_adjusted = vpos_end + prev_neg_ls_compensation;
```

대안:
- **간소화**: 빈 paragraph 의 음수 ls 만 검사 (일반 paragraph 의 음수 ls 보존)
- 가드: synam-001 의 음수 ls 57건 회귀 방지

## 4. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/integration_tests.rs` | TDD 테스트 추가 (+76 LOC) |
| `mydocs/plans/task_m100_540_impl.md` | 구현계획서 |
| `mydocs/working/task_m100_540_stage1.md` | 본 보고서 |

## 5. 승인 요청

Stage 1 완료. Stage 2 진행 승인 요청.
