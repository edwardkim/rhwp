# Stage C 완료보고서 — #1027: HeightCursor 구현 + 렌더러 위임(무동작)

- 타스크: #1027 / 브랜치 `local/task1027`
- 작성일: 2026-05-20
- 단계: Stage C — 공유 측정 엔진 리팩터 3단계 (inter-item VPOS 상태머신 추출 + parity)

## 1. 변경

설계서(`tech/shared_layout_measurement_engine.md`) Stage C: 렌더러의 컬럼 단위
**inter-item VPOS_CORR 상태머신**을 공유 구조체 `HeightCursor` 로 추출하고 렌더러가
위임 호출(무동작). Stage A/B 순수 함수를 결합.

### 신규 모듈 `src/renderer/height_cursor.rs`

```rust
pub(crate) struct HeightCursor {
    dpi, col_area_y, col_area_height, col_anchor_y,   // 기하
    vpos_page_base: Option<i32>,                       // #412
    vpos_lazy_base: Option<i32>,                       // #412
    prev_layout_para: Option<usize>,
    prev_item_was_partial_table: bool,                 // #991
}
fn new(dpi, col_area_y, col_area_height, col_anchor_y, vpos_page_base) -> Self
fn vpos_adjust(&mut self, y_offset, item_para, paragraphs, styles) -> f64
```

`vpos_adjust` 는 렌더러 `build_single_column` 의 보정 블록(이전 `layout.rs:2362~2504`)을
**동작 동일**하게 캡슐화: 직전 문단 overlay-shape bypass(Stage B `para_has_overlay_shape`),
분할표 bypass(#991), vpos-reset bypass, page/lazy base 산출(#412/#1022), sb 사전 차감(#643),
≤8px 백워드 + stale-table-host 클램프(Stage A `vpos_corrected_end_y`).

### 렌더러 위임 (`layout.rs`)

- 4개 컬럼 로컬(`prev_layout_para`/`vpos_page_base`/`vpos_lazy_base`/`prev_item_was_partial_table`)
  → `HeightCursor` 필드로 이전. 초기 page_base 는 `vpos_page_base_init` 로 산출 후 `new()` 주입.
- 보정 블록 ~143줄 → `y_offset = hcursor.vpos_adjust(...)` 한 줄 위임.
- 모든 write 사이트(post-jump set, prev_para 갱신, 표/Shape 후 base 무효화)는 `hcursor.*` 필드 대입.

## 2. 무동작 검증 (k-water-rfp.hwp, baseline 대비)

| 지표 | baseline | Stage C | 판정 |
|------|----------|---------|------|
| 전체 SVG md5 | `761d9dad…` | `761d9dad…` | ✅ **byte-identical** |
| 페이지 수 | 29 | 29 | ✅ 동일 |
| LAYOUT_OVERFLOW | 3 (p3/15/21, 5.1/6.5/16.5px) | 3 (동일 좌표) | ✅ diff 없음 |
| svg_snapshot | 5 pass / 3 debt(267/617/677) | 5 pass / 3 debt | ✅ 동일 |
| clippy(height_cursor) | — | 0 경고 | ✅ |
| lib 테스트 | 1308 pass | 1316 pass(+8 parity) | ✅ 무회귀 |

골든 3건(267/617/677)은 병합 시 골든=theirs 로 둔 사전 부채(Stage C 무관, Stage F 재판정 예정).

## 3. parity 단위 테스트 (8건, DPI=96 → 75 HU/px 손계산)

`height_cursor::tests` — vpos_adjust 분기 전수 검증:
- no_prev / same_para / partial_table_bypass / vpos_reset_bypass (4 bypass 경로)
- page_path_applied / page_path_sb_prededuct(#643) / lazy_path_applied_and_base_set(역산+base set)
- backward_clamp_rejected (≤8px 가드)

## 4. 다음 (Stage D)

`typeset.rs` 누적/fit 을 `HeightCursor` 의 height-only 패스로 교체(단단 우선). HeightMeasurer
per-item 높이 + 본 커서의 vpos 상태머신을 결합해 페이지네이터가 렌더러와 동일 측정 산출 →
노트(추진일정) 한컴 동일 쪽 배치 + LAYOUT_OVERFLOW ≤ 12 + 페이지수/공개 골든 무회귀.
**실제 동작 변경·고위험** — 광범위 골든 재판정 수반.
