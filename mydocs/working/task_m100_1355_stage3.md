# Stage 3 완료보고서 — Task #1355 구현 (Stage 2 설계 통합)

## 설계 (Stage 2)
미주 제목 배치 직후, **흐름 전진량이 gap 이상이면**(흐름이 이미 미주 사이 gap 을 만든
경우) 제목을 흐름 위치(`y_before_vpos`)로 되돌려 gap 을 1회만 남긴다. 흐름이 gap 을
만들지 않은 경우(`prev_content_bottom_y == y_before_vpos`)는 조건 미충족으로 무영향.
단일 수식 tail 압축(`compact_*`)이 이미 처리한 경우도 제외. 전면 통일 아님(조건부 게이트).

## 구현 (`src/renderer/layout.rs`)
`compact_endnote_title_gap_already_compacted` 산출 직전에 클램프 추가:

```rust
if current_is_endnote_question_title
    && col_content.endnote_flow
    && !compacted_equation_tail_title_gap
    && !endnote_title_direct_bottom_fit
    && !endnote_title_bottom_fit_applied
    && !current_title_tail_backtracked
    && prev_endnote_title_gap_px > 0.0
    && y_offset > y_before_vpos + 4.0
{
    if let Some(prev_bottom) = prev_item_content_bottom_y {
        if (y_before_vpos - prev_bottom) >= prev_endnote_title_gap_px * 0.9 {
            y_offset = y_before_vpos;
            hcursor.vpos_page_base = None;
            hcursor.vpos_lazy_base = None;
            compacted_equation_tail_title_gap = true; // preserve 재적용 방지
        }
    }
}
```

- `compact_*` 와 동일하게 base(page/lazy) 를 null 로 리셋해 후속 아이템 재계산
- `compacted_equation_tail_title_gap=true` 로 `should_preserve_endnote_title_gap` 비활성

## 적용 범위 (전 페이지 계측)
클램프 발화 = doubling 케이스 **6건** (flow_advance≥gap & vpos_adjust push).
미발화 push 케이스는 전부 `flow_advance==0`(정상 gap 추가) → **오탐 0건**.

## 테스트
신규 `tests/issue_1355_endnote_title_gap_double.rs`:
- p18 좌측 첫 미주 제목(문30) baseline y < 350 (정정 ~336, 이중계상 시 ~362)
- 수정 비활성 시 **y=362.1 로 FAIL** 확인 → 회귀 포착 검증됨

## 검증
- `cargo test --lib`: 1618 passed, 0 failed
- `issue_1082`(미주 드리프트) 5 passed + `issue_1355` 1 passed
- clippy 경고 없음, fmt 정리(변경 파일 한정)
