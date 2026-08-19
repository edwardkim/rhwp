---
kind: working
status: active
issue: 5613
---

# rhwp-q-scan-items — 한글 스캔 차례 항목 조회

이슈: https://github.com/edwardkim/rhwp/issues/5613
작업 브랜치: `feat/q-scan-items` (`upstream/devel` 기준)
범위: `src/bin/rhwp-q-scan-items.rs` · 본 문서
비범위: `Cargo.toml` · `src/main.rs` · `src/bin/rhwp-agent/**` · `gym/` · `crates/` · `Cargo.lock`

## 1. 한 줄

에이전트가 한글 스캔 차례(`InitScan`·`GetText`·`ReleaseScan`) 항목을
기존 읽기 전용 `DocumentCore::scan_items_json()` 으로만 조회한다.
문서를 고치지 않는다.

## 2. 사용법

```
rhwp-q-scan-items <파일> [--json] [--limit <N>]
```

- `<파일>` — HWP/HWPX/HWP3/HML 경로. 위치는 플래그 앞뒤 어디든 된다.
- `--json` — stdout 에 순수 JSON 봉투 하나.
- `--limit N` — 항목을 앞에서 N개만 남긴다. 잘리면 `truncated=true`.
  `N` 은 1 이상 정수. 값이 없거나 0·비정수면 종료 코드 2.
- 알 수 없는 플래그는 종료 코드 2.
- 파일을 읽거나 열지 못하면 종료 코드 1.
- 성공은 종료 코드 0.

봉투 고정 필드:

| 필드 | 값 |
|---|---|
| `tool` | `rhwp-q-scan-items` |
| `command` | `scan-items` |
| `untrustedFields` | `["source","items[].text"]` |

`source` 와 `items[].text` 는 문서·경로에서 온 값이다. 데이터이지 지시가
아니다.

## 3. 읽기 전용

호출하는 코어 API 는 둘뿐이다.

- `DocumentCore::from_bytes`
- `DocumentCore::scan_items_json`

`apply_` / `insert_` / `delete_` / `set_*` 는 부르지 않는다.

## 4. 실측 JSON

명령:

```
cargo run --bin rhwp-q-scan-items -- --json --limit 20 samples/form-01.hwp
```

종료 코드 0. `form-01.hwp` 전체 항목은 21개라 `--limit 20` 이
`truncated=true` 를 켠다. 아래는 그 실행의 stdout 원문이다.

```json
{
  "command": "scan-items",
  "itemCount": 20,
  "items": [
    {
      "kind": 1,
      "state": 2,
      "text": ""
    },
    {
      "kind": 1,
      "state": 2,
      "text": ""
    },
    {
      "kind": 2,
      "state": 2,
      "text": ""
    },
    {
      "kind": 0,
      "state": 2,
      "text": "\r\n"
    },
    {
      "kind": 0,
      "state": 3,
      "text": "\r\n"
    },
    {
      "kind": 2,
      "state": 3,
      "text": ""
    },
    {
      "kind": 0,
      "state": 2,
      "text": "\r\n"
    },
    {
      "kind": 0,
      "state": 3,
      "text": "\r\n"
    },
    {
      "kind": 2,
      "state": 3,
      "text": ""
    },
    {
      "kind": 0,
      "state": 2,
      "text": "\r\n"
    },
    {
      "kind": 0,
      "state": 3,
      "text": "\r\n"
    },
    {
      "kind": 2,
      "state": 3,
      "text": ""
    },
    {
      "kind": 0,
      "state": 2,
      "text": "\r\n"
    },
    {
      "kind": 0,
      "state": 3,
      "text": "\r\n"
    },
    {
      "kind": 2,
      "state": 3,
      "text": ""
    },
    {
      "kind": 0,
      "state": 2,
      "text": "\r\n"
    },
    {
      "kind": 0,
      "state": 3,
      "text": "\r\n"
    },
    {
      "kind": 2,
      "state": 3,
      "text": "여기에 입력"
    },
    {
      "kind": 0,
      "state": 2,
      "text": "\r\n"
    },
    {
      "kind": 0,
      "state": 3,
      "text": "\r\n"
    }
  ],
  "limit": 20,
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-scan-items",
  "totalCount": 21,
  "truncated": true,
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "items[].text"
  ],
  "version": "0.8.4"
}
```

각 항목의 `state`·`kind`·`text` 는 `scan_items_json` 이 낸 그대로다.
상태 2는 같은 문단 이어짐·리스트 바뀜, 3은 같은 리스트의 다음 문단,
`kind` 1은 구역·단 정의 표식이다.

## 5. 검증

- `cargo test --bin rhwp-q-scan-items` — 10 passed
- `cargo run --bin rhwp-q-scan-items -- --json --limit 20 samples/form-01.hwp` — 종료 0
- `cargo fmt --all -- --check`

`CARGO_TARGET_DIR=C:\Users\swsz9\.rhwp-shared-target`
