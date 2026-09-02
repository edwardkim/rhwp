# 재측정 — capabilities / --mcp / tools/list

이슈 #5342. 스킬 `rhwp-knowledge-map`. gym 아님. 새 CLI 없음.
이 장은 지도 행을 다시 쓰지 않는다. 앵커와 다음 점프만 적는다.

지도 §0 이 적어 둔 재확인 법이다. 개수는 계약이 아니다.
버전이 다르면 **바이너리가 이긴다**.

| ID | 명령 | 보는 것 |
| --- | --- | --- |
| RM01 | `rhwp capabilities` | 명령·플래그·recordFields·종료 코드 |
| RM02 | `rhwp capabilities --mcp` | MCP 무상태 도구 선언 |
| RM03 | `rhwp mcp-serve` + `tools/list` | 세션 포함 실제 목록 |

## tools/list 조립

지도 §0 코드 블록을 그대로 쓴다. initialize → initialized →
tools/list. 세션 도구는 `--mcp` 매니페스트에 없다.

## 검색

`rhwp capabilities --search <낱말> [--json]` 은 명령 이름·요약·
하위 명령을 찾는다. `export-agent-manifest --json` 은 네 축을
모으고 빠진 축은 `missingAxes` 로 밝힌다. 필드 이름은 여기서
발명하지 말고 매니페스트·§2 에서 가져온다.

## 하지 말 것

- 지도 §0 표의 개수를 암기해 답하기
- 바이너리 없이 개수를 손보기
- `rhwp knowledge-map` 같은 재측정 전용 명령 발명
