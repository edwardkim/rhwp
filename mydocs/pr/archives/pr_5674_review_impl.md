---
kind: pr-review-implementation
status: pending-ci
pr: 5674
---

# PR #5674 구현·통합 검토 기록

## 적용 순서

1. 최신 `upstream/devel@c2a36398d`에서 `review/kevin9327-q-cli-round2-20260819`를 만들었다.
2. #5659, #5662, #5663, #5664, #5668의 원 head를 번호순으로 `git cherry-pick -x` 적용했다.
3. q-kit 하위 명령 help 종료 코드 불일치를 확인해 `403da0a93` 메인터너 보정과 전수 help contract를 추가했다.
4. 파생 suite를 review worktree에서만 준비하고, formatter·clippy·focused contract·전체 release-test 회귀를 실행했다.

## 보정 이유

전역 `rhwp-q-kit --help`는 등록된 50개 명령을 광고하지만, `<명령> --help`는 각 handler의 미지 플래그 경로로 전달돼 종료 코드 2가 됐다. 등록된 query surface는 탐색 가능해야 하므로 dispatcher가 help를 공통 처리하게 고쳤다. 테스트는 help 목록을 다시 입력으로 삼아 이후 명령 등록 시 coverage 누락도 막는다.

## 검증 결과

- suite manifest 및 source unit tier 정책 통과
- formatter·clippy 통과
- q-kit focused contract 6/6, 단건 smoke 4/4, 하위 명령 help 50/50 통과
- 전체 release-test nextest 7,978/7,978 통과, 38 skipped

## 병합 후 체크리스트

1. 최신 trailing head CI 성공과 mergeability를 확인한다.
2. [#5674](https://github.com/edwardkim/rhwp/pull/5674)를 병합한다.
3. 원 PR #5659, #5662, #5663, #5664, #5668과 issue #5656, #5657, #5658, #5661, #5667에 통합·검증 결과를 댓글로 남기고 close한다.
4. `review/kevin9327-q-cli-round2-20260819`의 원격·로컬 브랜치, worktree, 전용 target을 소유·clean 상태 확인 뒤 정리한다.
