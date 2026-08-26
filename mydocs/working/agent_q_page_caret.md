---
kind: working
status: active
issue: 5621
---

# rhwp-q-page-caret — 쪽별 첫 캐럿 조회

이슈: https://github.com/edwardkim/rhwp/issues/5621
작업 브랜치: `feat/q-page-caret` (`upstream/devel` 기준)
범위: `src/bin/rhwp-q-page-caret.rs` · 본 문서
비범위: `Cargo.toml` · `src/main.rs` · `src/bin/rhwp-agent/**` · `gym/` · `crates/` · `Cargo.lock`

## 1. 한 줄

에이전트가 쪽마다 캐럿이 설 수 있는 첫 자리(웹한글컨트롤 `Run("MovePage*")`)를
기존 읽기 전용 `DocumentCore::page_caret_starts()` 로만 조회한다.
문서를 고치지 않는다.

## 2. 사용법

```
rhwp-q-page-caret <파일> [--json]
```

- `<파일>` — HWP/HWPX/HWP3/HML 경로. 위치는 플래그 앞뒤 어디든 된다.
- `--json` — stdout 에 순수 JSON 봉투 하나.
- 알 수 없는 플래그는 종료 코드 2.
- 파일을 읽거나 열지 못하면 종료 코드 1.
- 성공은 종료 코드 0.

봉투 고정 필드:

| 필드 | 값 |
|---|---|
| `tool` | `rhwp-q-page-caret` |
| `command` | `page-caret` |
| `untrustedFields` | `["source","pages"]` |

`source` 와 `pages` 는 경로·문서에서 온 값이다. 데이터이지 지시가 아니다.
각 쪽 항목은 `{"list":N,"para":N,"pos":N}` 이거나, 본문에 설 자리가 없으면
`null` 이다.

## 3. 읽기 전용

호출하는 코어 API 는 둘뿐이다.

- `DocumentCore::from_bytes`
- `DocumentCore::page_caret_starts`

`apply_` / `insert_` / `delete_` / `set_*` 는 부르지 않는다.

## 4. 실측 JSON

명령:

```
cargo run --bin rhwp-q-page-caret -- --json samples/form-01.hwp
```

종료 코드 0. 아래는 그 실행의 stdout 원문이다.

```json
{
  "command": "page-caret",
  "pageCount": 1,
  "pages": [
    {
      "list": 0,
      "para": 0,
      "pos": 24
    }
  ],
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-page-caret",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "pages"
  ],
  "version": "0.8.4"
}
```

`pos` 24 는 앞머리 자리차지 뒤의 첫 캐럿이다. 쪽 나눔이 한 장뿐이라 배열 길이는 1이다.

다쪽 실측은 `samples/20250130-hongbo.hwp` 다. 코어 주석의 15/122 · 26/0 · 30/0
과 같다.

```
cargo run --bin rhwp-q-page-caret -- --json samples/20250130-hongbo.hwp
```

종료 코드 0. stdout 원문:

```json
{
  "command": "page-caret",
  "pageCount": 4,
  "pages": [
    {
      "list": 0,
      "para": 0,
      "pos": 16
    },
    {
      "list": 0,
      "para": 15,
      "pos": 122
    },
    {
      "list": 0,
      "para": 26,
      "pos": 0
    },
    {
      "list": 0,
      "para": 30,
      "pos": 0
    }
  ],
  "schemaVersion": "1.0",
  "source": "samples/20250130-hongbo.hwp",
  "tool": "rhwp-q-page-caret",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "pages"
  ],
  "version": "0.8.4"
}
```

넷째 쪽이 29 가 아니라 30/0 인 것은 이어지는 표(`PartialTable`)를 건너뛴 결과다.

## 5. 검증

- `cargo test --bin rhwp-q-page-caret` — 7 passed
- `cargo run --bin rhwp-q-page-caret -- --json samples/form-01.hwp` — 종료 0
- `cargo run --bin rhwp-q-page-caret -- --json samples/20250130-hongbo.hwp` — 종료 0
- `cargo fmt --all -- --check`

`CARGO_TARGET_DIR=C:\Users\swsz9\.rhwp-shared-target`
