# 단계4 완료보고서: TAC 표 선행 공백 inline x 반영

- **타스크**: [#146](https://github.com/edwardkim/rhwp/issues/146)
- **마일스톤**: M100
- **브랜치**: `local/task146`
- **단계**: 4 / 5 (TAC 표 선행 공백 x 좌표 보정)
- **작성일**: 2026-04-23
- **상위 문서**: `mydocs/plans/task_m100_146_v3.md`, `mydocs/plans/task_m100_146_v3_impl.md`

## 1. 수정 내역

### 1.1 소스 (`src/renderer/layout.rs`)

**(a) `tbl_inline_x` 분기 확장 (기존 분기 뒤에 is_tac 추가)**

기존 (v2 상태):
```rust
let tbl_inline_x = if let Some((ix, _)) = inline_pos {
    Some(ix)
} else if !is_tac && tbl_is_square {
    Some(col_area.x)
} else {
    None
};
```

수정 후:
```rust
let tbl_inline_x = if let Some((ix, _)) = inline_pos {
    Some(ix)
} else if !is_tac && tbl_is_square {
    Some(col_area.x)
} else if is_tac {
    let leading = composed.get(para_index)
        .map(|c| compute_tac_leading_width(c, control_index, styles))
        .unwrap_or(0.0);
    Some(col_area.x + effective_margin + leading)
} else {
    None
};
```

**(b) 헬퍼 함수 `compute_tac_leading_width` 추가 (파일 끝)**

- target TAC 이 `composed.tac_controls` 에 있으면(inline 취급) 해당 위치까지 run 폭 합산
- 없으면(block 취급, 너비 ≥ 90% seg_width) line 0 전체 run 폭 합산

### 1.2 테스트 (`src/renderer/layout/tests.rs`)

- `test_tac_leading_width_block_table_full_line`: block 취급 TAC (`tac_controls` 빈 상태)에서 "    " 4 spaces × (10 - 1.6) = 33.6 px 합산 확인 (text-align.hwp 시나리오 재현)
- `test_tac_leading_width_inline_table_partial`: inline 취급 TAC (`tac_controls` 에 pos=2 기록)에서 "ab" 까지만(=20 px) 합산되고 "가나" 는 제외 확인

## 2. 원인 재확인 (단계3 이후 추가 조사)

v3 수행계획서 작성 시 이해:
- TAC 표 문단은 pagination 에서 `PageItem::Table` 만 발행, `FullParagraph` 미발행
- 따라서 `paragraph_layout.rs` TAC 분기가 호출되지 않아 `set_inline_shape_position` 미세팅
- `layout_table_item` 에서 `inline_pos=None` → `tbl_inline_x=None` → 표가 body_left 에 붙음

추가로 발견한 사실 (단계4 디버깅 중):
- `composer.rs:137-141` 에서 TAC 표는 `is_tac_table_inline(t, seg_width, text, controls)` 가 true 일 때만 `tac_controls` 에 추가됨
- `height_measurer.rs:23-24` 는 텍스트 있을 때 `table_width < seg_width × 0.9` 조건으로 inline 여부 판정
- text-align.hwp 표: table_width 582.2 px / seg_width 642.5 px = 90.6% → **0.9 임계치 초과 → "block 취급" 분류 → `tac_controls` 비어있음**

이 때문에 `composed.tac_controls` 에서 target control을 찾지 못해 초기 구현은 leading = 0 반환. 헬퍼에 **block 취급 fallback (line 0 전체 합산)** 을 추가해 해결.

## 3. 검증 결과

### 3.1 samples/text-align.hwp 표 좌표 수렴

| 대상 | 변경 전 (v2 상태) | 변경 후 (v3) | PDF 환산 | 오차 |
|------|-----------------|------------|---------|------|
| 표 첫 셀 clip-rect x | 75.59 | **109.59** | ≈ 112.0 | **2.41 px** |
| 표 두 번째 셀 clip-rect x | 204.41 | 238.41 | — | (동일 +34 px) |

PDF 기준 ±3 px 수렴 (목표 달성).

### 3.2 자동 테스트

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | 931 passed / 14 failed (14건 기존 실패, 본 PR 무관) |
| 신규 테스트 | `test_tac_leading_width_block_table_full_line`, `test_tac_leading_width_inline_table_partial` 2건 통과 |
| `cargo test --test svg_snapshot` | 3 passed (v2 에서 업데이트한 form-002 포함 모두 통과, v3 수정으로 추가 변경 없음) |
| `cargo clippy --lib -- -D warnings` | clean |

### 3.3 시각 확인

`output/compare/text-align/svg-chrome150-v3.png` (150dpi Chrome headless) 와 `pdf-1.png` 비교:
- 표가 body_left 에서 분리되어 PDF 와 동일한 indent 로 배치됨
- 제목 + 본문 + 표 + 주석 레이아웃 전반이 PDF 와 육안 일치 (폰트 굵기/글리프 메트릭 차이는 폰트 치환 한계로 별도 범위)

## 4. svg_snapshot 영향

v3 수정은 `layout_table_item` 의 is_tac 케이스만 추가한 분기다. 기존 분기(`inline_pos` 있음, `!is_tac && tbl_is_square`, 그 외 None) 는 그대로. 따라서:

- TAC 표를 가진 golden 샘플: 현재 svg_snapshot 에 없거나, 있어도 변경 없음 (3 passed 로 확인)
- 추후 TAC 표 샘플이 golden 에 추가될 때 본 수정의 효과가 드러남

## 5. 커밋 계획

본 단계 2개 커밋:

**커밋 1 (소스 + 테스트)**
- `src/renderer/layout.rs` (tbl_inline_x 분기 + compute_tac_leading_width 헬퍼)
- `src/renderer/layout/tests.rs` (TAC leading width 테스트 2건)
- 메시지: `Task #146: TAC 표 선행 텍스트 폭을 inline x 좌표에 반영`

**커밋 2 (문서)**
- `mydocs/plans/task_m100_146_v3.md`, `task_m100_146_v3_impl.md`
- `mydocs/working/task_m100_146_stage4.md`
- 메시지: `docs: Task #146 v3 계획서 + 단계4 보고서`

## 6. 다음 단계 (단계5)

- `cargo test` 전체 재확인
- 주요 샘플 스모크 스위프 (TAC 표 포함 문서)
- v3 최종 결과보고서 (`mydocs/report/task_m100_146_report_v3.md`)
- orders 갱신
