---
kind: memory
status: active
canonical: mydocs/manual/codex/MEMORY.md
last_verified: 2026-08-07
---

# Codex 프로젝트 메모리 덤프

이 문서는 rhwp 작업지시자가 세션에서 확정한 프로젝트 운영 규칙을 Codex가 다음 세션에서도
재사용하도록 보존하는 활성 메모리 진입점이다. 세션의 현재 브랜치나 종료된 task 상태는 기록하지
않으며, 그런 스냅샷은 [`archive/`](archive/)에 historical 문서로 보존한다.

세부 절차가 활성 canonical manual에 반영되어 있으면 해당 manual을 함께 따른다. 현재 작업지시자의
명시적 지시와 이 덤프가 다르면 현재 지시가 우선하며, 추정으로 충돌을 해소하지 않는다.

## 유지보수자 PR 1차 트리야지

2026-08-07 작업지시자가 확정한 열린 PR 목록의 1차 트리야지는 다음 세 가지 메타데이터 작업이다.

1. PR의 `Assignees`에 해당 PR의 author를 지정한다.
2. `Milestone`을 `v1.0.0`으로 지정한다.
3. PR 제목·본문·변경 파일 등 실제 내용을 근거로 저장소의 기존 `Labels`를 추가한다.

적용 후 열린 PR 전체를 다시 조회해 세 필드의 누락 여부를 확인한다. PR 댓글, 리뷰, 브랜치 갱신,
merge, close는 별도 작업지시가 필요한 후속 단계이며 1차 트리야지에 포함하지 않는다.
