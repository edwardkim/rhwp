# 25. 루프 종료 코드

`tools/chief/service_loop.py` 의 프로세스 코드다. 요청별 `result.status` 와
층을 섞지 않는다.

| 코드 | 언제 |
| --- | --- |
| 0 | `--once` 가 pending 을 한 바퀴 돌았다. needs-agent·failed 포함 |
| 1 | 루프 자체 예외 (현재 main 은 대부분 2 또는 0) |
| 2 | 큐 없음, 바이너리 없음, triage.py 없음, `--once`/`--watch` 둘 다 없음 |

요청 한 건의 실패는 프로세스 비0 이 아니다. 그 건의 `result.json` 이다.

## rhwp 쪽 코드 (핸들러가 읽는 것)

| 코드 | 의미 | Chief |
| --- | --- | --- |
| 0 | 성공 | 게이트로 진행 |
| 1 | 런타임 | 그 goal failed |
| 2 | 사용법 | failed |
| 3 | `--verify` 불일치 | export-hwpx / convert failed (C18/C19) |

fill 의 봉투 실패는 종종 exit 0 + 비어 있지 않은 `notFound` 다.
exit 만 보지 말고 봉투를 본다 (C09).

## 트리아지 엔진

triage.py 는 라우트가 escalate-bug 여도 exit 0 (티켓이 나왔으므로).
Chief 는 그 티켓의 `route` 로 갈린다. triage 비0 만 invalid-input 으로 접는다.
