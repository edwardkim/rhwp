# PR #1983 리뷰 구현 기록

## 대상

- PR: https://github.com/edwardkim/rhwp/pull/1983
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/1981, https://github.com/edwardkim/rhwp/issues/1982
- 작성자: planet6897
- 원 commits: `096accac66f7f9366ac8f3d7b0d4bdaaca7a333e`, `dfc28ff34028a96181dfd3881ce2fdf8c9ba1dd4`
- 체리픽 commits: `1e320494a`, `03e35dafa`

## Stage 1. 메타 확인

완료.

- reviewer `jangster77`를 assign했다.
- base `devel`, 문서 작성 시점 `MERGEABLE/CLEAN`을 확인했다.

## Stage 2. 체리픽

완료.

- PR 내부 commit 2개를 순서대로 적용했다.
- 충돌은 없었다.

## Stage 3. 검증

완료.

- `issue1981`, `issue1982` targeted test와 전체 회귀 테스트, clippy가 통과했다.

## Stage 4. 결론

통합 PR merge 후보. 원 PR은 통합 PR merge 후 supersede 처리한다.

