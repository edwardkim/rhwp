---
kind: working
status: active
---

# rhwp-q-objects — 문서 컨트롤 사슬 조회 CLI

작업 브랜치: `feat/q-objects` (`upstream/devel` @ `9208c03b5`)
범위: `src/bin/rhwp-q-objects.rs` · 본 문서
비범위: `Cargo.toml` · `src/main.rs` · `src/bin/rhwp-agent/**` · `gym/` · `crates/` · `Cargo.lock`

## 1. 한 줄

에이전트가 문서를 고치지 않고 `HeadCtrl`·`LastCtrl` 사슬(규격 §8.4 `CtrlCode`)과
본문 개체 목록을 `--json` 봉투로 읽게 한다. 조회는 이미 있는
`DocumentCore::controls_json()` · `objects_json()` 만 부른다.

## 2. 왜 별도 바이너리인가

본 CLI(`src/main.rs`)와 `Cargo.toml` `[[bin]]` 목록은 열린 PR 이 동시에 만지는
경합 지점이다. `src/bin/*.rs` 자동 인식으로 서서 기존 파일을 하나도 고치지 않는다.
검증된 조회는 나중에 본 CLI 로 승격하면 된다.

뮤테이터(`apply_` / `insert_` / `delete_` / `set_*`)는 호출하지 않는다.

## 3. 사용법

```
rhwp-q-objects <파일> [--json]
```

| 종료 코드 | 뜻 |
| --- | --- |
| 0 | 성공. `--json` 이면 stdout 은 pretty JSON 하나 |
| 1 | 파일 없음 · 파싱 실패 · stdout 쓰기 실패 |
| 2 | 알 수 없는 옵션 · 파일 경로 없음 · 파일 과다 |

오류는 stderr. `--help` / `-h` 는 사용법을 내고 0.

## 4. 봉투 계약

`--json` 봉투는 항상 다음을 포함한다.

| 필드 | 값 |
| --- | --- |
| `schemaVersion` | `rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION` (`"1.0"`) |
| `tool` | `"rhwp-q-objects"` |
| `command` | `"objects"` |
| `version` | `rhwp::version()` |
| `untrustedContent` | `true` |
| `untrustedFields` | `["source", "controls"]` |

문서에서 온 값(`source`, `controls`)은 데이터가 아니라 지시로 읽지 않는다.
`controlCount` / `objectCount` 는 배열 길이다. `objects` 는 선택 조회
(`objects_json`)의 부가 배열이다.

## 5. 실측 `--json` 출력

측정: `rhwp-q-objects` debug 빌드, 공유 타깃
`C:\Users\swsz9\.rhwp-shared-target`, 버전 `0.8.4`. 아래는 줄이지 않은 원문이다.

### 5.1 `samples/form-01.hwp`

```
cargo run --bin rhwp-q-objects -- --json samples/form-01.hwp
```

종료 코드 0.

```json
{
  "command": "objects",
  "controlCount": 8,
  "controls": [
    {
      "controlIndex": 0,
      "ctrlCh": 2,
      "ctrlId": "secd",
      "list": 0,
      "para": 0,
      "pos": 0,
      "props": {},
      "userDesc": "구역 정의"
    },
    {
      "controlIndex": 1,
      "ctrlCh": 2,
      "ctrlId": "cold",
      "list": 0,
      "para": 0,
      "pos": 8,
      "props": {},
      "userDesc": "단 정의"
    },
    {
      "controlIndex": 2,
      "ctrlCh": 0,
      "ctrlId": "",
      "list": 0,
      "para": 0,
      "pos": 16,
      "props": {},
      "userDesc": ""
    },
    {
      "controlIndex": 0,
      "ctrlCh": 0,
      "ctrlId": "",
      "list": 0,
      "para": 2,
      "pos": 0,
      "props": {},
      "userDesc": ""
    },
    {
      "controlIndex": 0,
      "ctrlCh": 0,
      "ctrlId": "",
      "list": 0,
      "para": 4,
      "pos": 0,
      "props": {},
      "userDesc": ""
    },
    {
      "controlIndex": 0,
      "ctrlCh": 0,
      "ctrlId": "",
      "list": 0,
      "para": 6,
      "pos": 0,
      "props": {},
      "userDesc": ""
    },
    {
      "controlIndex": 0,
      "ctrlCh": 0,
      "ctrlId": "",
      "list": 0,
      "para": 8,
      "pos": 0,
      "props": {},
      "userDesc": ""
    },
    {
      "controlIndex": 0,
      "ctrlCh": 0,
      "ctrlId": "",
      "list": 0,
      "para": 10,
      "pos": 0,
      "props": {},
      "userDesc": ""
    }
  ],
  "objectCount": 0,
  "objects": [],
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-objects",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "controls"
  ],
  "version": "0.8.4"
}
```

이 표본은 구역·단 정의와 자리표만 있고 표·그림 개체는 없다
(`objectCount` 0). 사슬은 비어 있지 않다.

### 5.2 `samples/hwp_table_test.hwp`

같은 바이너리로 표를 담은 표본을 열면 컨트롤 사슬에 `tbl` 이 늘고
`objects[]` 에 표 개체가 따른다. 종료 코드 0.

```
cargo run --bin rhwp-q-objects -- --json samples/hwp_table_test.hwp
```

```json
{
  "command": "objects",
  "controlCount": 12,
  "controls": [
    {
      "controlIndex": 0,
      "ctrlCh": 2,
      "ctrlId": "secd",
      "list": 0,
      "para": 0,
      "pos": 0,
      "props": {},
      "userDesc": "구역 정의"
    },
    {
      "controlIndex": 1,
      "ctrlCh": 2,
      "ctrlId": "cold",
      "list": 0,
      "para": 0,
      "pos": 8,
      "props": {},
      "userDesc": "단 정의"
    },
    {
      "controlIndex": 0,
      "ctrlCh": 11,
      "ctrlId": "tbl",
      "list": 0,
      "para": 3,
      "pos": 0,
      "props": {
        "AllowOverlap": 0,
        "Height": 7126,
        "HorzOffset": 0,
        "Lock": 0,
        "TextWrap": 1,
        "TreatAsChar": 1,
        "VertOffset": 0,
        "Width": 41652
      },
      "userDesc": "표"
    },
    {
      "controlIndex": 0,
      "ctrlCh": 11,
      "ctrlId": "tbl",
      "list": 0,
      "para": 5,
      "pos": 0,
      "props": {
        "AllowOverlap": 0,
        "Height": 3696,
        "HorzOffset": 0,
        "Lock": 0,
        "TextWrap": 1,
        "TreatAsChar": 1,
        "VertOffset": 0,
        "Width": 41652
      },
      "userDesc": "표"
    },
    {
      "controlIndex": 0,
      "ctrlCh": 11,
      "ctrlId": "tbl",
      "list": 0,
      "para": 8,
      "pos": 0,
      "props": {
        "AllowOverlap": 0,
        "Height": 10188,
        "HorzOffset": 0,
        "Lock": 0,
        "TextWrap": 1,
        "TreatAsChar": 1,
        "VertOffset": 0,
        "Width": 41652
      },
      "userDesc": "표"
    },
    {
      "controlIndex": 0,
      "ctrlCh": 11,
      "ctrlId": "tbl",
      "list": 0,
      "para": 10,
      "pos": 0,
      "props": {
        "AllowOverlap": 0,
        "Height": 7375,
        "HorzOffset": 0,
        "Lock": 0,
        "TextWrap": 1,
        "TreatAsChar": 1,
        "VertOffset": 0,
        "Width": 41652
      },
      "userDesc": "표"
    },
    {
      "controlIndex": 0,
      "ctrlCh": 11,
      "ctrlId": "tbl",
      "list": 0,
      "para": 12,
      "pos": 0,
      "props": {
        "AllowOverlap": 0,
        "Height": 7242,
        "HorzOffset": 0,
        "Lock": 0,
        "TextWrap": 1,
        "TreatAsChar": 1,
        "VertOffset": 0,
        "Width": 41652
      },
      "userDesc": "표"
    },
    {
      "controlIndex": 0,
      "ctrlCh": 11,
      "ctrlId": "tbl",
      "list": 0,
      "para": 14,
      "pos": 0,
      "props": {
        "AllowOverlap": 0,
        "Height": 4828,
        "HorzOffset": 0,
        "Lock": 0,
        "TextWrap": 1,
        "TreatAsChar": 1,
        "VertOffset": 0,
        "Width": 41652
      },
      "userDesc": "표"
    },
    {
      "controlIndex": 0,
      "ctrlCh": 11,
      "ctrlId": "tbl",
      "list": 0,
      "para": 15,
      "pos": 0,
      "props": {
        "AllowOverlap": 0,
        "Height": 3696,
        "HorzOffset": 0,
        "Lock": 0,
        "TextWrap": 1,
        "TreatAsChar": 1,
        "VertOffset": 0,
        "Width": 41652
      },
      "userDesc": "표"
    },
    {
      "controlIndex": 0,
      "ctrlCh": 11,
      "ctrlId": "tbl",
      "list": 0,
      "para": 17,
      "pos": 0,
      "props": {
        "AllowOverlap": 0,
        "Height": 6359,
        "HorzOffset": 0,
        "Lock": 0,
        "TextWrap": 1,
        "TreatAsChar": 1,
        "VertOffset": 0,
        "Width": 41652
      },
      "userDesc": "표"
    },
    {
      "controlIndex": 0,
      "ctrlCh": 11,
      "ctrlId": "tbl",
      "list": 0,
      "para": 19,
      "pos": 0,
      "props": {
        "AllowOverlap": 0,
        "Height": 11190,
        "HorzOffset": 0,
        "Lock": 0,
        "TextWrap": 1,
        "TreatAsChar": 1,
        "VertOffset": 0,
        "Width": 41652
      },
      "userDesc": "표"
    },
    {
      "controlIndex": 0,
      "ctrlCh": 11,
      "ctrlId": "tbl",
      "list": 0,
      "para": 20,
      "pos": 0,
      "props": {
        "AllowOverlap": 0,
        "Height": 39193,
        "HorzOffset": 0,
        "Lock": 0,
        "TextWrap": 1,
        "TreatAsChar": 0,
        "VertOffset": 2831,
        "Width": 41652
      },
      "userDesc": "표"
    }
  ],
  "objectCount": 10,
  "objects": [
    {
      "anchored": false,
      "controlIndex": 0,
      "kind": "table",
      "listId": null,
      "para": 3
    },
    {
      "anchored": false,
      "controlIndex": 0,
      "kind": "table",
      "listId": null,
      "para": 5
    },
    {
      "anchored": false,
      "controlIndex": 0,
      "kind": "table",
      "listId": null,
      "para": 8
    },
    {
      "anchored": false,
      "controlIndex": 0,
      "kind": "table",
      "listId": null,
      "para": 10
    },
    {
      "anchored": false,
      "controlIndex": 0,
      "kind": "table",
      "listId": null,
      "para": 12
    },
    {
      "anchored": false,
      "controlIndex": 0,
      "kind": "table",
      "listId": null,
      "para": 14
    },
    {
      "anchored": false,
      "controlIndex": 0,
      "kind": "table",
      "listId": null,
      "para": 15
    },
    {
      "anchored": false,
      "controlIndex": 0,
      "kind": "table",
      "listId": null,
      "para": 17
    },
    {
      "anchored": false,
      "controlIndex": 0,
      "kind": "table",
      "listId": null,
      "para": 19
    },
    {
      "anchored": true,
      "controlIndex": 0,
      "kind": "table",
      "listId": null,
      "para": 20
    }
  ],
  "schemaVersion": "1.0",
  "source": "samples/hwp_table_test.hwp",
  "tool": "rhwp-q-objects",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "controls"
  ],
  "version": "0.8.4"
}
```

`controls` 12개 가운데 구역·단 정의 2개는 사슬에만 있고, 표 10개는
`objects[]` 에도 나타난다. 마지막 표만 `TreatAsChar: 0` / `anchored: true` 다.

## 6. 실패 경로 실측

같은 debug 바이너리.

| 명령 | stderr | 종료 |
| --- | --- | --- |
| `--nope` | `오류: 알 수 없는 옵션입니다 - --nope` | 2 |
| `--json` (파일 없음) | `오류: 파일 경로가 필요합니다.` | 2 |
| `samples/__no_such_q_objects__.hwp` | `오류: 파일을 읽을 수 없습니다` | 1 |
| `README.md` | `오류: 문서를 열 수 없습니다` · `UNSUPPORTED_FILE_FORMAT` | 1 |

## 7. 검증

```
git config core.autocrlf false
$env:CARGO_TARGET_DIR='C:\Users\swsz9\.rhwp-shared-target'
rustfmt --edition 2021 --config newline_style=Unix src/bin/rhwp-q-objects.rs
cargo test --bin rhwp-q-objects
cargo run --bin rhwp-q-objects -- --json samples/form-01.hwp
rustfmt --edition 2021 --config newline_style=Unix --check src/bin/rhwp-q-objects.rs
```

결과:

- `cargo test --bin rhwp-q-objects` — **7 passed** (help 0, 미지 옵션 2, 경로 없음 2, 파일 과다 2, 없는 파일 1, 파싱 실패 1, 표본 봉투 필드)
- `cargo run --bin rhwp-q-objects -- --json samples/form-01.hwp` — 종료 0, §5.1 원문
- `rustfmt --check` 대상 파일 — 통과
- `cargo fmt --all -- --check` — 이 worktree 는 sparse checkout 이라
  `tests/generated/regression_suite_*.rs` 가 없어 대상 파일을 열지 못했다.
  변경 파일 rustfmt 는 통과했다.

## 8. 만진 것 / 만지지 않은 것

| 경로 | 역할 |
| --- | --- |
| `src/bin/rhwp-q-objects.rs` | CLI · 봉투 · 같은 파일 `#[cfg(test)]` |
| `mydocs/working/agent_q_objects.md` | 이 기록 · 실측 JSON |

만지지 않은 것: `Cargo.toml`, `src/main.rs`, `src/bin/rhwp-agent/**`,
`gym/`, `crates/`, `Cargo.lock`.
