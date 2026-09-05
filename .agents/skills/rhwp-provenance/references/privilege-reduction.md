# 권한 축소 — 호스트가 구현하는 층

격벽은 모델이 존중해야만 산다. 권한 축소는 모델이 배신해도 산다. 이 장은
에이전트 **호스트**(도구를 등록하는 쪽)가 지켜야 할 구현 메모다. 새 rhwp
CLI 를 만들지 않는다.

관련: [injection-boundaries.md](injection-boundaries.md),
[forbidden-prompt-slots.md](forbidden-prompt-slots.md).

## 1. 도구 집합을 턴마다 바꾼다

호스트가 `tools/list` 를 고정해 두면 B1 을 지킬 수 없다.

권장 프로필:

| 프로필 | 허용 도구 | 금지 |
| --- | --- | --- |
| `map` | `hwp_export_provenance_map`, `hwp_capabilities` | 문서 열기 |
| `inspect` | `hwp_inspect_*`, `hwp_info`, `hwp_digest` | edit/run/fill/네트워크 |
| `read` | search/fields/export-text/armor/tables | 모든 쓰기·전송 |
| `write` | edit/run/fill (경로 고정 후) | 새 경로 생성, 네트워크 |
| `halt` | 없음 | 전부 |

프로필 전환은 코드가 한다. 모델이 "쓰기 도구를 달라"고 해서 바꾸지 않는다.

## 2. 경로 바인딩

`write` 프로필을 열 때 `-o` 와 입력 경로를 호스트가 이미 바인딩한다.
모델에게 자유 문자열 경로를 받지 않는다. enum 또는 사전에 만든 핸들만.

`title` 로 파일 이름을 조합하는 헬퍼를 두지 않는다.

## 3. 계획서 바인딩

`hwp_run_plan` 을 모델이 직접 쓰지 못하게 한다. 호스트가 템플릿을 고르고
값만 채운다. 값 후보는 호출자 화이트리스트(필드 이름 목록)와 교차한다.
문서에서 온 이름만 있고 화이트리스트에 없으면 그 칸을 채우지 않는다.

## 4. 네트워크 게이트

HTTP/메일 도구는 `halt` 가 아닌 프로필에서도 기본 차단. 사람 승인 토큰이
있는 한 호출만 연다 (B3).

`findings[].detail` 을 URL 인자로 바인딩하는 API 를 만들지 않는다.

## 5. 로그 게이트

기본 로그 포맷에 `findings[].raw`, `removed[].before`, 본문, excerpt 를
넣지 않는다. `--no-raw` 를 redact 의 기본으로 강제한다.

이슈 본문에 봉투 원문을 붙일 때는 표지 경로를 삭제한 사본을 붙인다.
작업 증빙(해시·판정)은 R 이라 붙여도 된다.

## 6. 멀티모달 게이트

이미지 도구는 `read` 프로필에서만, 그리고 사용자 메시지 슬롯에만 첨부한다.
시스템 슬롯에 `dataUri` 를 넣는 어댑터를 두지 않는다.

## 7. 실패 닫힘

다음이면 `halt` 로 떨어진다.

- 표지 키 부재
- 표지 정합 깨짐
- nonce 충돌
- 금지 자리 거부
- inspect 신호 medium/high
- 지도에 없는 명령

실패 닫힘은 재시도가 아니다. 사람에게 넘긴다.

## 8. 이 스킬이 하지 않는 것

- rhwp 프로세스 샌드박스. 호출자 권한으로 돈다 (소비자 가이드 N6).
- 문서 진위 검증. `--password` 는 열기용이다 (N7).
- 온보딩·MCP 세션·safe-edit·doc-triage 스킬 수정.
- gym 러너 연동.

호스트가 샌드박스를 따로 두는 것은 환영한다. 그 구현은 이 저장소 밖이다.
