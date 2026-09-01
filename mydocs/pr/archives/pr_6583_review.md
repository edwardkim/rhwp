# PR #6583 검토 기록 — #6243 post-release canary 계획 정정

- PR: [#6583](https://github.com/edwardkim/rhwp/pull/6583)
- 이슈: [#6243](https://github.com/edwardkim/rhwp/issues/6243)
- 역할: `maintainer_general` self-review
- base: `devel@8eab87ce16ac667766b65032132c5a605898449a`
- 검토 대상 code head: `9601c121c47093c35092f558d15835607b36fe3f`
- 검토일: 2026-09-02 KST

## 1. 변경 범위

- `mydocs/plans/task_m100_6243.md`: 이미 종료된 #6215를 후행 canary로 사용하는 실행 불가능한 절차를
  실제 post-release Render Diff 대상 PR 관찰 절차로 정정한다.
- `mydocs/orders/20260902.md`: 원인, 수정 계획, v0.8.5 릴리즈 기록 게이트와 현재 상태를 기록한다.
- source, test, workflow, 버전, 배포물과 GitHub 권한은 바꾸지 않는다.

## 2. 사실 관계 대사

- #6243 구현 PR #6297은 merge commit `96da78a9c3e5ee78dd14109f8fdd9eef7c42b560`으로
  `devel`에 반영됐다.
- 종전 canary #6215는 통합 PR #6473으로 `devel`에 반영된 뒤 2026-08-30 종료됐다. `main` controller
  활성화 뒤 살아 있는 PR 계보를 만들 수 없으므로 post-release 증적으로 소급 사용할 수 없다.
- `main`은 검토 시점에 v0.8.4 commit `496333b27d21ddb9114ba9ae340bcb895870c9a7`이고,
  #6243의 trusted controller와 수정 Render Diff는 아직 live 기본 브랜치 정책이 아니다.
- #5949 Release Binary는 Render Diff trigger가 아니며, 그 dry-run을 #6243 canary로 세지 않는 구분이
  계획에 명시돼 있다.

## 3. 보호 불변식 검토

- 실제 Full code candidate, current-base merge bridge 1개, review-only tail의 세 SHA가 모두 있어야 한다.
- base가 전진하지 않았거나 Canvas worker가 실제 실행된 경우 #6243 완료 증적으로 세지 않는다.
- Full fallback은 안전 동작이며, 잘못된 fast-pass나 권한 확대가 아니면 릴리즈 rollback 조건으로 과장하지 않는다.
- 의미 없는 no-op canary를 금지하고 실제 유지보수 가치가 있는 Render Diff 대상 PR만 사용한다.
- #6243만 `main`에 cherry-pick하거나 admin push하지 않고 정상 v0.8.5 release PR로 controller를 활성화한다.

## 4. v0.8.5 기록 게이트 검토

- 기여자는 `v0.8.4..최종 릴리즈 후보`의 Git author·co-author와 GitHub PR author를 함께 수집한다.
- provenance-preserving cherry-pick·통합 PR은 commit message, `mydocs/pr/archives/`, 오늘할일을 교차 대사해
  원 PR author를 보존한다.
- `handle -> PR 번호` 근거표, `CHANGELOG.md`·`CHANGELOG_EN.md`의 기여자 절, GitHub 릴리즈 노트의
  기여자 집합이 일치해야 한다.
- 세 산출물 중 하나라도 없거나 집합이 다르면 release PR을 merge-ready로 판정하지 않는다.

## 5. 검증

- `git diff --check upstream/devel...9601c121c` — 통과
- `python3 scripts/check_markdown_links.py mydocs/plans/task_m100_6243.md mydocs/orders/20260902.md` — 2문서 통과
- `python3 -m unittest scripts.tests.test_review_only_fast_pass_workflows` — 24/24 통과
- exact head GitHub Actions:
  - CI run `33544833111` — 성공, 문서-only `Build & Test` 성공
  - CodeQL run `33544833068` — 성공, 분석 job 정상 생략
  - Proptest run `33544833109` — 성공
  - Adapter inter-diff run `33544833196` — 성공
- PR 상태: `MERGEABLE / CLEAN`, 실패·대기 check 없음

## 6. 발견 사항과 정정

- 비차단 정정 1건: 기존 Stage O2-3의 과거 Python 계약 수 `18/18`이 현재 기준처럼 읽혔다.
- trailing review 변경에서 최초 계획 당시 18/18과 2026-09-02 현행 24/24를 구분했다.
- 그 밖의 차단 문제, 과도한 범위 확장, 이슈 조기 종료 경로는 발견하지 못했다.

## 7. 최종 판정

- 판정: 승인
- 검증 대상: code head `9601c121c47093c35092f558d15835607b36fe3f`와 위 비차단 문서 정정
- #6243은 v0.8.5 `main` 활성화와 실제 후행 canary 성공 전까지 OPEN으로 유지한다.
- merge 전 조건: 이 review 기록을 포함한 최신 trailing head의 CI 성공, 최신 `devel` 대사,
  `MERGEABLE / CLEAN` 재확인과 메인테이너의 별도 merge 승인
