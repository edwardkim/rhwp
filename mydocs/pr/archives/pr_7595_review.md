---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7595 리뷰 — 수정: 누름틀 앞에 인라인 개체를 넣어도 누름틀 범위를 보존한다

## 최종 판정

**머지 보류 — 누적 적용 후 검증 중.** 메인터너 보정과 필수 검증을 끝낸 integration head에서 수용 여부를 결정한다.

## 접수 정보

- 원 PR: [#7595](https://github.com/edwardkim/rhwp/pull/7595), semanticist21, devel 대상, non-draft.
- 원 head: `520a8ee399f255d7a6b91477dd744745a47240dd`; 접수 상태 `MERGEABLE` / `CLEAN`. CI는 원 head 참고값이다.
- 검토 branch: `review/semanticist21-20261005`, base `cdba77b609c399fdef26a6c9e637716aa32c2177`.
- reviewer jangster77를 REST API로 먼저 지정했다. maintainer_general 기본; intake_and_review, local_validation, multi_pr_update_branch 및 적용 시 visual_fixture_evidence 보조 경로다.
- 2026-10-05 목록 갱신의 추가 non-draft5건을 이번 그룹에 누적했다. 개별 기록을 유지하고 원 contributor branch는 변경하지 않았다.

## 적용 이력

| 원 SHA | 로컬 SHA | 상태 |
| --- | --- | --- |
| `e2b4a4306da4f4910e5bcc3d498fd3a1a50a7586` | `75c7cbc9837f84fafc0d4e0fbccd2f5f29b120e8` | applied |
| `520a8ee399f255d7a6b91477dd744745a47240dd` | `576ee0ff35f0fd17d99528f54a5b1a0ad2969e48` | maintainer-conflict-resolved |

## 변경·소비 경로 검토

6종 inline 개체 삽입 → Paragraph::shift_for_inline_control_insert(control_idx,char_offset) → 뒤 field_ranges.control_idx 및 body active field 주소 → 조회/저장/활성 입력. #7521로 분리된 inline_content.rs에 HTML 그림 호출을 옮겨 적용했다. #7592 책갈피와 #7530 머리말 필드의 기존 caller를 함께 보정한다.

## 검증 입력과 결과

공개 API로 생성한 합성 문서의6종 × 편집 중/HWP/HWPX 재열기 및 활성 필드 끝 입력2개 검사.

`tests/cases/inline_control_field_ranges.rs`와 필수 lint/native/fresh WASM/전체 회귀는 같은 최종 head에서 순차 검증한다. 원 PR이 제공한 수정 전후 실행은 참고 자료이며 이번 후보의 결과로 승계하지 않는다. 실제 실행 결과는 아직 미검증이다. 파일 입력은 검토 commit과 해시를 대조한다.

## 조판 원칙 준수 검토

모델 소유 주소 보존 계약 적용. 셀 활성 주소 보정은 원 기여 범위 밖이며 합성 계약의 통과를 한컴 fidelity로 기록하지 않는다.

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
