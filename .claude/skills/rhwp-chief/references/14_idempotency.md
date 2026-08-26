# 14. 같은 요청을 두 번 처리하지 않는다

스위치는 `result.json` 의 존재다. 내용이 `failed` 여도 존재하면 pending 이
아니다. 부분 실패를 몰래 재시도하지 않는다.

```
pending = is_dir AND request.json AND NOT result.json
```

`is_already_processed(req_dir)` 가 그 한 줄이다.

## 왜 내용이 아니라 존재인가

루프가 한 번 손댄 폴더는 티켓·회신·(있을 수 있는) 산출이 남는다.
같은 폴더를 다시 변환하면 산출이 덮이거나, 패닉 문서를 두 번 죽인다.
재시도는 운영자가 `result.json` 을 지우는 명시적 행위다.

## watch 루프

`--watch N` 은 N초마다 pending 을 다시 스캔한다. 이미 처리된 폴더는
보이지 않는다. 새 폴더만 들어온다.

형식 오류 요청도 `result.json` 을 남긴다 (C11/C16). 그래서 깨진 JSON 이
watch 를 매 틱마다 죽이지 않는다.
기존 계약: `test_malformed_request_is_marked_complete_without_crashing_watch_loop`.

## 에이전트 쪽 멱등

에이전트가 needs-agent 를 처리한 뒤 `result.json` 을 갱신하면 루프는
그 폴더를 다시 집어가지 않는다. 에이전트가 `done` 폴더를 열어 재실행하는
것은 playbook 위반이다.
