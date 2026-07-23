# Task M100 #2308 구현 보고서 — revision 기반 render derived state

## 결론

[#2308](https://github.com/edwardkim/rhwp/issues/2308)은 2026-07-23 최신 기준에서도 유효했고,
`issue-2308-render-normalized-derived-state` 브랜치에서 구현했다.

- 최종 기준선: `upstream/devel@cbddc1cd87084b60685da9a2b4369a4511d86173`
- Stage 2 기반: `2cf54a602`
- Stage 3 #2004 cache: `6d1a80213`
- Stage 4 #2195 overlay: `6d2e6c35d`
- Stage 5 무효화·회귀: `6438a4cfb`
- editable `Document` IR을 단일 권위 상태로 유지한다.
- deferred edit의 mutable normalized paragraph mirror를 제거했다.
- #2195 중첩 표 폭은 source clone 없이 logical-path sparse overlay로 표현한다.
- #2004 이미지 스택은 source section revision으로 검증되는 immutable `Arc` derived cache로
  유지하며 stable 입력에서는 재복제하지 않는다.

focused Rust 회귀와 source guard는 통과했다. 전체 release test, clippy, WASM, Studio/E2E는 수행
계획의 별도 승인 게이트에 따라 아직 실행하지 않았다.

## 착수 전 재판단

2026-07-23 구현 착수 전에 이슈 상태와 최신 코드를 다시 확인했다.

- 이슈는 OPEN이고 assignee는 `postmelee`였다.
- 대체하거나 경쟁하는 열린 PR은 없었다.
- 당시 최신 `devel@29b5547e`에 `render_normalized`, `section.paragraphs.clone()`,
  `refresh_render_normalized_cell_paragraph_after_edit()`가 모두 남아 있었다.
- 최초 기준선 코멘트 뒤 하루 동안 범위를 대체할 구현이 추가되지 않았다.

따라서 새 기준선 코멘트를 중복 게시하지 않고 기존
[기준선 코멘트](https://github.com/edwardkim/rhwp/issues/2308#issuecomment-5041320803)를 유지한 채
구현했다. 구현 뒤 문서 전용 upstream 커밋 3개를 포함한 `cbddc1cd8` 위로 다시 rebase했다.

## 구현

### Hyper-Waterfall 이력

승인된 수행계획의 Stage 1~6 경계를 그대로 사용했다. 각 구현 Stage는 focused 검증과
`mydocs/working/task_m100_2308_stageN.md` 완료보고서를 포함한 커밋으로 닫은 뒤 다음 Stage로
이동했다.

| Stage | 범위 | 커밋 |
| --- | --- | --- |
| 1 | 현행 계약 특성화와 RED guard | `0bd524e5b` |
| 2 | revision ledger, 명시적 path, overlay 기반 | `2cf54a602` |
| 3 | #2004 immutable revision cache | `6d1a80213` |
| 4 | #2195 sparse width overlay | `6d2e6c35d` |
| 5 | 무효화·재사용·fragment geometry 회귀 | `6438a4cfb` |
| 6 | OVR, 기술 문서, 최종 보고 | 이 보고서를 포함한 문서 커밋 |

### revision과 logical path

`RenderNormalizationState`가 다음을 소유한다.

- 문서 전체 재초기화를 나타내는 `document_epoch`
- section 구조·기하 변경을 나타내는 `section_revisions`
- 구조가 안정적인 편집을 나타내는 `path_revisions`
- #2004 immutable section projection
- #2195 `RenderNormalizationOverlay`

표 셀, 표 캡션, 도형 글상자, 그림 캡션은 각각 명시적인 `RenderPathEntry` variant다. legacy 표 캡션
sentinel은 mutation API 경계에서만 해석하며 cache key로 저장하지 않는다. control variant나 index가
source IR과 맞지 않으면 이전 state를 조용히 쓰지 않고 `RenderError`를 반환한다.

### #2195 sparse width overlay

`NestedTableWidthProjection`은 logical path, 저장 폭, effective 폭, 배율만 가진다. source `Table`과
`Cell`의 폭은 바꾸지 않는다.

`HeightMeasurer`와 `LayoutEngine`의 다음 소비부가 같은 scale을 사용한다.

- nested table 측정
- 열 폭과 colspan constraint
- cell content available width
- table/cell border geometry
- partial table과 cell content layout

logical path map이 권위 identity이고 source pointer map은 hot-path lookup용이다. overlay를 만들 때
현재 source IR에서 pointer index를 다시 만든다. path·source/effective width·pointer가 같으면 이전
projection `Arc`를 재사용하고, source path가 사라지면 이전 entry도 사라진다.

### #2004 immutable revision cache

#2004 셀 이미지 스택은 TAC flag뿐 아니라 셀 문단 수, 합성 `LINE_SEG`, composed line을 함께 바꾸는
구조 projection이다. renderer/typeset의 기존 slice 계약을 유지하면서 mutable coherence 문제를
제거하기 위해 영향 section에만 immutable paragraph/composed `Arc`를 둔다.

- 같은 source section revision이면 동일 `Arc`를 재사용한다.
- deferred edit은 projection 내부 paragraph를 직접 교체하지 않는다.
- #2004 projection이 존재하는 section만 revision을 무효화하고 source IR에서 다시 파생한다.
- 일반 section과 #2195-only section은 paragraph section clone을 만들지 않는다.

이는 이슈 본문의 “revision 기반 derived cache 또는 명시적 overlay” 중 #2004에는 derived cache,
#2195에는 overlay를 적용한 경계다.

## 완료 조건 판정

| 완료 조건 | 근거 | 판정 |
| --- | --- | --- |
| warm deferred model/tree/cursor exact | `issue_2214_page_local_repaint` 3건 | PASS |
| #2195/#2004 layout 회귀 | #1891 3건, #2004 HWP/HWPX 2건 | PASS |
| 네 편집 path stale-state 계약 | explicit path revision unit + caption deferred test | PASS |
| stable 입력 전체 clone/global clear 방지 | #2004 stable `Arc`, #2195 stable/sibling `Arc` | PASS |
| unrelated cache reuse | warm #2214 + unrelated overlay edit identity | PASS |
| mismatch 정확성 우선 fallback | source path 제거 시 projection 0, invalid mapping `RenderError` | PASS |
| source IR 무손상 | nested source table/cell width 불변 test | PASS |
| `DocumentCore: Send` | compile-time unit assertion | PASS |

## focused 검증

| 명령 | 결과 |
| --- | --- |
| `cargo test --test issue_2308_render_normalized_derived_state` | 1 passed |
| `cargo test --test issue_2308_render_normalized_guard` | 1 passed |
| `cargo test --lib render_normalization::tests` | 4 passed |
| `cargo test --lib issue_2308_` | 3 passed |
| `cargo test --lib issue2308` | 2 passed |
| `cargo test --lib issue2214_deferred_table_caption_reports_flow_change` | 1 passed |
| `cargo test --test issue_2214_page_local_repaint` | 3 passed, 72.07s |
| `cargo test --test issue_2004_cell_image_stack_pagination` | 2 passed |
| `cargo test --test issue_1195_cell_table_empty_line` | 1 passed |
| `cargo test --test issue_1891` | 3 passed |
| `cargo test --test issue_1949_giant_cell_render_perf` | 1 passed, 65.55s |
| `cargo fmt --all -- --check` | PASS |
| `git diff --check` | PASS |

## 시각·geometry 검증

`tools/object_visual_regression.py --diff-against upstream/devel`로 다음 발동 샘플의 before/after 개체
geometry를 비교했다.

- `samples/76076_regulatory_analysis.hwp`: #2195 중첩 표 width overlay
- `samples/issue2004_cell_image_stack.hwp`: #2004 이미지 스택

| 샘플 | 페이지 | 개체 | 회귀 |
| --- | ---: | ---: | ---: |
| `76076_regulatory_analysis.hwp` | 82→82 | 9→9 | 0 |
| `issue2004_cell_image_stack.hwp` | 8→8 | 0→0 | 0 |

허용 오차 ±2px에서 최종 합계 회귀는 0건이다.

첫 실행에서는 #2195 표본 33/34쪽 중첩 1×1 표 조각 높이가 각각 `+12.5px`, `-11.0px`
달라졌다. 텍스트 줄 위치는 같았지만 `nested_table_mixed_fragment_heights()`의 페이지 분할
예산이 source cell width를 overlay scale 없이 읽었다. 해당 경로와 병합 셀·row-cut·partial-table의
동일 계급 누락을 보완한 뒤 0건이 됐고, 두 fragment 높이는 통합 테스트로 고정했다.

이 자동 OVR은 한컴 최종 시각 판정을 대체하지 않으며, PR 단계의 before/after/OVL 사람 판정은
별도 승인·검토 대상으로 남는다.

## 문서

- 수행 계획: `mydocs/plans/task_m100_2308.md`
- 구현 계획: `mydocs/plans/task_m100_2308_impl.md`
- 단계별 기록: `mydocs/working/task_m100_2308_stage1.md` ~
  `mydocs/working/task_m100_2308_stage6.md`
- 장기 계약: `mydocs/tech/rendering_engine_design.md`

## 남은 게이트

focused 결과 공유 뒤 다음 실행 여부를 작업지시자에게 별도 승인받는다.

1. 전체 release test와 clippy
2. WASM build
3. Studio unit/E2E
4. 한컴 기준 before/after/OVL 최종 시각 판정
5. 원격 push, draft PR 생성, 이슈 결과 코멘트

원격 변경은 아직 수행하지 않았다.
