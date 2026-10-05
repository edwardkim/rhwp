---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7597 리뷰 — 수정: 누름틀 끝 뒤 컨트롤의 글자 위치를 앞당기지 않는다

## 최종 판정

**머지 보류 — 누적 적용 후 검증 중.** 메인터너 보정과 필수 검증을 끝낸 integration head에서 수용 여부를 결정한다.

## 접수 정보

- 원 PR: [#7597](https://github.com/edwardkim/rhwp/pull/7597), semanticist21, devel 대상, non-draft.
- 원 head: `fe01de7653deb3de002e3207c88c20f2f49c42ea`; 접수 상태 `MERGEABLE` / `UNSTABLE`. CI는 원 head 참고값이다.
- 검토 branch: `review/semanticist21-20261005`, base `cdba77b609c399fdef26a6c9e637716aa32c2177`.
- reviewer jangster77를 REST API로 먼저 지정했다. maintainer_general 기본; intake_and_review, local_validation, multi_pr_update_branch 및 적용 시 visual_fixture_evidence 보조 경로다.
- 2026-10-05 목록 갱신의 추가 non-draft5건을 이번 그룹에 누적했다. 개별 기록을 유지하고 원 contributor branch는 변경하지 않았다.

## 적용 이력

| 원 SHA | 로컬 SHA | 상태 |
| --- | --- | --- |
| `cbbfedaf821fb6c0a6022bd7fa31dd6fcca5a04a` | `459fe9de0963e13ff84118fc0ae9bc6b51773ca6` | applied |
| `fe01de7653deb3de002e3207c88c20f2f49c42ea` | `ee2f5cf8de90db77050ea0e143c8d0d2e6e08110` | applied |

## 변경·소비 경로 검토

control_text_positions 갭 수 → FIELD_END/orphan END/title mark 수 제외 → control_utf16_positions·논리 주소·각주/그림/도형 실제 원점. 마지막 갭의 남은 실제 controls는 기존 끝 위치를 따른다.

## 검증 입력과 결과

누름틀1..3 끝 뒤의 각주가 글자4에 남는 public API 합성 입력1개 검사. 한컴/Native Skia 직접 위치 일치는 아직 미검증.

`tests/cases/field_end_control_positions.rs`와 필수 lint/native/fresh WASM/전체 회귀는 같은 최종 head에서 순차 검증한다. 원 PR이 제공한 수정 전후 실행은 참고 자료이며 이번 후보의 결과로 승계하지 않는다. 실제 실행 결과는 아직 미검증이다. 파일 입력은 검토 commit과 해시를 대조한다.

## 조판 원칙 준수 검토

렌더 소비 원시 주소의 정확성 적용. 이미 있는 숨은 표지는 컨트롤을 소유하지 않는다는 모델/serializer 계약을 대조한다. 실제 각주 위치를 편집 중과 저장 재열기에서 검사한다.

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거·일반성 | 충족 | JSON 또는 UTF-16 슬롯 계약이며 문서ID·px 분기 없음 |
| 측정·배치·줄 점유 | 미검증 | 원시 주소의 실제 렌더 소비 확인 필요 |
| 분할·이어받기 | 비해당 | pagination/rowspan/cut 변경 없음 |
| 사례·독립 기대값 | 미검증 | public API의 입력 의미/원시 슬롯 보존을 기대값으로 실행 예정 |
| 기준값 변경 | 비해당 | golden/허용치 변경 없음 |
| 주장·실행 범위 | 미검증 | 누적 head의 필수 검증 완료 전 |

## 원 head CI 참고값

- trusted_postmerge_reuse / Verify trusted post-merge reuse: SUCCESS
- trusted_postmerge_reuse / Verify trusted post-merge reuse: SUCCESS
- Evaluate trusted CI impact policy: SUCCESS
- trusted_postmerge_reuse / Verify trusted post-merge reuse: SUCCESS
- trusted_postmerge_reuse / Verify trusted post-merge reuse: SUCCESS
- Render Diff preflight: SUCCESS
- WASM Build: SKIPPED
- adapter inter-diff preflight: SUCCESS
- CodeQL preflight: SUCCESS
- Proptest preflight: SUCCESS
- Canvas visual diff: SUCCESS
- adapter inter-diff: SUCCESS
- CI preflight: SUCCESS
- Analyze (javascript-typescript): SUCCESS
- prop roundtrip: SUCCESS
- Analyze (python): SUCCESS
- Analyze (rust): SUCCESS
- Resolve nextest target duration policy: SUCCESS
- build-test-archive-a / Build test archive (a): SUCCESS
- Lint (fmt, clippy, WASM check): SUCCESS
- Native Skia tests: SUCCESS
- Frontend unit gates: SKIPPED
- Frontend package gates: SKIPPED
- Workflow promotion preflight: SKIPPED
- build-test-archive-b / Build test archive (b): SUCCESS
- build-test-archive-c / Build test archive (c): SUCCESS
- build-test-archive-d / Build test archive (d): SUCCESS
- test-archive-a-shard-1 / Default-feature tests (Archive A): SUCCESS
- test-archive-b-shard-1 / Default-feature tests (Archive B): SUCCESS
- test-archive-c-shard-1 / Default-feature tests (Archive C): SUCCESS
- test-archive-d-shard-1 / Default-feature tests (Archive D): SUCCESS
- Build & Test: SUCCESS
- CI Impact Policy: PENDING
- CodeQL: NEUTRAL

## 남은 범위·후속 처리

위 범위와 실제 통과 결과를 개별 판정에 연결한다. 통합 PR CI를 정확한 head에서 확인한 뒤 사용자 요청에 따라 merge·후속 처리를 수행한다. 메인터너 변경과 원 기여의 해결 범위는 contributor 안내에서 구분하고 미해결 전체 이슈는 Refs로 남긴다.

## upstream/devel 위 rebase 적용 위치 — 2026-10-05

기준 `c167dc6abbebf69546575e2d16d06223791bab82`. 아래는 현재 이력의 실제 적용 위치이며 위의 이전 검증 SHA는 당시 이력으로 보존한다.

| 원 commit SHA | rebase 전 로컬 SHA | 현재 적용 SHA | 상태 |
| --- | --- | --- | --- |
| `cbbfedaf821fb6c0a6022bd7fa31dd6fcca5a04a` | `459fe9de0963e13ff84118fc0ae9bc6b51773ca6` | `5962b4650b7423cf0ca0e3320b142507aed60936` | rebased |
| `fe01de7653deb3de002e3207c88c20f2f49c42ea` | `ee2f5cf8de90db77050ea0e143c8d0d2e6e08110` | `2231cb02540563408dfc5cde83279eb4e43d0fc1` | rebased |

원 저자와 cherry-pick 출처를 유지했다. #7491의 원4개는 #7599를 통해 이미 base에 포함되어 중복 적용하지 않았다. 메인터너 보정과 개별 리뷰 기록은 재배치했다. 최종 후보의 시각·전체 회귀 및 CI는 별도 확인한다.
