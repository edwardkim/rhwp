# 01. 큐 프로토콜

정본은 playbook §2. 루프는 이 폴더 모양만 본다.

```
큐폴더/
  2026-08-18-001/
    request.json          ← 필수. 이것이 있어야 pending
    공문.hwpx              ← request.doc 가 가리키는 파일
    values.json           ← fill 이면 params.data
    result.json           ← 있으면 처리됨. 루프가 다시 열지 않음
    response.md
    ticket.json
    out/
      공문.pdf
```

## 누가 무엇을 쓰나

| 행위자 | 쓰는 것 | 읽기만 |
| --- | --- | --- |
| 고객/상위 시스템 | `request.json`, 문서, (fill) 값 파일 | — |
| `service_loop.py` | `result.json`, `response.md`, `ticket.json`, `out/` | request + 문서 |
| rhwp-chief 에이전트 | needs-agent 폴더의 `result.json`·`response.md` 갱신 | 루프가 `done` 한 폴더 |

에이전트는 `done` / `escalated` / `invalid-input` 폴더를 재처리하지 않는다.
루프의 판정을 존중한다.

## pending 의 정의

디렉터리이고, `request.json` 이 있고, `result.json` 이 **없다**.
이름 순(`sorted`)으로 처리한다. 숨김 폴더·파일이어도 이 두 조건이면 pending.

```python
# tools/chief/service_loop.py
def pending_requests(queue):
    for d in sorted(queue.iterdir()):
        if d.is_dir() and (d / "request.json").is_file() and not is_already_processed(d):
            yield d
```

같은 요청을 두 번 처리하지 않는 유일한 스위치가 `result.json` 이다.
회신만 고치고 다시 돌리고 싶으면 `result.json` 을 지우는 것이 명시적 재시도다.
루프는 재시도 API 를 제공하지 않는다.

## 경로 감옥

`doc` 와 `params.data` 는 요청 폴더 안 상대 경로만 된다.

- `공문.hwpx` — 허용
- `docs/공문.hwpx` — 허용 (폴더 안 하위)
- `../다른큐/공문.hwpx` — 거부 (C02)
- `/var/secret.hwp` — 거부 (C02)
- `C:\\Users\\...` — 거부 (절대경로)

`resolve_request_file` 이 `Path.is_relative_to` 로 막는다. 심링크가 폴더 밖으로
나가도 resolve 후 거절된다.

## 한 요청 = 한 문서

폴더에 문서가 여러 개여도 루프는 `request.doc` 하나만 연다. 나머지 파일은
데이터(값 JSON, 첨부)로만 취급한다. 폴더 전체를 일괄 변환하는 일은
`rhwp-bulk-pipeline` 층이다.
