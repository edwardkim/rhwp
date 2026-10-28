---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-07
---

# Task #3820 Stage 46 — PR #4122 통합 리베이스 판정

## 목적

`samples/basic/issue2007_nested_cell_pagination_42065.hwp`의 기존 보정 브랜치를
PR #4122가 포함된 최신 `upstream/devel` 위로 리베이스하되, 두 구현 중 페이지 소유권과
PDF paint가 더 정확한 계약을 각각 구분해 보존한다.

## 기준점

- 리베이스 전 작업 head: `405a8e4c7`
- 리베이스 기준 `upstream/devel`: `06f8ebcca`
- PR #4122 merge commit: `accebdb20`
- 리베이스 후 최초 head: `00f87b4a6`

리베이스 전 `405a8e4c7`에서는 전체 integration gate가 5,304 passed / 0 failed /
28 ignored로 통과했다. 이 결과는 충돌 해소 전 기존 보정의 기준이며, 리베이스 후 결과를
대신하지 않는다.

## 직접 비교 결과

PR #4122의 자식 `RowCut` 재귀 투영은 source 페이지 소유권의 기준으로 더 정확하다.
리베이스 전 바이너리의 issue2007 p3 RenderTree에는 PDF p2 소유 문구인
`1. 출석요구 및 진술청취 또는 진술서 제출 요구`가 반복되고 SVG clip으로만 숨겨졌다.
PR #4122는 `NestedTableCut`과 `recursive_cut`을 partial renderer까지 전달해 그 문구를
다음 쪽 RenderTree에 재방출하지 않는다. 또한 #2430 p16의 저장 빈 Enter 간격은 기존
브랜치가 약 0.9px였던 반면 PR #4122 결과 약 24.94px가 한컴 PDF 약 27px에 가깝다.

반대로 리베이스 전 브랜치는 issue2007 p2–p4 우측선, p7–p17 fragment frame·clip,
완료된 중첩 표 뒤 제목, terminal tail 및 #3637·59043 비인라인 개체 소유권에 대한 더 넓은
PDF 기반 회귀를 갖고 있다. 따라서 어느 한쪽 파일 전체를 선택하지 않고 다음 계약으로
통합했다.

## 충돌 해소 계약

1. `recursive_cut.is_some()`이면 PR #4122의 child cursor가 항상 소유권의 권위다.
2. 기존 scalar `content_offset`, 첫 가시 unit 보정, 1×1 continuation의 위치 상한 해제는
   `recursive_cut.is_none()`인 fallback에만 적용한다.
3. `CellUnit`에는 PR #4122의 `stored_frame_break_before`, `nested_table_fragment`,
   `mixed_nested_recursive`와 기존의 `mixed_nested_starts_after_table`,
   `non_inline_control_range`를 함께 유지한다.
4. partial renderer의 재귀 `RowCut`과 upstream #4128의 metadata cursor helper 및 #4129의
   O(U) mixed-unit run walk를 유지한다.
5. 기존 border/clip/frame repair와 Square/Shape fragment owner 보정은 source cursor를
   바꾸지 않는 paint·fallback 계층에서 유지한다.
6. 저장된 순방향 full-line 빈 Enter는 PR #4122 계약을 유지하고, native HWP5 float ladder
   spacer 보정은 그 뒤의 별도 predicate로 합친다.

## 검증 계획

1. `cargo fmt --all -- --check`와 compile gate
2. issue2007 통합 회귀 12건
3. #2430, #3637, #1921/59043, #4129 scan budget, overflow-cell baseline
4. `cargo test --profile release-test --tests` 전체 integration gate
5. `cargo clippy --all-targets -- -D warnings`

검증은 다른 작업의 산출물과 분리한 `target/task-3820-3821-fidelity-rebase`에서
`CARGO_INCREMENTAL=0`으로 수행한다. 실패하면 baseline을 올리지 않고 원인 코드를 수정한다.

## 종료 판정

PR #4122의 재귀 child cursor와 기존 paint/fallback 보정을 함께 보존한 리베이스 충돌 해소는
`8fc0e2ef8`에 고정했다. 이후 issue2007 p12–p15에서 확인된 부모 `RowCut` 소유 경계 문제는
리베이스 자체와 분리해 [Stage 47](task_m100_3820_stage47_issue2007_parent_cut_analysis.md)에서
분석한다.
