# PR #2471 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2471](https://github.com/edwardkim/rhwp/pull/2471) |
| 작성자 / base | kevin9327 / `devel` |
| 관련 이슈 | [#2428](https://github.com/edwardkim/rhwp/issues/2428) |
| 범위 | 각주가 없는 페이지의 `hitTestFootnote` native 호출 fast-reject |
| 검토자 | @jangster77 (검토 전 지정) |
| 검토 스냅샷 | 2026-07-20 GitHub 조회: +29/-2, 4 files, `maintainerCanModify=true`, `mergeStateStatus=BEHIND` (동적 참고값) |
| 판단 | 누적 통합 PR에 수용 |

## 검토와 검증

- PR 본문은 각주가 없는 페이지에서 불필요한 native 호출을 피하는 성능 경로를 설명했고, PR 코멘트는 없었다.
- 기여자 변경 `0564f976c`을 충돌 없이 적용했다. 동작 보존 성능 최적화이므로 visual sweep 대상이 아니다.
- focused 성능/히트테스트 회귀와 최종 release-test, Clippy, WASM 빌드를 통과했다.

## 렌더 영향 판정

- 각주 hit-test의 fast-reject 성능 경로만 다루며 renderer 출력 경로를 바꾸지 않는다. visual sweep은 필요하지 않다.

## 리스크와 권고

- 각주가 없는 페이지에만 적용되는 fast-reject라서 실제 각주 hit-test 경로는 보존한다.
- **권고**: 누적 통합 PR에 수용. 최신 통합 PR head의 CI가 성공한 뒤에만 merge한다.
