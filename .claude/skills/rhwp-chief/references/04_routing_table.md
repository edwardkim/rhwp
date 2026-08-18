# 04. goal 라우팅 표 = 코드

playbook §4 표와 `tools/chief/service_loop.py` 의 `ROUTING_TABLE` /
`KNOWN_GOALS` 는 같은 물건이다. 한쪽만 고치면 표가 버그다.

| goal | 명령 (capabilities 이름) | 게이트 |
| --- | --- | --- |
| `diagnose` | `info` (티켓만, 추가 실행 없음) | 티켓 생성 |
| `export-text` | `export-text` | JSON 봉투 파싱 |
| `export-pdf` | `export-pdf` | 파일 + `%PDF-` |
| `export-hwpx` | `export-hwpx` | `--verify` exit 0 |
| `convert-hwp` | `convert` | `--verify` exit 0 |
| `extract-tables` | `export-tables`, `table-to-csv` | 표 수 = CSV 수 |
| `fill` | `fields` (핸들러는 `edit fill-fields`) | 봉투 3종 공백 |

`fill` 의 `needs:` 주석이 `fields` 인 이유: 바이너리가 필드 표면을 광고하는지
먼저 보고, 실행은 기존 `edit fill-fields` 다. 새 fill CLI 가 아니다.

## 광고되지 않은 명령

`Chief.available` 은 기동 시 `rhwp capabilities --json` 의 `commands[].name`.
핸들러 docstring 의 `needs:a,b` 가 이 집합에 없으면 C07 — `needs-agent`.
capabilities 조회 자체가 실패하면 `available is None` 이고 모든 goal 실행이
`needs-agent` 다. 버전 차이를 추측으로 메우지 않는다.

기존 계약 시험: `scripts/tests/test_automation_tool_contracts.py` 의
`test_chief_refuses_goal_when_capabilities_are_unknown`.

## 표에 행을 더하는 법 (커버리지)

1. needs-agent 로 같은 유형이 두 번 온다 (C13).
2. goal 이름 하나와 검증 게이트를 정의할 수 있다.
3. playbook §4 표 + `ROUTING_TABLE` + `goal_*` 핸들러를 **같은 PR**.
4. 게이트 없는 핸들러는 받지 않는다.
5. 새 `rhwp` 하위명령이 필요하면 이 스킬 PR 이 아니다.

LLM 이 "이 요청은 사실 export-text 네" 하고 표 밖 문자열을 실행하는 것은
커버리지가 아니다. 그것은 추측이다.

## off-table

`is_known_goal` 이 거짓이면 핸들러를 찾지 않고 즉시 `needs-agent`.
`getattr(..., "goal_" + ...)` 에 없는 이름이 들어가지 않게 표가 먼저 걸러낸다.
