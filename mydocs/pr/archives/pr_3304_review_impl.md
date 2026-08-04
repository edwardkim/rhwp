# PR #3304 v2 통합 실행 계획

## 적용 범위

- base: `upstream/devel` `fa953ffa6d22856c8450ae2881fca3404c3adca4`
- code candidate: `bcff621521c4af56eb6a7d68952d4bf1853c0dec`
- 원 PR: #3258, #3262, #3264, #3276, #3280, #3282, #3285, #3288
- 제외: #3036 — 메인터너 보류 상태

## 단계

1. 8개 feature와 메인터너 보정 commit을 v2에 누적하고 local release-test·focused contract·fmt·clippy를 확인한다.
2. code candidate `bcff62152`의 full CI 성공을 확인했다. 이 review 문서와 implementation 기록을 review-only
   trailing commit으로 추가하고, 최신 head preflight와 aggregate가 fast-pass 조건을 만족하는지 확인한다.
3. 작업지시자 승인 뒤 squash merge하고 merge SHA를 확인한다.
4. `upstream/devel`을 fast-forward한 뒤 관련 issue 9건과 원 PR 8건의 상태·중복 comment를 확인하고, 필요한
   close/comment를 게시한다.
5. v2 local/remote branch와 review 전용 target을 정확한 경로만 정리한다.

## rollback 경계

- merge 전에는 v2의 해당 보정 또는 feature commit만 revert한다. 원 contributor branch를 rewrite하지 않는다.
- merge 뒤 발견한 code 문제는 `devel` 직접 수정이 아니라 별도 후속 PR로 처리한다.
