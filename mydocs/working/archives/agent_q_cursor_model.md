---
kind: working
status: active
issue: 5648
---

# rhwp-q-cursor-model — 한글 커서 리스트 지도 조회

이슈: https://github.com/edwardkim/rhwp/issues/5648
작업 브랜치: `feat/q-cursor-model` (`upstream/devel` 기준)
생성: `src/bin/rhwp-q-cursor-model.rs` · 같은 파일
금지: `Cargo.toml` · `src/main.rs` · `src/bin/rhwp-agent/**` · `gym/` · `crates/` · `Cargo.lock`

## 1. 왜

에이전트가 한글 커서 좌표계(`GetPos`·`SetPos`·`MovePos`) 리스트 지도를
기존 읽기 전용 질의 `DocumentCore::get_cursor_model_json()` 만으로 조회한다.
문서 편집 API 는 쓰지 않는다.

## 2. 사용법

```
rhwp-q-cursor-model <파일> [--json]
```

- `<파일>` — HWP/HWPX/HWP3/HML 경로. 없으면 사용법 오류로 끝낸다.
- `--json` — stdout 에 순수 JSON 봉투 하나.
- 알 수 없는 옵션은 사용법 종료 코드 2.
- 파일을 읽거나 열지 못하면 실행 종료 코드 1.
- 성공은 종료 코드 0.

봉투 고정 필드:

| 필드 | 값 |
|---|---|
| `tool` | `rhwp-q-cursor-model` |
| `command` | `cursor-model` |
| `untrustedFields` | `["source","root","lists"]` |

`source` 와 `root`·`lists` 는 문서·경로에서 온 값이다. 지시가 아니라
데이터다.

## 3. 읽기 전용

아래 기존 API 만 호출한다.

- `DocumentCore::from_bytes`
- `DocumentCore::get_cursor_model_json`

`apply_` / `insert_` / `delete_` / `set_*` 는 부르지 않는다.

## 4. 실측 JSON

명령:

```
cargo run --bin rhwp-q-cursor-model -- --json samples/form-01.hwp
```

종료 코드 0. 진단은 없고 stdout 만 있다.

```json
{
  "command": "cursor-model",
  "listCount": 2,
  "lists": [],
  "root": {
    "endPara": 12,
    "endPos": 0,
    "paraCount": 13,
    "topPos": 24
  },
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-cursor-model",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "root",
    "lists"
  ],
  "version": "0.8.4"
}
```

`form-01.hwp` 는 본문 리스트만 있다. `listCount` 2 는 첫 하위 리스트
아이디이고 `lists` 는 비어 있다. `root.topPos` 24 는 문단 앞머리 자리차지
컨트롤 3개(각 8 코드 유닛)를 건너뛴 자리다.

같은 코어 주석의 영수증 표본:

```
cargo run --bin rhwp-q-cursor-model -- --json samples/issue-986-receipt.hwp
```

종료 코드 0. `listCount` 325, `root.topPos` 72, `root.paraCount` 3,
`lists` 323개. 코어 주석의 영수증 서식(자리차지 표 7개 → 72)과 같다.

```json
{
  "command": "cursor-model",
  "listCount": 325,
  "lists": [
    {
      "cellCount": 265,
      "cellIndex": 0,
      "col": 0,
      "colSpan": 13,
      "controlIndex": 2,
      "hostListId": 0,
      "hostPara": 0,
      "isCell": true,
      "listId": 2,
      "paraCount": 1,
      "row": 0,
      "rowSpan": 1,
      "sectionIndex": 0
    }
  ],
  "root": {
    "endPara": 2,
    "endPos": 0,
    "paraCount": 3,
    "topPos": 72
  },
  "schemaVersion": "1.0",
  "source": "samples/issue-986-receipt.hwp",
  "tool": "rhwp-q-cursor-model",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "root",
    "lists"
  ],
  "version": "0.8.4"
}
```

위 영수증 봉투는 첫 `lists[]` 항목만 남긴 발췌다. 전체 323개 셀 리스트는
같은 명령의 stdout 원문에 있다.

## 5. 검증

- `cargo test --bin rhwp-q-cursor-model` — 9 passed
- `cargo run --bin rhwp-q-cursor-model -- --json samples/form-01.hwp` — 종료 0
- `cargo run --bin rhwp-q-cursor-model -- --json samples/issue-986-receipt.hwp` — 종료 0
- `cargo fmt --all -- --check`

`CARGO_TARGET_DIR=C:\Users\swsz9\.rhwp-shared-target`
