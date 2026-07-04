# PR #1449 처리 계획 - PR 리뷰 워크플로 문서 규칙 분리

- PR: https://github.com/edwardkim/rhwp/pull/1449
- 관련 이슈: #1425
- base: `devel`
- head: `task_m100_1425`
- 처리 경로: collaborator self-merge 후보 예외 경로

## 1. 처리 원칙

이 PR은 새로 정리한 collaborator self-merge 후보 예외 경로를 적용한다.

- review 문서와 처리 계획서를 PR diff에 포함한다.
- `draft`, `mergeable`, `head SHA`, `CI 상태`는 merge 후 낡는 값이므로 확정 사실처럼 기록하지 않는다.
- 최종 merge는 PR head 최신 커밋 기준 GitHub Actions 통과와 작업지시자 승인 후 진행한다.
- merge 수행은 작업지시자가 GitHub merge 버튼으로 직접 진행한다.

## 2. 후속 커밋 구성

PR 생성 후 다음 문서를 PR head에 추가한다.

1. `mydocs/pr/archives/pr_1449_review.md`
2. `mydocs/pr/archives/pr_1449_review_impl.md`
3. `mydocs/report/task_m100_1425_report.md` 변경 파일 목록 보정

최신 `upstream/devel` 반영 과정에서 `mydocs/orders/20260621.md` add/add 충돌이 발생했으며, PR #1447
주문서 내용을 보존하고 #1425 M100 항목을 추가하는 방식으로 해소했다.

## 3. 검증 전략

문서 전용 PR이므로 다음 검증으로 제한한다.

- `git diff --check`
- 핵심 규칙 검색:

```bash
rg -n "maintainer|collaborator|Collaborator|self-merge|archives|draft|mergeable|head SHA|CI 상태|최신 GitHub Actions" mydocs/manual/pr_review_workflow.md
```

코드 변경이 없으므로 cargo/npm 빌드와 테스트는 생략한다.

## 4. GitHub 처리 순서

1. `task_m100_1425`를 최신 `upstream/devel` 기준으로 rebase한다.
2. 충돌이 있으면 upstream 문서를 보존하고 #1425 항목만 병합한다.
3. PR review 문서와 처리 계획서를 archive 경로에 작성한다.
4. 후속 문서 커밋을 PR head에 push한다.
5. PR 본문에 review 문서 경로와 최종 merge 조건을 반영한다.
6. GitHub Actions가 재실행되면 최신 결과를 확인한다.
7. checks 통과 후 draft를 ready 상태로 전환한다.
8. 작업지시자가 GitHub merge 버튼으로 merge한다.
9. merge 후 #1425 close 여부를 확인한다.

## 5. merge 전 확인 조건

merge 직전에는 다음 세 조건을 최신 상태로 확인한다.

- PR head 최신 커밋 기준 GitHub Actions 통과
- PR diff에 `mydocs/pr/archives/pr_1449_review.md`와 `pr_1449_review_impl.md` 포함
- 작업지시자 승인

## 6. merge 후 추가 문서 커밋 방지

이 PR의 review 문서와 처리 계획서는 PR head에 포함한다. 따라서 merge 후 별도 저장소 문서 커밋은 만들지 않는다.

필요한 후속 작업은 저장소 변경 없이 다음 상태 확인으로 제한한다.

- PR #1449 merged 상태 확인
- #1425 closed 상태 확인
- auto-close 실패 시 작업지시자 승인 후 수동 close
