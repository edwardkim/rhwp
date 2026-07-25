# PR #3285 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#3285](https://github.com/edwardkim/rhwp/pull/3285) |
| 작성자 / base | `kevin9327` / `devel` |
| 검토자 | `@jangster77` (GitHub review request 확인) |
| 원 head | `9132da68c642ab282619534a4c51034a30b729a4` (2026-07-25 조회 참고값) |
| 규모 | +639/-1, 7 files, 3 commits |
| 관련 이슈 | #3283 (제목·처리 보고서 기준; GitHub closing reference는 없음) |
| 통합 보정 | `5f495e60a`, `a60ae3294` — 글상자 주소·Unicode offset과 test module 정합 |
| 판단 | v2 통합 PR 수용 후보 |

## 범위와 검토

- `search --json`은 본문·표 셀·글상자를 위치와 페이지와 함께 조회한다. 원 feature `074cd71e1`를 누적 적용했다.
- 원 구현은 글상자 내 match에 outer paragraph만 남겨 재탐색할 control/inner-paragraph 주소가 없었고, Unicode
  소문자 확장 문자열의 char offset이 원문 index와 달라질 수 있었다.

## 보정과 검증

- v2의 `5f495e60a`는 optional `textbox {control, paragraph}` 주소와 원문 char offset mapping을 추가한다.
  `a60ae3294`는 같은 회귀가 기존 test module 안에 한 번만 선언되도록 정리한다.
- 글상자 sample direct CLI 결과에서 `textbox.control=2`, `textbox.paragraph=10`을 확인했고,
  `search_json_contract` 10 passed 및 누적 full release-test 전체 성공을 확인했다.
- renderer·layout·fixture·golden 변경은 없으므로 visual sweep과 baseline 등록은 불필요하다.

## 권고와 다음 조건

- **권고: v2 통합 PR로 수용.** 두 보정은 source branch가 아니라 v2에만 포함한다. code/test 보정이므로 full CI를
  기다린다.
- 원 PR의 `MERGEABLE`/`BEHIND`와 CI 성공은 참고값이며 통합 PR의 최신 상태를 재확인한다.
