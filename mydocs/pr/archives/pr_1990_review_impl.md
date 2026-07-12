# PR #1990 리뷰 구현 기록

## 대상

- PR: https://github.com/edwardkim/rhwp/pull/1990
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/1984
- 작성자: planet6897
- 원 head SHA: `4aeea2e90f5b38515cd9fe797efceef5bdeb8b12`
- 체리픽 commit: `1bfeabcae`

## Stage 1. 메타 확인

완료.

- reviewer `jangster77`를 assign했다.
- base `devel`, 문서 작성 시점 `MERGEABLE/CLEAN`을 확인했다.

## Stage 2. 체리픽 및 충돌 해소

완료.

- #1989 뒤에 적용했다.
- `src/serializer/hwpx/section.rs`에서 충돌이 발생했다.
- #1987 secPr 스칼라 보존과 #1984 각주/미주 모양 보존을 모두 유지하도록 해결했다.

## Stage 3. 검증

완료.

- `issue1984` targeted test, full integration test, clippy가 통과했다.

## Stage 4. 결론

통합 PR merge 후보. 원 PR comment에는 충돌 해결 방식과 통합 반영 사실을 남긴다.

