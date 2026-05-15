# 최종 결과 보고서: Task #898

`exam_math.hwp` 가운데 세로선 (직선 Shape) 끝과 바탕쪽 쪽번호 박스가 시각적으로 붙어 PDF (한컴 2022) 와 차이를 보이는 결함 수정.

- 이슈: https://github.com/edwardkim/rhwp/issues/898
- 마일스톤: v1.0.0 (M100)
- 브랜치: `local/task898` (← `devel`)
- 수행계획: [`mydocs/plans/task_m100_898.md`](../plans/task_m100_898.md)
- 구현계획: [`mydocs/plans/task_m100_898_impl.md`](../plans/task_m100_898_impl.md)

## 1. 결함 요약

`samples/exam_math.hwp` 페이지 1 (및 전 페이지):
- 본문 문단 0 직선 Shape (0×300mm, 페이지 중앙) 끝 y ≈ 1358 px
- 바탕쪽 표 1×3 쪽번호 박스 (`1/20`) 상단 y ≈ 1359 px
- 두 객체 사이 간격 약 1 px — PDF(약 20 px) 와 차이 → 시각적으로 붙음

## 2. 원인 (Stage 1)

`src/renderer/layout/table_layout.rs::compute_table_y_position` 의 **Paper-relative + depth=0 + wrap=TopAndBottom/BehindText/InFrontOfText** 분기에서 `outer_margin_top` 이 산식에 누락.

기존: `raw_y = v_offset` (≈ 1359.39 px)
한컴: `raw_y = v_offset + outer_margin_top` (≈ 1378.28 px)

바탕쪽 표 데이터 (exam_math.hwp 1×3 표):
- `vertical_offset` = 101954 HU = 359.55 mm
- `outer_margin_top` = **1417 HU = 5.00 mm**
- 기대 표 상단 = 364.55 mm = 1378.28 px ↔ PDF 측정 1378.1 px (0.18 px 오차)

## 3. 수정 (Stage 2)

`src/renderer/layout/table_layout.rs:1200~1205`:

```rust
let om_top_px = if matches!(vert_rel_to, VertRelTo::Paper) {
    hwpunit_to_px(table.outer_margin_top as i32, self.dpi)
} else { 0.0 };
let om_bottom_px = if matches!(vert_rel_to, VertRelTo::Paper) {
    hwpunit_to_px(table.outer_margin_bottom as i32, self.dpi)
} else { 0.0 };
let raw_y = match vert_align {
    Top | Inside  => ref_y + v_offset + caption_top_offset + om_top_px,
    Center        => ref_y + (ref_h - table_height) / 2.0 + v_offset + caption_top_offset,
    Bottom|Outside => ref_y + ref_h - table_height - v_offset + caption_top_offset - om_bottom_px,
};
```

영향 범위 한정: `vert_rel_to == Paper` 만 적용. Page/Para 기준 표는 변화 없음.

## 4. 검증 (Stage 3)

### 단위 테스트
- 신규 `tests/issue_898.rs::master_page_table_includes_outer_margin_top` 추가 — 회귀 가드
- 전체 `cargo test --release`: **1412 passed, 0 failed**

### Lint
- `cargo clippy --release --lib -- -D warnings`: **clean**

### 시각 회귀
- exam_math.hwp 20쪽: 전 페이지 표 셀 y=1378.28 일관성 ✓
- exam_kor/eng/social/science, shortcut, KTX: 오류 없음, 시각 정상
- 골든 SVG snapshot 8건 (issue_617 exam_kor 포함): 변화 없음 ✓

### 시각 확인 (페이지 1 + 5)
가운데 세로선과 쪽번호 박스 사이 명확한 여백 확보 — PDF (한컴 2022) 와 시각 일치.

## 5. 변경 파일

- `src/renderer/layout/table_layout.rs` — Paper-relative `outer_margin_top/bottom` 적용
- `tests/issue_898.rs` — 신규 회귀 가드 테스트
- `mydocs/plans/task_m100_898.md` — 수행계획서
- `mydocs/plans/task_m100_898_impl.md` — 구현계획서
- `mydocs/working/task_m100_898_stage{1,2,3}.md` — 단계별 보고서
- `mydocs/report/task_m100_898_report.md` — 최종 보고서

## 6. 회귀 차단

- `tests/issue_898.rs` 회귀 가드 — y=1359.x 위치로 되돌아가면 테스트 실패
- 골든 SVG snapshot 8건 유지

## 7. 결론

Issue #898 사용자 보고 결함 해결. 한컴 PDF 와 시각 정합 (0.18 px 오차).
회귀 0건, 영향 범위 Paper-relative 표로 한정.
