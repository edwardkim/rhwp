# Codex Memory Dump

덤프 시점: 2026-05-13, Asia/Seoul

이 폴더는 Codex가 현재 세션에서 들고 있는 작업 기억을 재사용 가능한 형태로 분류한 덤프이다.
`mydocs/manual/memory/`의 Claude 메모리 덤프를 읽어 Codex 작업 방식에 반영한 내용과, Task #854 진행 중 새로 확인된 세션 기억을 함께 정리한다.

## Index

- [현재 세션 상태](current_session.md)
- [운영 규칙과 작업 태도](operating_rules.md)
- [문서·Git 워크플로](docs_and_git_workflow.md)
- [검증 기준과 권위 자료](validation_policy.md)
- [HWPX to HWP 변환기 기억](hwp_hwpx_converter_memory.md)
- [Task #854 재시도 기억](task_m100_854_memory.md)
- [프로젝트·참조 메모리](references_and_project_notes.md)
- [로딩한 Claude 메모리 인벤토리](loaded_claude_memory_inventory.md)

## Highest Priority Reminder

Codex는 이 프로젝트에서 구현부터 시작하면 안 된다.

하이퍼-워터폴 순서를 따른다:

1. 이슈와 현재 브랜치 확인
2. 관련 트러블슈팅과 기존 문서 검색
3. 분석 문서 또는 계획서 작성
4. 작업지시자 승인
5. 구현
6. 테스트와 한컴 검증 준비
7. 보고서와 커밋

Task #854의 현재 교훈은 특히 강하다. HWPX 파싱, IR 매핑, rhwp-studio 렌더링은 일단 정상 영역으로 보고, 문제는 IR clone/materialize 후 HWP5 저장에서 누락되거나 잘못 매핑되는 구조를 찾는 것이다.
