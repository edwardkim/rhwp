---
kind: canonical
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-07-16
---

# Documentation And Git Workflow

> 이 문서는 문서·Git 작업의 공통 절차를 다룬다. PR 검토·merge·후속 처리의 역할별 규칙은
> [PR 리뷰·통합 워크플로우](../pr_review_workflow.md)를 우선한다. 현재 세션이나 종료된 작업의 상태는
> 이 문서에 기록하지 않는다.

## Document Language

모든 프로젝트 문서는 한국어로 작성한다.

## Working Document Naming

단계별 작업 문서:

```text
mydocs/working/task_m100_{issue}_stage{N}.md
```

예:

```text
mydocs/working/task_m100_854_stage1.md
mydocs/working/task_m100_854_rebuild_stage4.md
```

최종 보고서:

```text
mydocs/report/task_m100_{issue}_report.md
```

오늘할일:

```text
mydocs/orders/YYYYMMDD.md
```

## Folder Roles

- `mydocs/orders/`: 오늘할일
- `mydocs/orders/archives/`: 전월 이전 오늘할일 보관 — 매월 초 전월분을 이동하고 당월분만 루트에 유지
- `mydocs/plans/`: 수행 계획서, 구현 계획서
- `mydocs/plans/archives/`: 완료된 계획서 보관 (merge 후 정리 시 이동)
- `mydocs/working/`: 단계별 완료 보고서
- `mydocs/report/`: 최종 보고서
- `mydocs/feedback/`: 작업지시자 피드백, 코드 리뷰 의견
- `mydocs/troubleshootings/`: 재발 방지용 문제 해결 기록
- `mydocs/tech/`: 기술 조사와 스펙 정리
- `mydocs/manual/`: 매뉴얼과 장기 지침
- `mydocs/manual/memory/`: 과거 사용자 피드백과 프로젝트 memory의 historical 출처
- `mydocs/manual/codex/`: Codex 부트스트랩과 현행 문서·Git 절차. 종료 세션 자료는 `archive/`에 보존

## Issue Workflow

이슈 기반 작업의 기본 순서:

1. GitHub Issue 확인 또는 생성
2. 열린 PR 확인
3. 이슈 assignee 지정
4. 작업 브랜치 생성 또는 전환
5. 역할별 절차에 따라 오늘할일 또는 PR review 문서 갱신
6. 계획서 작성
7. 작업지시자 승인
8. 구현과 테스트
9. 단계별 보고서 작성
10. 커밋
11. 작업지시자 승인 후 이슈 close

## GitHub CLI Usage

GitHub connector가 읽기는 가능하지만 mutation 권한 부족으로 403을 반환할 수 있다.
이슈 assignee 지정, 이슈/PR metadata 수정, 코멘트 작성 등 GitHub 변경 작업은
로컬 인증된 `gh` CLI를 사용한다.

예:

```bash
gh issue edit 1063 --add-assignee edwardkim -R edwardkim/rhwp
```

운영 규칙:

- connector mutation이 403으로 실패하면 `gh` CLI로 재시도한다.
- sandbox 네트워크 제한으로 `api.github.com` 연결 실패가 나면 동일 `gh` 명령을 escalation으로 재시도한다.
- `gh`로 수행한 GitHub 변경은 오늘할일, 계획서, 보고서 중 관련 문서에 기록한다.
- `gh` 사용도 하이퍼-워터폴 절차를 대체하지 않는다. 이슈 확인, 브랜치, 문서, 승인 게이트는 그대로 유지한다.

## PR Workflow

외부 기여자 PR은 내부 task와 다르게 처리한다.

문서 위치:

```text
mydocs/pr/
```

파일명:

```text
pr_{number}_review.md
pr_{number}_review_impl.md
pr_{number}_report.md
```

PR 댓글 톤은 과장하지 않는다. "정말 감사합니다", "정성스러운 PR" 같은 반복적이고 과한 표현보다 사실 중심으로 쓴다.

## Internal Task PR Approval

내부 타스크 브랜치에서 PR은 작업지시자 별도 승인 후에만 생성한다.

- "PR 준비"는 커밋, 검증 기록, PR 본문 초안, 생성 명령 준비까지를 의미한다.
- `gh pr create` 실행, Open PR 생성, Draft/Open 상태 전환은 별도 승인을 받은 뒤 진행한다.
- 실수로 승인 없이 PR을 열었으면 작업지시자 지시에 따라 즉시 close하고, 후속 진행은 승인 대기 상태로 되돌린다.
- PR 직전 전체 CI 성격의 긴 검증(`cargo test --verbose`, `cargo clippy -- -D warnings` 등)은
  focused test와 visual sweep 결과를 공유한 뒤 작업지시자 승인을 받은 경우에만 실행한다.
- 작업 전체에 대한 자동 승인 또는 `/Goal` 자동 진행 지시가 있어도 PR CI 전체 테스트 승인을
  대체하지 않는다. PR CI는 별도 명시 승인이 필요하다.

## Commit Rules

- 보고서와 오늘할일 갱신은 task 브랜치에서 소스 변경과 함께 커밋한다.
- merge 전에는 `git status`를 확인한다.
- 이슈 close 전에는 정정 commit이 `devel` 또는 대상 브랜치에 실제 포함되어 있는지 확인한다.
- 사용자가 만들었을 수 있는 변경은 임의로 되돌리지 않는다.

## Branch And PR Rule

- 로컬 작업과 검증의 기준은 최신 `upstream/devel`이다.
- 일반 변경은 작업 브랜치에서 검증한 뒤 `devel` 대상 PR로 통합한다. `upstream/devel`에 직접 push하지 않는다.
- collaborator·maintainer가 원 PR에 보정하거나 merge 후 운영 기록을 반영하는 경우에도
  [PR 리뷰·통합 워크플로우](../pr_review_workflow.md)의 역할별 경로를 따른다.
- `local/*`은 로컬 작업 이름일 뿐 원격 `devel`을 갱신하는 명령의 근거가 아니다.
