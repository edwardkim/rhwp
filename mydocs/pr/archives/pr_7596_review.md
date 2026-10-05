---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7596 리뷰 — 수정: 누름틀 삽입·제거가 같은 문단의 다른 컨트롤과 누름틀 위치를 보존한다

## 최종 판정

**머지 보류 — 누적 적용 후 검증 중.** 메인터너 보정과 필수 검증을 끝낸 integration head에서 수용 여부를 결정한다.

## 접수 정보

- 원 PR: [#7596](https://github.com/edwardkim/rhwp/pull/7596), semanticist21, devel 대상, non-draft.
- 원 head: `1e882aae27ed5474f1794a7d453a552f514b6078`; 접수 상태 `MERGEABLE` / `CLEAN`. CI는 원 head 참고값이다.
- 검토 branch: `review/semanticist21-20261005`, base `cdba77b609c399fdef26a6c9e637716aa32c2177`.
- reviewer jangster77를 REST API로 먼저 지정했다. maintainer_general 기본; intake_and_review, local_validation, multi_pr_update_branch 및 적용 시 visual_fixture_evidence 보조 경로다.
- 2026-10-05 목록 갱신의 추가 non-draft5건을 이번 그룹에 누적했다. 개별 기록을 유지하고 원 contributor branch는 변경하지 않았다.

## 적용 이력

| 원 SHA | 로컬 SHA | 상태 |
| --- | --- | --- |
| `5b98c883446fd0cc0a733ac21484d8a276cfe6d8` | `e79480f113abdbf5520f4e71c6ac384e06c323df` | applied |
| `1e882aae27ed5474f1794a7d453a552f514b6078` | `b349b4144e4f4fe7db01ae3ea9d34417e52b0ecd` | applied |

## 변경·소비 경로 검토

insert/remove ClickHere → raw_slot_gaps 검증 → BEGIN/END16유닛 및 내용만 삽입/삭제 → 원시 참조/field 번호/활성 상태 → HWP empty-field END의 BEGIN 직후 방출. 인접 빈 필드는 중첩되지 않고 범위/컨트롤 소유를 유지한다.

## 검증 입력과 결과

컨트롤 앞뒤, 맨 앞/중간/인접 빈 필드, target 우선순위, 글자모양, HWP/HWPX 왕복, 불명확 슬롯14개 검사.

`tests/cases/clickhere_field_slots.rs`와 필수 lint/native/fresh WASM/전체 회귀는 같은 최종 head에서 순차 검증한다. 원 PR이 제공한 수정 전후 실행은 참고 자료이며 이번 후보의 결과로 승계하지 않는다. 실제 실행 결과는 아직 미검증이다. 파일 입력은 검토 commit과 해시를 대조한다.

## 조판 원칙 준수 검토

raw 슬롯 보존과 serializer 순서 계약 적용. 데이터가 불명확한 문단은 무변경 거절한다. 값 설정의 rebuild_char_offsets는 별도 잔여 범위이며 이 PR이 전부 해결했다고 쓰지 않는다.

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거·일반성 | 충족 | JSON 또는 UTF-16 슬롯 계약이며 문서ID·px 분기 없음 |
| 측정·배치·줄 점유 | 비해당 | 측정/줄 나눔/높이 계산 규칙 변경 없음 |
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
- CodeQL: NEUTRAL
- CI Impact Policy: SUCCESS

## 남은 범위·후속 처리

위 범위와 실제 통과 결과를 개별 판정에 연결한다. 통합 PR CI를 정확한 head에서 확인한 뒤 사용자 요청에 따라 merge·후속 처리를 수행한다. 메인터너 변경과 원 기여의 해결 범위는 contributor 안내에서 구분하고 미해결 전체 이슈는 Refs로 남긴다.

## upstream/devel 위 rebase 적용 위치 — 2026-10-05

기준 `c167dc6abbebf69546575e2d16d06223791bab82`. 아래는 현재 이력의 실제 적용 위치이며 위의 이전 검증 SHA는 당시 이력으로 보존한다.

| 원 commit SHA | rebase 전 로컬 SHA | 현재 적용 SHA | 상태 |
| --- | --- | --- | --- |
| `5b98c883446fd0cc0a733ac21484d8a276cfe6d8` | `e79480f113abdbf5520f4e71c6ac384e06c323df` | `47a176d5f100fb8660b9e489c82a2ada2d2ff572` | rebased |
| `1e882aae27ed5474f1794a7d453a552f514b6078` | `b349b4144e4f4fe7db01ae3ea9d34417e52b0ecd` | `226b567ae9a626268baedac13a73a359f56e997f` | rebased |

원 저자와 cherry-pick 출처를 유지했다. #7491의 원4개는 #7599를 통해 이미 base에 포함되어 중복 적용하지 않았다. 메인터너 보정과 개별 리뷰 기록은 재배치했다. 최종 후보의 시각·전체 회귀 및 CI는 별도 확인한다.
