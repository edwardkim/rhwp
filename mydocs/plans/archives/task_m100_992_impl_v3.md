# 구현계획서 v3 — task992: 페이지 영역 밖 콘텐츠 제거

- 타스크: 로컬 task992 / 브랜치 `local/task992`
- 마일스톤: M100 (v1.0.0)
- 작성일: 2026-05-19
- 선행: `task_m100_992.md`(수행계획서), `task_m100_992_impl.md`(v1)·`_impl_v2.md`(v2), 단계보고서 stage1~3.

## 0. v3 개정 사유

Stage 1~3 완료 후 오버플로 4건 → 3건(페이지 123·144·176). Stage 3가 페이지 171(데코레이션 오분류)을 제거했고, 남은 3건은 **모두 동일 원인** — 분할 표 연속분의 높이를 페이지네이터와 렌더러가 **서로 다른 코드 경로**로 계산해 어긋난다.

- 페이지네이터: `height_measurer.rs`의 `measure_table_impl` / `MeasuredCell` / `remaining_content_for_row`.
- 렌더러: `layout.rs`의 `LayoutEngine` — `resolve_row_heights` / `calc_cell_paragraphs_content_height` / `calc_nested_table_height` / `calc_cell_remaining_content_height`.

두 경로가 같은 셀 높이를 다르게 재서(중첩 표 ~20%, 텍스트 ~10% 차이), 페이지네이터가 과소 측정 → 추가 분할 누락 → 렌더러가 본문 밖까지 그림.

Stage 2에서 측정값을 키우는 *보정*을 시도했으나 비-TAC 중첩 표(호스트 줄 line_height에 표 높이가 이미 반영)에서 이중 가산 회귀(`test_634`)가 났다. **보정이 아니라 측정 경로 통일**이 필요하다 — 작업지시자 승인.

대상 표(페이지 123·144·176): `pi=308`(중첩 표 다수), `pi=108`/`pi=111`/`pi=547`/`pi=550`(중첩 표 없는 대형 다행 표) — 통일은 중첩·일반 셀 모두 포함해야 한다.

## 1. 목표

`height_measurer`의 분할 표 셀 높이 측정을 렌더러 `LayoutEngine`의 측정 함수로 **통일**한다. 페이지네이터와 렌더러가 정의상 같은 값을 내므로 분할 표 연속분이 페이지 경계를 넘지 않는다.

## 2. 설계

`LayoutEngine`의 셀 높이 함수는 `dpi`만 의존하며 이미 구현·검증되어 있다.

- `calc_cell_paragraphs_content_height(paras, styles, inner_width)` — 셀 문단 콘텐츠 높이(`resolve_row_heights`가 행 높이 계산에 사용).
- `calc_cell_remaining_content_height(cell, styles, content_offset)` — content_offset 이후 셀 잔여 콘텐츠 높이. per-paragraph로 `중첩표면 max(calc_nested_table_height, 줄높이)/아니면 줄높이`를 합산. `(total − content_offset)` 반환. 분할 표 연속분 측정용으로 *설계되었으나 현재 미사용(사문)*.

### 변경

1. `HeightMeasurer`가 내부에 `LayoutEngine`(dpi) 보유.
2. `measure_table_impl`의 셀 콘텐츠 높이(경로 A `content_height`)를 `LayoutEngine::calc_cell_paragraphs_content_height`로 대체 → `row_heights`가 `resolve_row_heights`와 정의상 일치.
3. `MeasuredCell.total_content_height`를 `calc_cell_remaining_content_height(cell, styles, 0.0)`로 산출 → 렌더러 분할 렌더와 동일 기준.
4. `remaining_content_for_row(row, content_offset)`: 셀 잔여 콘텐츠를 `total_content_height − content_offset`(= `calc_cell_remaining_content_height`와 동일식)로 계산. 줄 단위 스냅·#362 캡 등 기존 분기는 통일 기준에 맞게 정리.
5. `measure_table_impl`의 중첩 표 재귀(`measure_table`)와 `total_height`도 동일 기준으로 정합 — `nested_h`가 렌더러 `calc_nested_table_height`와 일치하도록.

### 호환

`resolve_row_heights`는 `MeasuredTable`가 주어지면 `mt.row_heights`를 그대로 반환한다. 통일 후 `mt.row_heights`는 `resolve_row_heights`의 자체 계산과 같아져, 렌더러·페이지네이터가 완전 정합한다.

## 3. 단계 구성

### Stage 1~3 — 완료

조사 / 페이지 143 측정 시도·진단 / 페이지 171 데코레이션 수정(`a4c7e823`).

### Stage 4 — 측정 경로 통일

- 위 §2 변경 적용. `height_measurer.rs` 중심, 필요 시 `LayoutEngine` 측정 함수 가시성(`pub(crate)`) 조정.
- `cargo test --release` 전체 + `cargo clippy` 수행. 골든 SVG 회귀가 나면 각 변경 페이지를 한컴 2022 PDF와 대조해 **정정인지 회귀인지 판정** — 정정이면 골든 갱신, 회귀면 원인 교정.
- 단계보고서 + 커밋.

### Stage 5 — 전체 검증 + 최종 보고서

- 비공개 샘플 184쪽 `export-svg` 후 전 페이지 `<text>` y > viewBox 높이 스캔 → **오버플로 0**(현재 3건).
- `dump-pages`로 pi=308·108·111 측정·배치가 렌더와 일치하는지 확인.
- `cargo test --release` 전체 통과 + `cargo clippy` 무경고.
- 공개 분할 표 샘플 교차 회귀 + 페이지 수 회귀 확인.
- WASM 영향(렌더러·페이지네이터) — 릴리즈 시 Docker 재빌드 명시.
- `report/task_m100_992_report.md` 작성.

## 4. 리스크

- 측정 경로 통일은 분할 표·중첩 표를 포함한 문서의 페이지네이션을 바꿀 수 있다 → 골든 SVG 회귀 가능. Stage 4에서 변경 페이지를 PDF 권위 자료와 대조해 판정한다(통일된 값이 렌더러 실제와 일치하므로 대부분 정정 방향).
- 페이지 수 보수적 증가 가능(과소 측정 해소). 수행계획서 §4의 "페이지 수 누적 드리프트"와는 별개 — 본 변경은 오버플로 제거가 목적.
- 제외(수행계획서 §4): HWP3, 파서, 폰트 메트릭 자체의 한컴 불일치.

## 5. 비공개 문서 처리

재현용 HWPX/PDF는 커밋하지 않는다. 회귀 테스트는 공개 골든 SVG 우선. `orders/` 파일은 생성·수정하지 않는다.
