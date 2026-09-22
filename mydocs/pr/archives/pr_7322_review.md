# PR #7322 검토 기록 — 새 nextest target의 기본 추정 5초

- 원 PR: [#7322](https://github.com/edwardkim/rhwp/pull/7322)
- 관련 이슈: [#7321](https://github.com/edwardkim/rhwp/issues/7321)
- 작성자: `jangster77` (collaborator)
- 원 code head: `076fb58473e47f50b10685f4f00add68d966c9ae`
- merge commit: [`a592d4ddcc5a85748689f8dd84a9f34e921ae9cd`](https://github.com/edwardkim/rhwp/commit/a592d4ddcc5a85748689f8dd84a9f34e921ae9cd)
- 기록 경로: 옵션 2 — 원 PR의 trailing commit 없이 별도 문서 PR로 보존

## 변경 범위

새 `#[test]`·`#[case]`가 duration policy에 아직 없을 때의 기본 추정 시간을 테스트 하나당
60초에서 5초로 낮췄다. 변경 파일은 다음 세 개뿐이다.

- `tests/suites/nextest-target-duration-policy.json`
- `scripts/tests/nextest-target-duration-policy.test.mjs`
- `scripts/tests/test_nextest_archive_workflow.py`

단일 BCD archive·ownership manifest 실험은 실측에서 기존 병렬 B/C/D보다 전체 CI 시간이 길어
채택하지 않았다. 기존 B/C/D archive build와 worker 분배 구조는 유지했다. 비교 근거와 채택 결정은
[#7321의 최종 분석](https://github.com/edwardkim/rhwp/issues/7321#issuecomment-5761646083)에 남겼다.

## 검증

로컬에서 다음 계약 검사를 통과했다.

- `node --test scripts/tests/nextest-target-duration-policy.test.mjs` — 14 passed
- `python3 -m unittest scripts/tests/test_nextest_archive_workflow.py` — 15 passed

원 code head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/35609597799)는 Build & Test,
CodeQL, Adapter inter-diff, Proptest roundtrip과 CI Impact Policy를 모두 통과했다.

renderer·layout·paint·WASM 출력·HWP/HWPX fixture는 바뀌지 않아 Visual Sweep과 한컴 PDF 비교는 비대상이다.

## 병합 및 후속 처리

원 PR은 2026-09-21에 merge commit `a592d4d`로 `devel`에 병합됐다. [Close Issues on devel
Push](https://github.com/edwardkim/rhwp/actions/runs/35611894199)는 성공했고, closing keyword에 따라
#7321이 자동 종료된 것을 확인했다.

post-merge에는 정책대로 별도 검증 CI를 다시 실행하지 않았다.
[Refresh nextest target duration data](https://github.com/edwardkim/rhwp/actions/runs/35611893226)는
성공한 PR CI run `35609597799`의 B/C/D worker attempt 1 측정값을 수집해 `ci-metrics/nextest-target-durations`에
반영했다. 이 기록 PR은 review·오늘할일만 포함하며 원 PR의 trailing commit이 아니다.

## 최종 판정

**승인.** 새 target은 측정 전 5초씩 배정되고, 이후 성공한 B/C/D JUnit 실측이 duration policy를 갱신한다.
