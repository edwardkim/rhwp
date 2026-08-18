# 17. service_loop.py 사용

```
python3 tools/chief/service_loop.py --queue <큐폴더> --bin <rhwp> --once
python3 tools/chief/service_loop.py --queue <큐폴더> --bin <rhwp> --watch 10
python3 tools/chief/service_loop.py --queue <큐폴더> --bin <rhwp> --timeout 30 --once
```

바이너리 탐색 순서: `--bin` → `RHWP_BIN` → `PATH` 의 `rhwp`.

## 플래그

| 플래그 | 의미 |
| --- | --- |
| `--queue` | 필수. 요청 폴더들의 부모 |
| `--bin` | rhwp 경로 |
| `--once` | pending 을 한 바퀴 돌고 종료 |
| `--watch 초` | 상시. `--once` 와 둘 중 하나 필수 |
| `--timeout` | 기본 30초. PDF/변환은 핸들러가 배수 |

둘 다 없으면 exit 2. 큐가 없거나 바이너리가 없거나 `tools/fde/triage.py` 가
없으면 exit 2.

## 표준출력

마지막에 `{"processed": {"done": N, ...}}` 한 줄. 요청별 로그는 stderr.
`--once` 의 exit 0 은 "시도 완료" — needs-agent 가 있어도 0 이다.
판정은 각 폴더의 `result.json` 에 있다.

## 이 도구는 rhwp CLI 가 아니다

rhwp 의 chief/queue 하위명령은 없다. 발명하지 않는다.
에이전트는 위 python3 명령만 친다.

## 코드 진입점

- `ROUTING_TABLE` / `KNOWN_GOALS` / `TRIAGE_SKIP_GOAL`
- `normalize_goal` · `is_known_goal` · `route_skips_goal` · `is_already_processed`
- `resolve_request_file`
- `Chief.triage` · `Chief.handle` · `goal_*`
- `process_request` · `pending_requests` · `write_response` · `main`
