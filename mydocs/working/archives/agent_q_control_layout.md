---
kind: working
status: active
issue: 5626
---

# 쪽 위 표·그림 배치 조회 CLI — rhwp-q-control-layout

작업 브랜치: `feat/q-control-layout`
구현 단일 경로: `src/bin/rhwp-q-control-layout.rs`
이슈: [#5626](https://github.com/edwardkim/rhwp/issues/5626)

## 1. 한 줄

에이전트가 한 쪽에 놓인 표·그림·도형의 좌표·plane·zOrder 를 받도록,
이미 있는 `DocumentCore::get_page_control_layout_native` 를 그대로 CLI 로
노출한다. 문서를 고치지 않는다. 빈 `controls` 배열은 성공이다.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- 단독 `rhwp-q-control-layout <파일> --page <N> [--json]`
- `--page` 는 0부터 세는 쪽 번호, 필수
- `DocumentCore::from_bytes` 로 연 뒤 `get_page_control_layout_native(page)` 만 호출
- 코어가 낸 JSON 을 해석해 봉투에 싣는다
- 봉투 `tool="rhwp-q-control-layout"` · `command="control-layout"` ·
  `untrustedFields=["source","controls"]`
- 빈 `controls` 배열은 성공
- 종료 코드 0 / 1 / 2
- 같은 파일 `#[cfg(test)]`, 표본 `samples/form-01.hwp` 0쪽
- 작업 기록 `mydocs/working/agent_q_control_layout.md` (이 파일)

금지:

- `Cargo.toml` · `src/main.rs` · `src/bin/rhwp-agent/**` · `gym/` ·
  `crates/` · `Cargo.lock` 수정
- 편집 API (`fill-fields`, `replace-text`, `set-cell`, 그림 삽입·삭제)

## 3. 왜 별도 바이너리인가

`get_page_control_layout_native` 는 이미 있다. 스튜디오·WASM 이 쓰는 MB 급
레이어 트리 JSON 을 다시 받아 키를 훑지 않고, 쪽 위 컨트롤 배치만 받는다.

본 CLI(`src/main.rs`)와 capabilities·출처 지도는 열린 PR 이 동시에 만지는
경합 지점이다. 이 파동은 `src/bin/rhwp-q-control-layout.rs` 한 파일만
추가한다. Cargo 가 `src/bin/*.rs` 를 자동 인식하므로 `Cargo.toml` 은
건드리지 않는다.

## 4. 계약

종료 코드:

| 코드 | 뜻 |
|------|----|
| 0 | 성공 (빈 controls 포함) |
| 1 | 실행 오류 (파일 없음, 문서 열기, 쪽 범위 밖) |
| 2 | 사용법 오류 (파일/`--page` 누락, 알 수 없는 플래그) |

`--json` 이면 stdout 은 순수 JSON 봉투 하나다. 진단은 stderr.
문서 파생 값은 `source`·`controls` 로 출처를 밝힌다.

## 5. 실측 JSON

명령:

```
cargo run --bin rhwp-q-control-layout -- --json --page 0 samples/form-01.hwp
```

실측: `CARGO_TARGET_DIR=C:\Users\swsz9\.rhwp-shared-target`, 종료 코드 0.

```json
{
  "command": "control-layout",
  "controlCount": 0,
  "controls": [],
  "page": 0,
  "pageCount": 1,
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-control-layout",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "controls"
  ],
  "version": "0.8.4"
}
```

`form-01.hwp` 0쪽에는 표·그림이 없다. `controls: []` · `controlCount: 0` 은
성공 봉투다. 빈 배열을 오류로 보지 않는다.

그림이 있는 표본 `samples/pic2.hwp` 0쪽 (종료 코드 0, `pageCount: 2`):

```json
{
  "command": "control-layout",
  "controlCount": 3,
  "controls": [
    {
      "controlIdx": 3,
      "h": 203.7,
      "paraIdx": 0,
      "plane": 2,
      "secIdx": 0,
      "stableIndex": [
        0,
        0,
        3
      ],
      "type": "image",
      "w": 161.1,
      "wrap": "square",
      "x": 473.9,
      "y": 132.0,
      "zOrder": 0
    },
    {
      "controlIdx": 2,
      "h": 394.5,
      "paraIdx": 0,
      "plane": 2,
      "secIdx": 0,
      "stableIndex": [
        0,
        0,
        2
      ],
      "type": "image",
      "w": 312.0,
      "wrap": "square",
      "x": 127.7,
      "y": 132.0,
      "zOrder": 1
    },
    {
      "controlIdx": 0,
      "h": 317.7,
      "paraIdx": 7,
      "plane": 2,
      "secIdx": 0,
      "stableIndex": [
        0,
        7,
        0
      ],
      "type": "image",
      "w": 218.4,
      "wrap": "square",
      "x": 461.9,
      "y": 620.0,
      "zOrder": 2
    }
  ],
  "page": 0,
  "pageCount": 2,
  "schemaVersion": "1.0",
  "source": "samples/pic2.hwp",
  "tool": "rhwp-q-control-layout",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "controls"
  ],
  "version": "0.8.4"
}
```

사용법 오류 실측 (종료 코드 2, stdout 비어 있음):

```
$ rhwp-q-control-layout samples/form-01.hwp
오류: --page 가 필요합니다.
사용법: rhwp-q-control-layout <파일> --page <N> [--json]
```

```
$ rhwp-q-control-layout --page 0
오류: 파일 경로가 필요합니다.
사용법: rhwp-q-control-layout <파일> --page <N> [--json]
```

쪽 범위 밖은 실행 오류 1 이다. `--page 99` 는
`오류: 쪽 컨트롤 배치를 조회할 수 없습니다 - 페이지 99을(를) 찾을 수 없습니다.`

## 6. 시험

```
cargo test --bin rhwp-q-control-layout
```

실측: `10 passed; 0 failed` (0.04s).

- `form01_page0_empty_controls_is_success` — 봉투 계약, 빈 controls 성공
- `--page`·`--json` 파일
- `--page` / 파일 누락 → 2
- `--fill-fields` 같은 편집 플래그 거부 → 2
- 음수 쪽 번호 → 2
- 99쪽 → 1
- `pic2.hwp` 0쪽 그림 3개 (파일이 있을 때)
- 소스에 `apply_` / `insert_` / `delete_` / `set_*` 없음

## 7. fmt

```
cargo fmt --all
cargo fmt --all -- --check
```

통과. rustfmt `newline_style = Unix`.

## 8. 만진 것 / 만지지 않은 것

만진 것:

| 경로 | 역할 |
|------|------|
| `src/bin/rhwp-q-control-layout.rs` | CLI + 같은 파일 계약 시험 |
| `mydocs/working/agent_q_control_layout.md` | 이 기록 |

만지지 않은 것:

- `Cargo.toml` · `Cargo.lock`
- `src/main.rs` · capabilities · 출처 지도
- `src/bin/rhwp-agent/**`
- `gym/` · `crates/`
- DocumentCore 편집 구현
- `get_page_control_layout_native` 본문
