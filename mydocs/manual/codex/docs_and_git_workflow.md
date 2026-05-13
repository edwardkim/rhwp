# Documentation And Git Workflow

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
- `mydocs/plans/`: 수행 계획서, 구현 계획서
- `mydocs/working/`: 단계별 완료 보고서
- `mydocs/report/`: 최종 보고서
- `mydocs/troubleshootings/`: 재발 방지용 문제 해결 기록
- `mydocs/tech/`: 기술 조사와 스펙 정리
- `mydocs/manual/`: 매뉴얼과 장기 지침
- `mydocs/manual/memory/`: Claude 메모리 덤프
- `mydocs/manual/codex/`: Codex 메모리 덤프

## Issue Workflow

이슈 기반 작업의 기본 순서:

1. GitHub Issue 확인 또는 생성
2. 열린 PR 확인
3. 이슈 assignee 지정
4. 작업 브랜치 생성 또는 전환
5. 오늘할일 문서 갱신
6. 계획서 작성
7. 작업지시자 승인
8. 구현과 테스트
9. 단계별 보고서 작성
10. 커밋
11. 작업지시자 승인 후 이슈 close

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

## Commit Rules

- 보고서와 오늘할일 갱신은 task 브랜치에서 소스 변경과 함께 커밋한다.
- merge 전에는 `git status`를 확인한다.
- 이슈 close 전에는 정정 commit이 `devel` 또는 대상 브랜치에 실제 포함되어 있는지 확인한다.
- 사용자가 만들었을 수 있는 변경은 임의로 되돌리지 않는다.

## Current Branch Memory

현재는 사용자 지시로 `local/devel`에 있다.

Task #854 재작업을 다시 시작하려면 이슈와 브랜치 절차를 다시 확인해야 한다. 곧바로 구현하지 않는다.
