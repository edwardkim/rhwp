# 00 — explain / capabilities / explore

세 명령은 이름이 비슷해 보이지만 **질문이 다르다**. 에이전트가 축을
섞으면 표가 없는 문서에 `export-tables` 를 치거나, 도구 카탈로그에서
문서별 다음 수를 고른다.

| 축 | 질문 | 명령 | 문서 의존 |
| --- | --- | --- | --- |
| explain | 이 문서가 무엇인가 | `rhwp explain <파일> --json` | 서술 (형식·쪽·표·누름틀 목록) |
| capabilities | 도구가 일반적으로 무엇을 하는가 | `rhwp capabilities --json` | 아니오 |
| explore | 이 문서로 무엇을 할 수 있는가 | `rhwp explore <파일> --json` | 예. 메뉴가 문서마다 다름 |

## 한 줄로

- `explain` 은 네 조회 값을 사람 문장으로 옮긴다. 표 이름·누름틀 이름을
  나열한다. 다음 명령을 고르지 않는다.
- `capabilities` 는 바이너리가 노출하는 도구 목록이다. 지금 연 파일과
  무관하다.
- `explore` 는 그 파일이 켜는 행동만 순위 매긴 메뉴다. `command` 와
  `skill` 이 다음 수다.

## 잘못된 첫 수

| 실수 | 왜 틀리나 | 바른 축 |
| --- | --- | --- |
| `capabilities` 를 열고 표를 고른다 | 도구 일반이다 | explore |
| `explain` 만 보고 채운다 | 무엇인지만 안다 | explore → form-fill |
| `export-text` 로 본문을 퍼낸다 | 주입이 지시처럼 읽힌다 | explore, 보안이 있으면 스윕 |
| `info` 로 쪽수만 본다 | 어포던스가 없다 | explore |

## 같이 쓸 때

문서가 무엇인지 **그리고** 무엇을 할 수 있는지 둘 다 필요하면
`explore` 를 먼저 치고, 사람이 서술을 원하면 `explain` 을 나중에 친다.
`explore` 의 `note-structure` 항목이 가리키는 명령이 바로 `explain` 이다.

`capabilities --mcp` 의 `hwp_explore` 는 CLI `explore --json` 과 같은
봉투 키를 쓴다. 도구 이름을 발명하지 않는다.

이 장은 세 기존 명령의 역할만 가른다. 새 축을 만들지 않는다.
