# Task #1152 Stage 2 — 코드 패치 + 단위 검증

- 이슈: [#1152](https://github.com/edwardkim/rhwp/issues/1152)
- 브랜치: `local/task1152`
- 작성일: 2026-05-28

## 1. 변경 파일

| 파일 | 변경 | 라인 수 |
|------|------|---------|
| `src/renderer/typeset.rs` | `typeset_tac_table()` 진입부 가드 추가 | +20 |
| `tests/issue_1152_intra_para_vpos_reset.rs` | 신규 회귀 테스트 | +79 |

무관한 rustfmt diff (`cargo fmt` 실수로 인접 라인 정리된 4 군데) 는 모두 `git checkout` 으로 복원, 오직 가드 추가만 남김.

## 2. 패치 본체

```rust
// [Task #1152] 호스트 문단의 intra-paragraph vpos-reset 가드.
// empty-text host paragraph 가 N controls + N line_segs 1:1 매핑이고,
// 현재 TAC 표의 매핑 line_seg(ctrl_idx>0) 의 vpos==0 이면 HWP 가 "이 표를
// 새 페이지 상단부터" 라고 명시한 신호. fit 검사는 표 크기가 잔여 영역에
// 들어가면 통과시키지만 명시 신호를 존중하려면 fit 이전에 advance.
// 케이스: 2022년 국립국어원 업무계획.hwp pi=586 ci=1 (별첨 박스).
if !st.current_items.is_empty()
    && ctrl_idx > 0
    && para.text.is_empty()
    && para.line_segs.len() == para.controls.len()
    && para
        .line_segs
        .get(ctrl_idx)
        .map(|s| s.vertical_pos)
        .unwrap_or(-1)
        == 0
{
    st.advance_column_or_new_page();
}
```

## 3. 단위 검증 결과

### dump-pages

```
=== 페이지 32 (global_idx=31, section=0, page_num=30) ===
  body_area: x=75.6 y=94.5 w=642.5 h=933.5
  단 0 (items=1, used=915.1px)
    PartialTable   pi=586 ci=0  rows=8..12  cont=true  12x5  vpos=69196..0 [vpos-reset@line1]

=== 페이지 33 (global_idx=32, section=0, page_num=31) ===
  body_area: x=75.6 y=94.5 w=642.5 h=933.5
  단 0 (items=24, used=869.5px, hwp_used≈861.9px, diff=+7.6px)
    Table          pi=586 ci=1  1x3  635.8x38.9px  wrap=TopAndBottom tac=true  vpos=69196..0 [vpos-reset@line1]
    FullParagraph  pi=587  h=13.3 ...
    Table          pi=588 ci=0  1x2  642.5x43.5px  wrap=TopAndBottom tac=true ...
    FullParagraph  pi=590  h=21.3 ... "□ 연 혁"
    ...
```

| 지표 | 패치 전 | 패치 후 |
|------|---------|---------|
| page 32 items | 2 | **1** |
| page 32 used | 926.8px | 915.1px |
| page 33 첫 항목 | `(빈) 문단` | **`pi=586 ci=1` 별첨 박스** |
| page 33 hwp_used diff | -48.5 | **+7.6** (한컴 정합 개선) |

### 신규 회귀 테스트

```
running 1 test
test issue_1152_별첨_box_starts_page_33_not_32 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 4. 결론 / 다음 단계

- 가드 패치 적용으로 page 32 → 33 정합 한컴 PDF 와 일치.
- 신규 회귀 테스트 통과.
- 무관한 fmt diff 없음.

→ Stage 3 (회귀 + clippy + 인접 케이스 페이지 수) 로 진행.
