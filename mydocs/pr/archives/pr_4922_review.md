# PR #4922 검토 - 계획 CAS 판정과 재계획 hint

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4922](https://github.com/edwardkim/rhwp/pull/4922) |
| 관련 이슈 | [#4378](https://github.com/edwardkim/rhwp/issues/4378), [#3905](https://github.com/edwardkim/rhwp/issues/3905) |
| 작성자 | `kevin9327` (`kevin`) |
| 검토 방식 | #4931 누적 통합을 위한 archive review |
| base / head | `devel` / `feat/plan-preconditions-cas` |
| source candidate | `fbb8fb10fc967dba3593a9a90d22159fd7149507` |
| 통합 commit | `23f77f51e381128cef64bbcf7cc0af12dde7fd4e` |
| 규모 | 12 files, +494 / -43 |
| 작성 시점 상태 | `OPEN`, `MERGEABLE`, `CLEAN` |

## 범위와 판단

- plan `preconditions.inputSha256` 불일치를 usage error와 구분해 exit code `3`의 판정으로 통일하고,
  실행 가능한 `nextCall` 재검증 힌트를 추가한다.
- capabilities, schema, MCP 및 journal이 같은 CAS 어휘를 사용하도록 맞춰 표면별 결과 갈림을 줄인다.
- CLI/MCP/계획서 schema를 함께 바꾸므로 source candidate에서 Native Skia, CodeQL, Canvas visual diff를 포함한
  전체 CI가 성공한 점을 확인했다.

## 검증

- source candidate의 Build & Test, Native Skia, CodeQL, Canvas visual diff, frontend package gate가 성공했다.
- 기본 feature 세 shard·slow shard 및 lint도 성공했다.
- #4931 누적 tree의 clippy와 전체 `release-test` integration 회귀를 종료 코드 `0`으로 완료했다.

## 위험과 권고

exit code `3`을 자동화 판정으로 해석하는 downstream은 `nextCall`을 선택적으로 사용해야 하며, 실제 재실행 전
계획을 재검증해야 한다. 계약 변경의 tests와 전체 CI가 뒷받침되므로 #4931 통합 merge를 권고하고, 원 PR은
merge 뒤 supersede 처리한다.
