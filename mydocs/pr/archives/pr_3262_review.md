# PR #3262 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#3262](https://github.com/edwardkim/rhwp/pull/3262) |
| 작성자 / base | `kevin9327` / `devel` |
| 검토자 | `@jangster77` (GitHub review request 확인) |
| 원 head | `f8bd6017245002b99126a08a4b9f60e60549f9f0` (2026-07-25 조회 참고값) |
| 규모 | +1247/-10, 8 files, 4 commits |
| 관련 이슈 | #3261 (제목·처리 보고서 기준; GitHub closing reference는 없음) |
| 통합 보정 | `df3797ce4` — `export-structure` 다중 입력을 사용법 오류로 처리 |
| 판단 | v2 통합 PR 수용 후보 |

## 범위와 검토

- `export-structure --json` 계약과 `batch export-structure` 축을 추가하며 #3258 위에 적층된다.
- 가시성 검토 branch에서 원 feature `eb09d5027`를 누적 적용했다.
- 원 구현은 `export-structure file-a file-b --json`을 성공 처리하면서 마지막 파일만 사용했다. 위치 인자의
  묵시적 덮어쓰기는 machine-readable 계약과 맞지 않는다.

## 보정과 검증

- v2의 `df3797ce4`는 정확히 한 파일만 허용하고 다중 입력을 stdout 없는 `EXIT_USAGE`로 고정한다.
- `cli_json_contract` focused 검증 22 passed와 누적 full release-test 전체 성공을 확인했다.
- Rust CLI·문서 변경만 있고 renderer·layout·fixture·golden 변경은 없으므로 visual sweep과 baseline 등록은 불필요하다.

## 권고와 다음 조건

- **권고: #3258 뒤 v2 통합 PR로 수용.** 1,000줄 초과 PR의 보정 범위는 입력 계약으로 한정하며 contributor
  source head에는 push하지 않는다.
- 원 PR의 `MERGEABLE`/`BEHIND`와 CI는 참고값이다. 최신 통합 PR의 full CI와 merge 조건을 재확인한다.
