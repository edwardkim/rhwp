# PR #1449 리뷰 - PR 리뷰 워크플로 문서 규칙 분리

- PR: https://github.com/edwardkim/rhwp/pull/1449
- 제목: `Task #1425: PR 리뷰 워크플로 문서 규칙 분리`
- 작성일: 2026-06-21
- 작성자: `postmelee`
- 관련 이슈: #1425 `PR 리뷰 워크플로 문서의 maintainer/collaborator 규칙 분리`
- base: `devel`
- head: `task_m100_1425`
- 처리 경로: collaborator self-merge 후보 예외 경로

## 1. 요약 판단

PR #1449는 #1425에서 요구한 PR 리뷰 워크플로 문서 정리 작업이다.

핵심 변경은 maintainer 일반 경로와 collaborator self-merge 후보 예외 경로를 명확히 분리하는 것이다.
PR #1420에서 추가된 collaborator 운영 규칙은 삭제하지 않고 별도 예외 섹션으로 보존했고, maintainer가
외부 PR을 검토하는 기본 경로는 active review 문서 작성 후 archives 이동 방식으로 복구했다.

이 PR 자체도 새 예외 경로의 적용 대상이다. 따라서 review 문서와 처리 계획서를 PR head에 포함하고,
작업지시자가 GitHub merge 버튼으로 최종 merge를 수행한다.

## 2. 변경 범위

| 파일 | 내용 |
|---|---|
| `mydocs/manual/pr_review_workflow.md` | maintainer 일반 경로와 collaborator self-merge 후보 예외 경로 분리 |
| `mydocs/orders/20260621.md` | #1425 작업 기록 및 최신 devel의 PR #1447 주문서와 충돌 해소 |
| `mydocs/plans/task_m100_1425.md` | 수행계획서 |
| `mydocs/plans/task_m100_1425_impl.md` | 구현계획서 |
| `mydocs/report/task_m100_1425_report.md` | 최종 보고서 |
| `mydocs/pr/archives/pr_1449_review.md` | 본 PR review 문서 |
| `mydocs/pr/archives/pr_1449_review_impl.md` | 본 PR 처리 계획 |

## 3. 핵심 확인 사항

### 3.1 maintainer 기본 경로

- review 문서 기본 위치를 `mydocs/pr/pr_{N}_review.md`,
  `mydocs/pr/pr_{N}_review_impl.md`로 복구했다.
- 처리 완료 후 `mydocs/pr/archives/`로 이동하는 절차를 다시 명시했다.
- collaborator 예외 경로가 maintainer 기본 경로를 대체하지 않는다고 명시했다.

### 3.2 collaborator self-merge 후보 예외 경로

- PR 번호 생성 후 review 문서를 PR head에 포함하는 예외 경로를 별도 8장으로 분리했다.
- 처음부터 `mydocs/pr/archives/pr_{N}_review*.md`를 사용하는 조건을 collaborator self-merge 후보로 제한했다.
- `upstream` 작업 브랜치 직접 push 규칙을 예외 경로의 remote push 규칙으로 이동했다.

### 3.3 volatile 상태값 기록 규칙

- `draft`, `mergeable`, `head SHA`, `CI 상태`를 확정 사실처럼 쓰지 않도록 제한했다.
- 필요한 경우 "문서 작성 시점 참고값" 또는 "merge 전 최신 상태 확인 필요"로만 쓰도록 했다.
- 최종 merge 조건을 "PR head 최신 커밋 기준 GitHub Actions 통과 + 작업지시자 승인"으로 명시했다.

## 4. 검증

로컬 검증:

| 명령 | 결과 |
|---|---|
| `rg -n "maintainer|collaborator|Collaborator|self-merge|archives|draft|mergeable|head SHA|CI 상태|최신 GitHub Actions" mydocs/manual/pr_review_workflow.md` | 통과 |
| `git diff --check` | 통과 |

코드 변경은 없으므로 `cargo test`, `npm test`, 빌드 검증은 수행하지 않았다.

## 5. PR 상태값 기록 원칙

이 review 문서는 PR에 포함되어 merge 후에도 보존된다. 따라서 `draft`, `mergeable`, `head SHA`, `CI 상태`는
현재 확정 사실로 기록하지 않는다.

merge 전에는 다음을 최신 상태로 다시 확인한다.

- PR head 최신 커밋 기준 GitHub Actions 통과
- PR diff에 본 review 문서와 처리 계획서 포함
- 작업지시자 GitHub merge 승인

## 6. 리스크

| 리스크 | 판단 |
|---|---|
| 문서 규칙 혼선 | 낮음. 기본 경로와 예외 경로를 별도 섹션으로 분리했다. |
| collaborator 규칙 삭제 오해 | 낮음. #1420 규칙을 예외 경로로 보존했다. |
| volatile 상태값 재발 | 낮음. 금지 예시와 허용 표현을 함께 추가했다. |
| 코드 회귀 | 없음. 코드 변경 없음. |

## 7. 권고

merge 전 최신 GitHub Actions가 통과하고 작업지시자가 GitHub merge 버튼으로 승인하면 merge 가능으로 판단한다.

merge 후에는 #1425 자동 close 여부를 확인하고, auto-close가 실패하면 작업지시자 승인 후 수동 close한다.
