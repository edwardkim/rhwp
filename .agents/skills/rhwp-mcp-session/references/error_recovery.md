# 오류 복구 — 세 층을 혼동하지 않는다

권위: `mydocs/manual/mcp_integration_guide.md`, 실패 사전 §14.

| 층 | 신호 | 재시도 | 다음 수 |
|---|---|---|---|
| JSON-RPC | `error{code,message}` | 금지 | 메시지/메서드/params 를 고친다 |
| 도구 실패 | `isError:true` | 닫힌 핸들만 | 필수 인자·경로·프로필을 고친다. exit 2 는 재시도 금지 |
| 봉투 판정 | `isError:false` + 필드 | 조건부 | `identical`/`notFound`/`invalid`/`nextOffset` 을 읽는다 |

## 층 1 — JSON-RPC

| code | 뜻 | 복구 |
|---:|---|---|
| -32700 | 줄이 JSON 이 아님 | 한 줄 한 객체. 로그를 stdout 에 섞지 않는다 |
| -32600 | 요청 구조 오류 | jsonrpc 2.0 필드 |
| -32601 | 메서드 없음 | 지원 목록만 |
| -32602 | params 구조 오류 | `params.name` 필수 |
| -32002 | 리소스 없음 | `resources/list` 로 URI 재확인 |

클라이언트가 `2024-11-05` 를 보내도 서버는 `2025-06-18` 로 응답할 수 있다.
버전 불일치를 하드 실패로 보면 핸드셰이크에서 막힌다.

## 층 2 — isError

실측 바늘:

- `알 수 없는 도구` + `didYouMean[]` + `nextCall`
- `path 가 필요합니다` / `docId 가 필요합니다` / `query 가 필요합니다`
- `열려 있지 않은 핸들: doc-1 (hwp_open 먼저)` + `nextCall{name:"hwp_open"}`
- `종료 코드 1:` 파일·권한·파싱 — 입력을 고친다
- `종료 코드 2:` 사용법 — 인자를 고친다. 같은 호출 재시도 금지
- `현재 프로필에서는 세션 도구를 제공하지 않습니다`

닫힌 핸들만 자동 복구 루프다: `hwp_open` → 새 `docId` → 원래 조회.
`hwp_close` 의 만료 문구에는 `(hwp_open 먼저)` 가 없을 수 있다.
매칭 키는 `열려 있지 않은 핸들` 이다.

## 층 3 — 봉투 (오류가 아닌 데이터)

| 필드 | 도구 | 게이트 |
|---|---|---|
| `identical:false` | `hwp_ir_diff` | 차이 발견. 사람 큐 |
| `notFound` / `ambiguous` | `hwp_fill_fields` / `hwp_doc_fill_fields` | 이름·순번을 고친다 |
| `replacedCount:0` | replace 계열 | 찾기 실패 보고 |
| `overflow` | set_cell 계열 | 칸은 쓰임. 넘침 보고 |
| `invalid != []` | `hwp_run_plan` | MCP 는 isError:false |
| `truncated` + `nextOffset` | text/search/extract | 이어보기. nextOffset 없으면 끝 |
| `verify.identical:false` | save/convert | 저장은 됐고 IR 차이 |

`content[0].text` 는 문자열화된 JSON 이다. 단건 도구는 `structuredContent` 를 써라.
배치만 `structuredContent=null`.

## 복구 의사코드

```
if response.error:                 # JSON-RPC
    fix protocol; do not retry
elif result.isError:
    body = parse(content[0].text)
    if "열려 있지 않은 핸들" in body:
        call nextCall              # hwp_open
        retry original with new docId
    elif body.nextCall:
        inspect; do not invent
    else:
        fix args; do not retry same bytes
else:
    env = structuredContent or parse(text)
    gate on env fields
```

## 상대 경로

`path` 는 서버 cwd 기준이다. MCP 로는 항상 절대 경로.
