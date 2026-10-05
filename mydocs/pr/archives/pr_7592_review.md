---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-05
---

# PR #7592 리뷰 — 수정: 책갈피 추가·삭제 후 원시 위치와 뒤 누름틀을 보존한다

## 최종 판정

**머지 보류 — 누적 적용 후 검증 중.** 메인터너 보정과 필수 검증을 끝낸 integration head에서 수용 여부를 결정한다.

## 접수 정보

- 원 PR: [#7592](https://github.com/edwardkim/rhwp/pull/7592), semanticist21, devel 대상, non-draft.
- 원 head: `2e1082980708ec36022246f945fd5ff9626cbf87`; 접수 상태 `MERGEABLE` / `CLEAN`. CI는 원 head 참고값이다.
- 검토 branch: `review/semanticist21-20261005`, base `cdba77b609c399fdef26a6c9e637716aa32c2177`.
- reviewer jangster77를 REST API로 먼저 지정했다. maintainer_general 기본; intake_and_review, local_validation, multi_pr_update_branch 및 적용 시 visual_fixture_evidence 보조 경로다.
- 2026-10-05 목록 갱신의 추가 non-draft5건을 이번 그룹에 누적했다. 개별 기록을 유지하고 원 contributor branch는 변경하지 않았다.

## 적용 이력

| 원 SHA | 로컬 SHA | 상태 |
| --- | --- | --- |
| `627c9060867bea690b45c31813d855cd4d8c0fe9` | `4135c1b1a52f1f5f3b97e014934aa31addaf4751` | applied |
| `89b46177b4d157a742ff38b2055d5f926fde3863` | `084f4a2c83915cbe01853862b7df5c892b92aec9` | applied |
| `4bd33c66f5588abf56fbc3a1851e634891abd8dc` | `31f077276efa0c93c6256802d98ad01c32268c59` | applied |
| `2e1082980708ec36022246f945fd5ff9626cbf87` | `10bb47410c34eb3e7c5970278bed1d8f28b98aff` | applied |

## 변경·소비 경로 검토

책갈피 추가/삭제 → 8 UTF-16 슬롯 이동 → char_offsets·char_count·글자모양·range/markpen·field control index → 저장 재조판. 삭제 전 raw 슬롯이 불명확하면 InvalidField로 무변경 거절한다. #7595와 함께 적용해 새 helper의 컨트롤 번호 이동을 중복하지 않도록 메인터너가 통합했다.

## 검증 입력과 결과

본문 책갈피 앞뒤 컨트롤, 비BMP·tab·하이라이트, 동일 세션 추가/삭제, 숨은 슬롯 거절9개 검사.

`tests/cases/bookmark_delete_preservation.rs`와 필수 lint/native/fresh WASM/전체 회귀는 같은 최종 head에서 순차 검증한다. 원 PR이 제공한 수정 전후 실행은 참고 자료이며 이번 후보의 결과로 승계하지 않는다. 실제 실행 결과는 아직 미검증이다. 파일 입력은 검토 commit과 해시를 대조한다.

## 조판 원칙 준수 검토

슬롯 보존의 모델 계약 적용. layout에 샘플별 좌표 보정은 없다. 합성 public API와 HWPX 책갈피 삽입본의 원시 주소/저장 결과를 확인하며 한컴 일치와 구분한다.

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
| `627c9060867bea690b45c31813d855cd4d8c0fe9` | `4135c1b1a52f1f5f3b97e014934aa31addaf4751` | `bfc9be54786257d2b9442799ddc9bd5700010a05` | rebased |
| `89b46177b4d157a742ff38b2055d5f926fde3863` | `084f4a2c83915cbe01853862b7df5c892b92aec9` | `c3450e89558ce8743a400f6a74f062eab2cb8abd` | rebased |
| `4bd33c66f5588abf56fbc3a1851e634891abd8dc` | `31f077276efa0c93c6256802d98ad01c32268c59` | `af0c9315e30b7c4f513f201ba7d1d6568ee5132d` | rebased |
| `2e1082980708ec36022246f945fd5ff9626cbf87` | `10bb47410c34eb3e7c5970278bed1d8f28b98aff` | `0b76864a2bf1e2b27eca0678ee6870f8b278b7db` | rebased |

원 저자와 cherry-pick 출처를 유지했다. #7491의 원4개는 #7599를 통해 이미 base에 포함되어 중복 적용하지 않았다. 메인터너 보정과 개별 리뷰 기록은 재배치했다. 최종 후보의 시각·전체 회귀 및 CI는 별도 확인한다.
