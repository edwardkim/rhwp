# 구현계획서 v2 — task992: 페이지 영역 밖 콘텐츠 제거

- 타스크: 로컬 task992 / 브랜치 `local/task992` (`local/devel` `f7c31f3a`에서 분기)
- 마일스톤: M100 (v1.0.0)
- 작성일: 2026-05-19
- 선행: `task_m100_992.md`(수행계획서), `task_m100_992_impl.md`(v1, 페이지 143 단독), `task_m100_992_stage1.md`(Stage 1 조사보고서)

## 0. v2 개정 사유

Stage 1 조사에서 페이지밖 콘텐츠 2건의 근본 원인이 **서로 다름**을 확인했다. 작업지시자 결정으로 task992 범위를 두 건 모두로 확대한다(페이지 171 별도 타스크 분리 안 폐기).

- 페이지 143 — `height_measurer`의 중첩 표 셀 per-cell `max` 측정 결함.
- 페이지 171 — `IN_FRONT_OF_TEXT` 표가 데코레이션으로 분류되어 페이지 분할이 아예 안 됨.

## 1. 페이지 143 — 중첩 표 셀 측정 (Stage 1 확정)

`measure_table_impl`이 *텍스트 문단 + 중첩 표를 함께 가진 셀*을 **per-cell `max`** 로 측정 → 작은 성분 누락. 두 경로:

- 경로 B `total_content_height` (line 1179): `nested_h.max(line_sum)`.
- 경로 A `row_heights`의 `content_height` (line 718~727): `last_seg_end.max(text_height)` — `last_seg_end`가 중첩 표 vpos 점프를 반영 못 함.

렌더러는 per-paragraph(문단별 `중첩표면 max(nested,line)/아니면 line`)로 **합산**한다. 페이지네이터도 동일하게 per-paragraph 합산으로 고친다. `nested_h`는 중첩 표의 `total_height`(경로 A 유래)이므로 경로 A·B를 함께 고치면 재귀적으로 정정된다. `remaining_content_for_row`의 줄 단위 스냅 경로는 `line_heights`(중첩 표 미포함)만으로 잔량을 재므로, 중첩 표 셀은 비례 경로를 쓰도록 정정한다.

계측값(pi=308 행 1): `max`=1774.4 / 성분합=2420.7 / 실제 렌더≈2695px.

## 2. 페이지 171 — 데코레이션 오분류 (Stage 1 확정)

`pi=533` = 32행×7열 표. HWPX 원본 속성:

```
textWrap="IN_FRONT_OF_TEXT" pageBreak="CELL" repeatHeader="1" rowCnt="32"
hp:pos treatAsChar="0"
```

`typeset.rs:2033~2044`: `text_wrap ∈ {InFrontOfText, BehindText}` & `col_count==1` 표를 **데코레이션(`PageItem::Shape`)** 으로 분류 → 본문 흐름 불참 + 페이지 분할 안 함(Issue #703: 워터마크/배경 표 시멘틱). 그 결과 pi=533이 한 페이지에 통째로 그려져 ≈1378px(선언 708px의 약 2배)로 본문 밖까지 넘침.

그러나 한컴 PDF(권위 자료, 2022)는 이 표를 **페이지 분할 + 머리행 반복**으로 렌더한다(PDF 168→169쪽 연속). 즉 pi=533은 워터마크가 아니라 분할되는 본문 표다. Issue #703의 "InFrontOfText/BehindText ⇒ 데코레이션 ⇒ 분할 안 함" 휴리스틱이 과도하다.

판별자: `TablePageBreak`은 `None`(나누지 않음)·`CellBreak`(셀 단위)·`RowBreak`(행 단위) 3종. 진짜 데코레이션(워터마크)은 `None`. **`page_break != None`** 인 표는 쪽 분할이 명시된 본문 표다(pi=533은 `CellBreak`). → 데코레이션 단축 분기에서 `page_break != None` 표를 제외한다.

(`pagination/engine.rs`의 동일 시멘틱 분기는 사문이므로 무관. 렌더러 `layout.rs`의 `prev_has_overlay_shape`는 Shape/Picture만 검사하므로, typeset이 정상 Table/PartialTable 항목을 내보내면 표 경로로 렌더된다.)

## 3. 단계 구성

### Stage 1 — 조사 (완료, `62eeae10`)

페이지 143·171의 근본 원인 확정. `task_m100_992_stage1.md`.

### Stage 2 — 페이지 143: 중첩 표 셀 측정 정정

- `height_measurer.rs` 경로 B(line 1155~1181): `total_content_height` 보정을 per-cell `max` → 텍스트 줄 합 + 중첩 표 높이 **합산**.
- 경로 A(line 718~727): 중첩 표 분기 `content_height`를 텍스트 + 중첩 표 합산으로 정정 → `total_height`(→`nested_h`) 재귀 정확.
- `remaining_content_for_row`(line 1419~1466): 중첩 표 셀은 줄 단위 스냅 대신 비례 경로 사용, 캡 주석 갱신.
- 단위 테스트 추가(인메모리 `Table`: 텍스트 문단 + 중첩 표 셀의 합산 검증).
- `cargo test`(해당 모듈) + `cargo clippy` 무경고 → 커밋 + `task_m100_992_stage2.md`.

### Stage 3 — 페이지 171: 데코레이션 오분류 수정

- `typeset.rs:2033`: InFrontOfText/BehindText 데코레이션 단축 분기에 `&& table.page_break == TablePageBreak::None` 가드 추가 → `CellBreak`/`RowBreak` 표는 `format_table`→`typeset_block_table` 정상 페이지네이션.
- 렌더러가 해당 표를 분할 표(`PartialTable`)로 정상 렌더하는지 확인(z-order/오버레이 회귀 점검).
- 골든 SVG 회귀 확인(Issue #703 워터마크 케이스 `test_typeset_703...` 포함 — 그 표는 `page_break == None`이므로 영향 없음 확인).
- `cargo test` + `cargo clippy` → 커밋 + `task_m100_992_stage3.md`.

### Stage 4 — 전체 검증 + 최종 보고서

- 비공개 샘플 181쪽 `export-svg` 후 전 페이지 `<text>` y > viewBox 높이 스캔 → **오버플로 0**(현재 2건).
- `dump-pages`로 pi=308·pi=533 측정·배치가 렌더와 일치하는지 재확인.
- `cargo test --release` 전체 통과 + 골든 SVG 회귀 0 + `cargo clippy --release` 무경고.
- 공개 분할 표 샘플 교차 회귀 + 페이지 수 회귀 확인.
- WASM 영향(렌더러·페이지네이터 수정) — 릴리즈 시 Docker 재빌드 필요 명시.
- `report/task_m100_992_report.md` 작성.

## 4. 영향 범위 / 리스크

- 수정 파일: `src/renderer/height_measurer.rs`(Stage 2), `src/renderer/typeset.rs`(Stage 3).
- Stage 2 회귀: 중첩 표 포함 셀의 측정이 커진다. 중첩 표 전용 셀(1x1 래퍼 unwrap 외)은 기존과 사실상 동일, 텍스트+표 혼재 셀만 증가 → 골든 SVG로 검출. 측정 증가는 분할을 보수화 → 페이지 수 +쪽 드리프트 가능.
- Stage 3 회귀: `page_break != None`인 InFrontOfText/BehindText 표가 이제 본문 흐름에 참여·분할된다. 진짜 데코레이션 워터마크는 `page_break == None`이므로 영향 없음. 골든 SVG로 검출.
- 가드: 측정·분류 정정으로 Stage 4에서 오버플로 0 미달 시, 렌더러측 페이지 경계 가드 도입을 재검토(별도 stage). 현재는 도입하지 않음.
- 제외(수행계획서 §4): 페이지 수 누적 드리프트, HWP3, 파서.

## 5. 비공개 문서 처리

재현용 HWPX/PDF는 커밋하지 않는다. 회귀 테스트는 공개 합성 픽스처 우선, 불가 시 비커밋 처리 후 보고서에 명시. `orders/` 파일은 생성·수정하지 않는다.
