---
kind: report
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-06
---

# PR #7590 리뷰 — 수정: 양식 값 JSON을 serde_json으로 해석해 따옴표·역슬래시를 보존한다

## 최종 판정

**통합 후보 검토 충족 — code candidate CI 성공, 동일 PR 후행 기록의 최신 aggregate 확인 후 통합.** 실제 변경·회귀·독립 Print/직접 시각 확인 및 비조판 계약을 이 PR의 기록 범위에서 충족했다. 원 head 자체의 approve/merge와 구분하며, 누적 source8569f49ce와 통합 [PR #7601](https://github.com/edwardkim/rhwp/pull/7601)의 code candidate774fe4071 최신 원격 CI가 성공했다. 후행 기록 head의 aggregate와 merge 가능 상태까지 확인한 뒤 통합한다.

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

## Merge 후 contributor PR comment 계획

원 기여에 감사한 뒤 실제 통합 PR 링크·merge SHA·정확한 최종 head CI와 이 PR의 회귀 실행 결과를 한국어 존댓말로 게시한다. 원 head는 merge 직전에 다시 확인하고 동일할 때만 통합으로 대체된 원 PR을 닫는다. 원 contributor fork branch는 삭제하지 않는다.

이 PR의 원시 주소/저장/조회 계약과 누적 보정 범위를 구분한다. 합성 계약 검사를 한컴 조판 일치로 표현하지 않는다.

- 조판 출력은 변경하지 않는 주소/JSON/캐럿/선택/탐색 계약이므로 이 PR 자체의 Print 일치율을 꾸며 적지 않는다. 실제 public API·회귀 검사의 적용 범위와 미검증 범위를 명시한다.
- 이슈는 확인된 해결 범위만 다루고, 남은 조판·입력 축은 `Refs`와 원 이슈 링크로 유지한다. 게시 뒤 API로 실제 줄바꿈·한글·이미지 URL을 다시 확인한다.


## 최종 production 전쪽 재검증 — 2026-10-06

Production·검증 source `8569f49ce051ee343d58866a4f1e20a642d9e7fa`, 정책 base `c167dc6abbebf69546575e2d16d06223791bab82`를 검증했다. fresh WASM SHA-256 `410f8f3540f2856fcd7200a115f191a87aed4b2e726267bc54d960b35948735a`, JS `70cde06a369fa7fd4fc8bc8f3d6acaee158596ba1a116a2c72159002b0b5654e`; root `pkg`와 Studio public의 실제 바이트가 같다. Native binary SHA-256 `d4ff621918e81070809efe96555f9e484905f12c20d77f9a78f81ce4095fbe46`. 아래 결과는 원점 리셋 보정까지 포함한 최신 source의 재출력이다.

Native/fresh WASM 각각30항목·37쪽(합계74쪽 대응) 최신 재출력, 최저91.96451%, 90% 미만/누락/측정 불가/글꼴 예외0이다. canonical TSV: ignored `output/pr-review/semanticist21-20261005/ladder-reset-tsv/<native|wasm>/<key>/silhouette.tsv`. 입력·Print 출처와 직접 판독은 위 개별 증거를 따르며, [공통 렌더/TSV 명령·재출력 검증](../assets/semanticist21-20261005/README.md#문단-원점-리셋-보정의-최종-nativefresh-wasm-검증)에 연결한다. 전체 Rust 및 원격 CI 완료 여부는 다음 최종 판정에서 별도로 기록한다.


이 PR 자체의 비조판 계약 판정은 위 개별 회귀·실제 화면 기록을 따른다. 공통 대조군 점수를 자체 출력의 정확성으로 환산하지 않는다.

- 최신 source의 실제 브라우저3종 PASS(화면/질의/폼), source registry·fmt·Native/WASM/workspace-all-targets Clippy·workspace build·base manifest/unit-tier 정책 PASS. Native 선행33건 및 Cargo 집중32건은 각각 모두 PASS이며 최종 전체 nextest/Native Skia 결과는 다음 판정에 기록한다. 원시 로그·중간 JSON·TSV는 ignored `output/pr-review/semanticist21-20261005`에 보존한다.


## 최종 로컬 게이트 — production8569f49ce

정책 base `c167dc6abbebf69546575e2d16d06223791bab82`, 검증 source `8569f49ce051ee343d58866a4f1e20a642d9e7fa`. fmt·Native/WASM32/workspace-all-targets Clippy `-D warnings`·workspace build·manifest 및 source unit tier `--check --base-ref <base>` 모두PASS다. 파생 suite를 준비한 동일 review checkout의 `target/pr-review`에서 Cargo를 순차 실행했다.

- 전체 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`:10,491 PASS/0 FAIL/50 SKIP,706.118초, exit0.
- Native 선행33/ Cargo 집중32 모두PASS; 겹침16 partition 전체를 포함한다. 기존 fixture/baseline/래칫을 완화하지 않았다.
- Native Skia lib·missing picture·direct PDF·ComboBox·password 모두PASS. optional backend 검사를 원 head CI 또는 SVG 점수로 대신하지 않았다.
- fresh WASM/Studio 동기화·실제 화면/질의/폼3종·최신 Native/fresh WASM74쪽 대응/TSV 모두PASS(최저91.96451%, 미달/누락/측정 불가0, font exception0).

실제 명령·exit·시간은 ignored `output/pr-review/semanticist21-20261005/oct06-ladder-reset-validation-progress.json`, 원 출력은 `logs/oct06-ladder-reset-*.log`에만 보존한다. 각 단계의 마지막 summary는 아래 공통 증거 README에 기록한다. source가 바뀌면 이 실행 결과를 그대로 승계하지 않는다. 원 PR의 별도 CI와 누적 후보의 최신 원격 CI를 구분하며 통합 PR의 최종 head CI를 확인한 뒤 merge한다.


## 통합 PR #7601 code candidate CI와 후행 기록

[통합 PR #7601](https://github.com/edwardkim/rhwp/pull/7601), code candidate `774fe407160598bd029cbb13a3545ee30ca057df`의 [최신 head CI](https://github.com/edwardkim/rhwp/pull/7601/checks)가 모두 종료되어 성공/정상 생략을 확인했다. 정확한 run URL·결론은 [공통 CI 증거](../assets/semanticist21-20261005/README.md#통합-pr-7601-code-candidate-ci)에 기록했다. 원 PR의 별도 CI를 이 결과로 바꾸지 않는다.

Production source `8569f49ce051ee343d58866a4f1e20a642d9e7fa`의 최종 전체 nextest 로그에서 이 PR의 실제 검사 결과를 확인했다. 아래 PASS는 같은 source의 전체 실행이며 이전 원 PR의 보고를 승계한 값이 아니다.

| 원본 검사 | 실제 PASS | FAIL |
| --- | --- | --- |
| `tests/cases/form_value_json.rs` | 5 | 0 |

이 commit은 개별 archive 기록·오늘할일·CI 증거만 보완한다. Rust source/tests와 fresh WASM은 위 검증 source와 같으며, 같은 PR의 후행 head 최신 aggregate를 확인한 뒤 일반 merge commit으로 통합한다. merge 뒤 확정되는 SHA·이슈 상태·원 PR별 코멘트는 GitHub 후속 기록으로 남긴다. 위 접수·중간 실패·진행 중 문구는 당시 source의 역사 기록이며 이 절의 최신 판정과 구분한다.
