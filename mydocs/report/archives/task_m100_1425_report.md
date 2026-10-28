# Task M100 #1425 최종 보고서 - PR 리뷰 워크플로 문서의 maintainer/collaborator 규칙 분리

- 이슈: #1425 "PR 리뷰 워크플로 문서의 maintainer/collaborator 규칙 분리"
- 마일스톤: M100 (v1.0.0)
- 브랜치: `task_m100_1425`
- 작성일: 2026-06-21

## 1. 개요

`mydocs/manual/pr_review_workflow.md`에서 maintainer 일반 PR 처리 경로와 collaborator self-merge 후보
예외 경로가 섞여 보이던 문제를 정리했다.

PR #1420에서 추가된 collaborator 운영 규칙은 삭제하지 않고 별도 예외 섹션으로 보존했다. 동시에
maintainer가 외부 기여자 PR을 검토하는 기본 경로는 active review 문서 작성 후 archive 이동 방식으로
복구했다.

## 2. 주요 변경

### maintainer 일반 경로 복구

- 기본 review 문서 작성 위치를 `mydocs/pr/pr_{N}_review.md`,
  `mydocs/pr/pr_{N}_review_impl.md`로 정리했다.
- 처리 완료 후 `mydocs/pr/archives/`로 이동하는 절차를 7.5절에 복구했다.
- 승인 요청 예시의 review 문서 경로도 active 경로 기준으로 수정했다.

### collaborator self-merge 후보 예외 경로 분리

- 새 8장 `Collaborator self-merge 후보 예외 경로`를 추가했다.
- 처음부터 `mydocs/pr/archives/pr_{N}_review*.md`를 사용하는 조건을 collaborator self-merge 후보로 제한했다.
- PR head에 review 문서와 처리 계획서를 포함하는 이유를 "merge 후 추가 문서 커밋 방지"로 명시했다.
- `upstream` 작업 브랜치 직접 push 규칙은 예외 경로의 remote push 규칙으로 이동했다.

### volatile 상태값 기록 규칙 추가

- `draft`, `mergeable`, `head SHA`, `CI 상태`를 확정 사실처럼 기록하지 않도록 3.3절을 추가했다.
- 필요한 경우 "문서 작성 시점 참고값" 또는 "merge 전 최신 상태 확인 필요"로만 쓰도록 제한했다.
- 최종 merge 조건을 "PR head 최신 커밋 기준 GitHub Actions 통과 + 작업지시자 승인"으로 명시했다.

## 3. 변경 파일

| 파일 | 내용 |
|---|---|
| `mydocs/manual/pr_review_workflow.md` | maintainer/collaborator 경로 분리, volatile 상태값 규칙 추가 |
| `mydocs/orders/20260621.md` | #1425 오늘할일 항목 추가 및 완료 처리 |
| `mydocs/plans/task_m100_1425.md` | 수행계획서 |
| `mydocs/plans/task_m100_1425_impl.md` | 구현계획서 |
| `mydocs/report/task_m100_1425_report.md` | 최종 보고서 |
| `mydocs/pr/archives/pr_1449_review.md` | PR #1449 review 문서 |
| `mydocs/pr/archives/pr_1449_review_impl.md` | PR #1449 처리 계획 |

## 4. 검증

실행한 검증:

```bash
rg -n "maintainer|collaborator|Collaborator|self-merge|archives|draft|mergeable|head SHA|CI 상태|최신 GitHub Actions" mydocs/manual/pr_review_workflow.md
```

- 통과. maintainer 일반 경로, collaborator 예외 경로, volatile 상태값 규칙 위치를 확인했다.

```bash
git diff --check
```

- 통과. Markdown 공백 오류 없음.

코드 변경은 없으므로 `cargo test`, `npm test`, 빌드 검증은 실행하지 않았다.

## 5. 비목표 확인

- PR #1420 또는 PR #1424의 코드 변경은 수정하지 않았다.
- GitHub branch protection, repository 권한 설정은 변경하지 않았다.
- maintainer 승인 없이 이슈 close 또는 PR merge 절차를 자동화하지 않았다.

## 6. 결론

#1425의 완료 기준인 maintainer 일반 경로와 collaborator self-merge 후보 경로 분리, collaborator 규칙 보존,
volatile 상태값 기록 제한을 문서에 반영했다. 향후 collaborator self-merge PR의 review 문서는 ready 전환,
CI 재실행, merge 이후에도 상태값 모순이 생기지 않도록 작성 기준을 갖게 되었다.
