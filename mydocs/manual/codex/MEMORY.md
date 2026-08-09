---
kind: memory
status: active
canonical: mydocs/manual/codex/MEMORY.md
last_verified: 2026-08-08
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

## Node 자식 프로세스 테스트의 샌드박스 가드레일

`rhwp-studio`의 일부 Node 테스트는 `spawnSync()`로 별도 Node 드라이버를 실행한다. Codex
샌드박스에서는 이 자식 프로세스 생성이 `EPERM`으로 차단될 수 있으므로, 이런 테스트가 포함된
`npm test`는 처음부터 샌드박스 밖에서 실행한다.

차단 시 자식의 `status`가 정상처럼 보이면서 `stdout`·`stderr`가 비고, 부모 테스트에는 "결과 JSON
없음" 또는 "성공 마커 없음"만 나타날 수 있다. 이 패턴을 코드 결함으로 분류하거나 같은 sandbox
명령을 반복하지 않는다. 필요하면 작은 `spawnSync` 진단으로 `error: EPERM`을 한 번 확인한 뒤 전체
테스트를 escalation으로 재실행하고, 그 결과를 공식 판정으로 기록한다.

2026-08-08 WSL2 Node v24.15.0 환경에서 sandbox 실행은 해당 드라이버 테스트 5건이 위 패턴으로
실패했고, 동일 `npm test`를 sandbox 밖에서 실행하자 802/802건이 통과했다.
