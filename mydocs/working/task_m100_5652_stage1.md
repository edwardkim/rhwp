---
kind: working
status: done
canonical: mydocs/working/task_m100_5652_stage1.md
last_verified: 2026-08-23
---

# #5652 Stage 1 — 스캐너 구조 좌표 확장

- **계획서**: [`mydocs/plans/task_m100_5652.md`](../plans/archives/task_m100_5652.md) §3-2
- **브랜치**: `task5652` (`upstream/devel` `bf30bd792` 기준)

## 1. 무엇을 만들었나

| 모듈 | 책임 |
|---|---|
| `crates/rhwp-ooxml-chart/src/data.rs` | 같은 스캔에서 구조 좌표를 기록 — `ChartPoint.element_span`, `PlotKind`, `PtCount`, `BlockShape{element_span, pt_count, insert_at}`, `ChartSeries.{plot, prefix, element_span, name_span, idx_span, order_span, labels_shape, values_shape}`. `SKIPPED_SUBTREES` 에 `dPt` |
| `src/document_core/commands/object_ops/chart.rs` | `same_chart_data` 에 `plot` 비교 1줄(span 류는 비교하지 않음) |
| `tests/cases/ooxml_chart_structure_contract.rs` (신규) | 합성 XML 계약 16건 |
| `tests/issue_4100_chart_data_edit.rs` Stage 9 | 코퍼스 56건 스윕 `structure_spans_slice_back_across_the_corpus` |

전부 additive 라 기존 코드는 무수정으로 컴파일된다(외부 리터럴 생성자 0건). 기존 검증
(`NoSeries`·비순차 fail-closed)과 기존 크레이트 테스트(data 17 + patch 10)는 손대지 않았다.

## 2. 결정과 근거

- **속성값 구간은 그 태그 하나의 raw 바이트 안에서만 찾는다** — quick_xml 0.41 은 속성
  오프셋을 주지 않는다. 이터레이션 read 직전·직후 위치로 태그 구간 `[pos_before, buffer_position)`
  을 잡고, 그 안에서 `val=` 를 수동 파싱한 뒤 **재슬라이스가 quick_xml 속성값과 같을 때만**
  구간을 싣는다(`attr_value_span`). 다르면 `None` → 그 블록의 개수 변경만 거부된다.
- **`End(pt)` 에서 요소 구간을 다는 조건** — `c:v` 없는 `<c:pt/>`·`<c:pt></c:pt>` 는 점을
  싣지 않으므로, `pt_start` 에 "pt 시작 시점의 블록 점 개수"를 함께 두고 개수가 늘었을 때만
  마지막 점에 `element_span` 을 단다. 그렇지 않으면 앞 점에 엉뚱한 구간이 붙는다.
- **삽입 앵커 우선순위** — 마지막 `c:pt` 요소 끝 → `c:ptCount` 요소 끝 → 캐시 닫는 태그 직전.
  `in_cache` 게이트라 `multiLvlStrCache`(캐시 목록 밖)는 앵커·ptCount 모두 `None` — 다층 카테고리
  구조 편집 거부의 근거가 스캔 결과에 그대로 실린다.
- **plot 종류는 스캐너가 기록** — 손실 파서 2차 호출은 두 좌표계 위험(계획서 §3-2). `…Chart`
  로 끝나는 요소만 plot 이고(`chart`·`chartSpace` 제외) 모르는 것은 `Other`; 콤보는 계열마다.
- **`c:idx`/`c:order` 는 계열 직속·첫 것만** — `c:dPt` 는 서브트리째 건너뛰고(코퍼스 0건, 방어),
  블록·캐시 안의 `idx` 는 무시한다.
- **테스트 배치** — 크레이트 `#[test]` 는 CI 상한(4225 = 현재값)으로 늘릴 수 없어 합성 계약은
  `tests/cases/` 신규 파일, 코퍼스 스윕은 예외 타깃 `issue_4100` 에 뒀다(계획서 §2).

## 3. 판정

| 테스트 | 고정 내용 |
|---|---|
| `pt_element_spans_wrap_each_point` / `…_are_disjoint_and_ordered` | 점 요소 구간이 `<c:pt`…`</c:pt>`, 텍스트 구간을 품고, 비중첩·문서 순서 |
| `empty_point_element_still_has_an_element_span` | `<c:v/>` 는 span None 이어도 요소 구간은 있다 |
| `series_element_spans_wrap_each_ser_and_carry_prefix` | `<c:ser>`…`</c:ser>`, prefix `c:`/`chart:` |
| `pt_count_span_reads_back_the_declared_count` / `block_element_spans_wrap_the_section` | ptCount 구간==선언값, `c:tx` 의 ptCount 비혼입, 블록 구간 |
| `insert_anchor_sits_after_the_last_point_and_before_the_cache_close` / `empty_cache_anchors_before_its_close_tag` | 앵커 위치 3단 |
| `series_name_span_reads_back_the_name` | strCache 형·리터럴형·무명 |
| `idx_and_order_spans_read_back` / `dpt_idx_does_not_override_series_idx` | 채번 자리, dPt 방어 |
| `plot_kind_follows_the_enclosing_plot_element` | 12종 + 미지→Other + 콤보 계열별 |
| `blocks_without_cache_or_ptcount_expose_none` / `multi_level_cache_is_not_an_insert_anchor` / `scatter_blocks_are_x_and_y` / `legacy_spans_still_slice_back_to_their_text` | 경계·기존 계약 |
| `structure_spans_slice_back_across_the_corpus` (issue_4100) | 코퍼스 56건: 요소 구간 모양, `c:idx`/`c:order`==위치, 이름 구간, **선언 ptCount == 실제 점 수 전건**, 앵커 == 마지막 점 끝 |

### 게이트 실측 (2026-08-23, `DEVELOPER_DIR=/Library/Developer/CommandLineTools` — Xcode 라이선스 미승인 우회)

| 게이트 | 결과 |
|---|---|
| `cargo test -p rhwp-ooxml-chart --lib` | 165 passed / 0 failed (개수 불변) |
| `regression_suite_024 ooxml_chart_structure_contract::` | 16 passed |
| `issue_4100_chart_data_edit` | 39 passed / 2 ignored (+1 신규) |
| `issue_4099` 10 · `issue_4098` 9 · `issue_4055` 9 · `issue_4694` 5 · `issue_3546` 2 · `issue_3547` 3 · `chart_csv_contract` 17 · `set_chart_data_contract` 4 | 전건 passed |
| `cargo fmt --all -- --check` | 통과 |
| `rust-test-suite-manifest.mjs --prepare` → `--check` | 통과 (`--generate` 가 아니라 `--prepare` 가 CI 경로 — 계획서 §5 게이트 문구 정정) |
| `rust-unit-test-tiers.mjs --check --base-ref upstream/devel` | 4225 tests (불변) |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |

## 4. 확인된 것

- 코퍼스 56건 전건에서 선언 `ptCount` == 실제 점 수, `c:idx`/`c:order` == 문서 위치, 앵커 뒤가
  캐시 닫는 태그 — 꼬리 증감 모델의 전제가 실데이터에서 성립한다.
- 구조 좌표를 더해도 `same_chart_data` 는 명시 비교라 ①② 오프셋 차이에 영향이 없다
  (`both_representations_carry_the_same_xml`·`matching_representations_are_patched_independently` green).

## 5. 다음

S2 — `patch.rs` `Splice` 프리미티브 + `ChartEdit`/`apply_chart_edits`, 스파이크 수술 오라클 바이트 동일.
