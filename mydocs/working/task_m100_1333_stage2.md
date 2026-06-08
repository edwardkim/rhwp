# Stage 2 완료보고서 — 코드 수정 + 검증 (M100 #1333)

구현계획서: `mydocs/plans/task_m100_1333_impl.md`
Stage 1: `mydocs/working/task_m100_1333_stage1.md`

## 1. 수정 내용

`src/renderer/layout.rs` `build_columns` — zone 구분선 emit 을 페이지 단위
`has_zone_specific_layout` 로 게이트하여 page-level `build_column_separators`
(layout.rs:1139 게이트) 와 정확히 배타 관계로 만든다.

### (1) 함수 진입부에 페이지 술어 계산 추가 (layout.rs:2492 부근)

```rust
let has_zone_specific_layout = page_content
    .column_contents
    .iter()
    .any(|cc| cc.zone_layout.is_some());
```

### (2) emit 기록 조건에 술어 추가 (layout.rs:2616 부근)

```rust
if has_zone_specific_layout
    && zone_layout.column_areas.len() >= 2
    && zone_layout.separator_type > 0
{
    prev_zone_layout_for_sep = Some(zone_layout.clone());
    prev_zone_sep_y_start = current_zone_start_y.max(zone_layout.body_area.y);
} else {
    prev_zone_layout_for_sep = None;
}
```

> `zone_layout = col_content.zone_layout.unwrap_or(layout)` (layout.rs:2531) 의 page
> layout 폴백으로 인해, zone-specific layout 이 없는 페이지(초기 단정의·연속 페이지)
> 에서도 emit 되어 page-level 과 이중 렌더되던 결함을 차단.

## 2. 검증 결과

### 2.1 대상 문서 `3-09월_교육_통합_2024-구분선아래20구분선위20.hwp` (HWP)

- 23쪽 전부 단 구분선(x=396.9) **세로선 1개** (이전 2개 → 1개, 중복 제거).
- 남은 선 y=90.7→1092.3 (**body 전체 높이**) — PDF 정답지(한글 2022) 와 일치.
- 좌우 쪽 테두리(x=26.5/767.3) 정상 유지.

### 2.2 HWPX 쌍 `...구분선아래20구분선위20.hwpx`

- 동일하게 x=396.9 단일 전체높이 선(90.7→1092.3). 정상.

### 2.3 회귀 — shortcut.hwp (`samples/basic/shortcut.hwp`, #874)

수정 전후 단 구분선 동일 (페이지별 1,1,2,1,1,2,1 — 총 9개 전수 보존).
모두 zone emit(zone_layout=Some) 출처로, 게이트 영향 없음.

### 2.4 테스트 / clippy

- `cargo test --release`: **2107 passed, 0 failed** (issue_874 포함).
- `cargo clippy --release`: 경고/에러 없음 (exit 0).

## 3. 다음 단계

Stage 3 — 다단 보유 추가 샘플 회귀 점검 + 최종 결과보고서 작성.
