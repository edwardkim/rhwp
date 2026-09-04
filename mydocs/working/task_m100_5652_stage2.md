---
kind: working
status: done
canonical: mydocs/working/task_m100_5652_stage2.md
last_verified: 2026-08-23
---

# #5652 Stage 2 — 패처 일반화: 구간→바이트열 splice, 꼬리 삽입·삭제, 계열명·라벨 치환

- **계획서**: [`mydocs/plans/task_m100_5652.md`](../plans/archives/task_m100_5652.md) §3-3
- **브랜치**: `task5652` (`upstream/devel` `bf30bd792` 기준)

## 1. 무엇을 만들었나

| 모듈 | 책임 |
|---|---|
| `crates/rhwp-ooxml-chart/src/patch.rs` | `Splice{span, bytes, seq, addr}` 프리미티브 + 정렬 `(start, end, seq)`·커서 단일 패스 `splice()`; 공개 `ChartEdit{Value, SeriesName, AppendPoints, TruncatePoints, AppendSeries, TruncateSeries}` + `apply_chart_edits()`; `apply_value_edits()` 는 `ChartEdit::Value` 위임 래퍼(시그니처 불변); `is_safe_text` 공개; `EditTarget::Label` 이 카테고리 라벨에도 유효(`LabelNotEditable` 제거); `PatchError` += `SeriesNameNotPatchable`·`UnsafeName`·`BlockNotResizable`·`EmptyBlockRefused`·`EmptySeriesRefused`·`SeriesNotClonable`·`LengthMismatch`·`OverlappingStructureEdits`·`StructureSpanOutOfRange` |
| `tests/cases/ooxml_chart_structure_contract.rs` S2 절 | 합성 계약 17건 |
| `tests/issue_4100_chart_data_edit.rs` | `engine_patch_matches_the_spike_surgery_byte_for_byte`(독립 오라클), `category_labels_are_not_editable` → `category_labels_patch_at_the_byte_layer` 재작성 |
| 크레이트 테스트 | `category_label_edit_is_refused` → `category_label_edit_replaces_the_cache_text` 재작성(개수 불변 10) |

## 2. 결정과 근거

- **삽입·삭제·치환이 한 프리미티브** — `Splice` 의 구간이 길이 0 이면 삽입, 바이트열이 비면
  삭제. 정렬 키 `(start, end, seq)` 라 같은 앵커의 삽입 여럿은 계획 순서를 지키고, 겹침 검사
  `start < cursor` 는 B1 그대로. 재직렬화 0.
- **계열 신설 = 템플릿 조각 재스캔 + 같은 splice** — 마지막 `c:ser` 의 `element_span` 바이트
  조각을 `scan_chart_values` 로 다시 읽어(구간이 조각 기준) 내부 `Planner` 로 이름·idx/order·
  라벨·값·꼬리 증감을 적용하고, 그 결과를 템플릿 끝에 길이 0 삽입한다. `c:f`·`spPr`·확장이
  그대로 복제된다(#5447 §3-1 — 복제 `c:f` 무해). 오류는 `with_series(new_index)` 로 바깥 계열
  번호로 옮긴다.
- **최종 행 수 계약** — `AppendSeries.values` 는 같은 목록의 점 증감을 반영한 템플릿의 최종 행
  수와 같아야 한다(`LengthMismatch`). `labels: None` 이면 템플릿이 받는 라벨 편집(치환·꼬리
  증감)을 그대로 물려받아, 신설 계열이 템플릿의 최종 모양을 따른다.
- **빈 점 `<c:v/>`** — 기존 계열에서는 여전히 치환 불가(`ValueNotPatchable`), 꼬리 삭제는 가능.
  신설 계열에서만 요소째 새로 쓴다(복제본이라 최소 diff 전제가 없다).
- **기계층은 의미를 모른다** — 종류별 가드·`structure` 의도·수치 검사는 코어(S3). 여기서는
  주소·중복·겹침·안전 텍스트·구조 좌표 부재만 거부한다. 꼬리 하한(`keep == 0`)만은 패처도
  막는다 — 빈 블록/계열은 스캐너가 다시 읽지 못하는 산출이라 어느 층에서든 나가면 안 된다.
- **계열 삭제와 신설을 한 목록에 섞지 않는다** — 채번이 어긋난다. `OverlappingStructureEdits`.

## 3. 판정

| 테스트 | 고정 내용 |
|---|---|
| `category_label_edit_patches_the_cache_text` / `series_name_edit_replaces_cache_text_only` / `…_without_tx_is_refused` | 라벨·계열명 캐시 텍스트만, `c:f` 잔존 |
| `append_points_adds_pt_elements_and_recounts` / `truncate_points_removes_the_tail_and_recounts` | 꼬리 증감 + ptCount 재계산 + 산출 재스캔(idx 순차) |
| `blank_point_can_be_truncated_but_not_replaced` / `truncate_to_zero_is_refused` / `block_without_anchor_cannot_be_resized` | 경계 |
| `append_series_clones_the_last_series_with_new_idx_order_name_values` / `…_with_row_change_uses_the_final_row_count` / `…_length_mismatch_is_refused` / `same_offset_inserts_follow_plan_order` | 계열 신설 |
| `truncate_series_removes_trailing_ser_elements` / `overlapping_structure_edits_are_refused` / `unsafe_texts_are_refused_for_names_labels_and_new_points` | 계열 삭제·겹침·안전 텍스트 |
| `structure_output_rescans_and_identity_reapplies_byte_identical` / `apply_value_edits_is_an_unchanged_wrapper` | 자기정합·B1 래퍼 |
| **`engine_patch_matches_the_spike_surgery_byte_for_byte`** | 묶은세로막대형 ① 에 대해 행추가·행삭제(중간)·계열추가·계열삭제·계열명변경·라벨변경, 분산형 점추가 **7종 전부 스파이크 수술 산출과 바이트 동일** — 위치 기반 모델의 주장(계획서 §3-1) 실측 |

### 게이트 실측 (2026-08-23)

| 게이트 | 결과 |
|---|---|
| `cargo test -p rhwp-ooxml-chart --lib` | 165 passed (개수 불변) |
| `ooxml_chart_structure_contract::` | 33 passed (S1 16 + S2 17) |
| `issue_4100_chart_data_edit` | 40 passed / 2 ignored |
| `issue_4099` 10 · `issue_4098` 9 · `issue_4055` 9 · `issue_4694` 5 · `issue_3546` 2 · `issue_3547` 3 · `chart_csv_contract` 17 · `set_chart_data_contract` 4 | 전건 passed |
| fmt `--check` · suite-manifest `--prepare`→`--check` · unit-tiers `--base-ref upstream/devel`(4225 불변) · clippy `-D warnings` | 통과 |

## 4. 확인된 것

- "중간 행 삭제 = 뒤를 앞으로 당겨 쓰기 + 꼬리 삭제"가 코퍼스 형상에서 스파이크의 "요소 제거 + idx
  재번호"와 바이트 동일하다 — idx 재번호 패치가 정말 필요 없다.
- 신설 계열의 `c:f` 를 템플릿 그대로 두는 것이 스파이크 산출과 같고, 그 산출은 한컴이 정상 렌더한
  것으로 이미 판정돼 있다(#5447 원장 `묶은세로막대형-계열추가` = 반영).

## 5. 다음

S3 — 코어 `ChartEdits.structure` 의도 분기·`validate_structure`·종류별 가드 4종·`plan_edits`→`ChartEdit`·패치 후 self-check.
