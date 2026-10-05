---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7590 리뷰 — 수정: 양식 값 JSON을 serde_json으로 해석해 따옴표·역슬래시를 보존한다

## 최종 판정

**머지 보류 — 누적 적용 후 검증 중.** 메인터너 보정과 필수 검증을 끝낸 integration head에서 수용 여부를 결정한다.

## 접수 정보

- 원 PR: [#7590](https://github.com/edwardkim/rhwp/pull/7590), semanticist21, devel 대상, non-draft.
- 원 head: `6529a6aaa6e2be432b42ea101983eba2ce7ec914`; 접수 상태 `MERGEABLE` / `CLEAN`. CI는 원 head 참고값이다.
- 검토 branch: `review/semanticist21-20261005`, base `cdba77b609c399fdef26a6c9e637716aa32c2177`.
- reviewer jangster77를 REST API로 먼저 지정했다. maintainer_general 기본; intake_and_review, local_validation, multi_pr_update_branch 및 적용 시 visual_fixture_evidence 보조 경로다.
- 2026-10-05 목록 갱신의 추가 non-draft5건을 이번 그룹에 누적했다. 개별 기록을 유지하고 원 contributor branch는 변경하지 않았다.

## 적용 이력

| 원 SHA | 로컬 SHA | 상태 |
| --- | --- | --- |
| `694d65555982afe6e22af2faa7948235fccb3995` | `e0166dc743f12bc22c881402fa5a10598817afb7` | applied |
| `6529a6aaa6e2be432b42ea101983eba2ce7ec914` | `089286608d715ae39a00aade41032980974950ab` | applied |

## 변경·소비 경로 검토

본문·셀 양식 API → apply_form_value의 serde_json 전체 object 해석 → raw_stream 무효화 → HWP/HWPX 저장. 잘못된 JSON은 값 변경 전에 거절하고 타입 불일치 키는 기존처럼 무시한다. 공통 json_escape로 제어문자 조회를 보존한다.

## 검증 입력과 결과

기존 공개 samples/form-01.hwp, samples/hwpx/form-002.hwpx와 escape/잘못된 입력/저장 왕복5개 검사.

`tests/cases/form_value_json.rs`와 필수 lint/native/fresh WASM/전체 회귀는 같은 최종 head에서 순차 검증한다. 원 PR이 제공한 수정 전후 실행은 참고 자료이며 이번 후보의 결과로 승계하지 않는다. 실제 실행 결과는 아직 미검증이다. 파일 입력은 검토 commit과 해시를 대조한다.

## 조판 원칙 준수 검토

JSON 문법과 원문 값 보존 계약이다. 조판 규칙·좌표·기준값 변경은 비해당이며 폼 렌더의 기존 검증과 구분한다.

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
- WASM Build: SKIPPED
- adapter inter-diff preflight: SUCCESS
- CodeQL preflight: SUCCESS
- Proptest preflight: SUCCESS
- adapter inter-diff: SUCCESS
- CI preflight: SUCCESS
- Analyze (javascript-typescript): SUCCESS
- prop roundtrip: SUCCESS
- Analyze (python): SUCCESS
- Analyze (rust): SUCCESS
- Resolve nextest target duration policy: SUCCESS
- build-test-archive-a / Build test archive (a): SUCCESS
- Lint (fmt, clippy, WASM check): SUCCESS
- Native Skia tests: SKIPPED
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
| `694d65555982afe6e22af2faa7948235fccb3995` | `e0166dc743f12bc22c881402fa5a10598817afb7` | `b2e5b43bc305403c9434727626ddca9eb3f1d00b` | rebased |
| `6529a6aaa6e2be432b42ea101983eba2ce7ec914` | `089286608d715ae39a00aade41032980974950ab` | `c96ee8ecce31b97e42f647f0791c6aa58261bd99` | rebased |

원 저자와 cherry-pick 출처를 유지했다. #7491의 원4개는 #7599를 통해 이미 base에 포함되어 중복 적용하지 않았다. 메인터너 보정과 개별 리뷰 기록은 재배치했다. 최종 후보의 시각·전체 회귀 및 CI는 별도 확인한다.
