# Current Session State

## Repository

- 작업 디렉터리: `/home/edward/mygithub/rhwp`
- 사용 쉘: `bash`
- 날짜: 2026-05-13
- 시간대: `Asia/Seoul`
- 사용자는 한국어로 작업 지시 중이다.

## Branch

사용자 지시로 브랜치를 `local/devel`로 전환했다.

마지막 확인된 상태:

```text
## local/devel
```

직전에는 Task #854 브랜치가 정리된 상태였다.

```text
a510c6c1 (local/task_m100_854) fix: complete task 854 rebuild stage 4
```

이후 사용자가 요청하여 `local/devel`로 이동했다.

## Recent User Directives

최근 작업지시자의 핵심 지시:

- 현재 작업 브랜치가 오염되었으므로 clean 후 처음부터 다시 접근한다.
- 판단하지 말고, 사용자가 지운다고 하면 지운다.
- POC로 돌아가서 확인한다.
- 구현부터 하는 습관을 고쳐야 한다.
- 하이퍼-워터폴 방법론을 따른다.
- HWPX 파싱, IR 매핑, rhwp-studio 렌더링은 문제가 아니다.
- 문제는 clone/materialize 후 HWP5 저장 과정에서 빠지거나 잘못 매핑되는 것이다.
- `mydocs/manual/memory`의 Claude 메모리 덤프를 Codex 메모리로 로딩한다.
- `mydocs/manual/codex`에 Codex 메모리 전체를 카테고리별로 덤프한다.

## Current File Work

사용자 요청에 따라 `mydocs/manual/codex/` 폴더를 만들고 Codex 메모리 덤프 파일을 작성 중이다.

이 작업은 문서 덤프 작업이며, 소스 구현 작업이 아니다.
