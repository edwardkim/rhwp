---
kind: working
status: active
issue: 5643
---

# 쪽 글자 배치 조회 CLI — rhwp-q-text-layout

작업 브랜치: `feat/q-text-layout`
구현 단일 경로: `src/bin/rhwp-q-text-layout.rs`
이슈: [#5643](https://github.com/edwardkim/rhwp/issues/5643)

## 1. 한 줄

에이전트가 한 쪽의 글자 배치(run 좌표·charX·서체)를 받도록, 이미 있는
읽기 전용 `DocumentCore::get_page_text_layout_native(page)` 를 별도 CLI 로
감싼다. 문서를 고치지 않는다. 빈 `runs` 배열도 성공이다.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- 명령 `rhwp-q-text-layout <파일> --page <N> [--json]`
- `--page` 는 0부터 세는 쪽 번호, 필수
- `DocumentCore::from_bytes` 로 연 뒤 `get_page_text_layout_native(page)` 호출
- 코어가 낸 JSON 을 해석해 봉투에 싣는다
- 봉투 `tool="rhwp-q-text-layout"` · `command="text-layout"` ·
  `untrustedFields=["source","runs"]`
- 빈 `runs` 배열은 성공
- 종료 코드 0 / 1 / 2
- 같은 파일 `#[cfg(test)]`, 표본 `samples/form-01.hwp` 0쪽
- 작업 기록 `mydocs/working/agent_q_text_layout.md` (이 파일)

금지:

- `Cargo.toml` · `src/main.rs` · `src/bin/rhwp-agent/**` · `gym/` ·
  `crates/` · `Cargo.lock` 수정
- 편집 API (`fill-fields`, `replace-text`, `set-cell`, 그림 삽입·삭제)

## 3. 왜 별도 바이너리인가

`get_page_text_layout_native` 는 이미 있다. 소비자가 레이어 트리 JSON 을
다시 받아 TextRun 키를 훑지 않고, 한 쪽의 글자 배치만 받는다.

본 CLI(`src/main.rs`)와 capabilities·출처 지도는 열린 PR 이 동시에 만지는
경합 지점이다. 이 파동은 `src/bin/rhwp-q-text-layout.rs` 한 파일만
추가한다. Cargo 가 `src/bin/*.rs` 를 자동 인식하므로 `Cargo.toml` 은
건드리지 않는다.

## 4. 계약

종료 코드:

| 코드 | 뜻 |
|------|----|
| 0 | 성공 (빈 runs 포함) |
| 1 | 실행 오류 (파일 없음, 문서 열기, 쪽 범위 밖) |
| 2 | 사용법 오류 (파일/`--page` 누락, 알 수 없는 플래그) |

`--json` 이면 stdout 은 순수 JSON 봉투 하나다. 진단은 stderr.
문서 파생 값은 `source`·`runs` 로 출처를 밝힌다.

호출하는 코어 API 는 둘뿐이다.

- `DocumentCore::from_bytes`
- `DocumentCore::get_page_text_layout_native`

`apply_` / `insert_` / `delete_` / `set_*` 는 부르지 않는다.

## 5. 실측 JSON

명령:

```
cargo run --bin rhwp-q-text-layout -- --json --page 0 samples/form-01.hwp
```

실측: `CARGO_TARGET_DIR=C:\Users\swsz9\.rhwp-shared-target`, 종료 코드 0.

```json
{
  "command": "text-layout",
  "page": 0,
  "pageCount": 1,
  "runCount": 15,
  "runs": [
    {
      "bold": false,
      "charShapeId": 1,
      "charStart": 0,
      "charX": [
        0.0
      ],
      "fontFamily": "함초롬바탕",
      "fontSize": 13.3,
      "h": 26.5,
      "italic": false,
      "letterSpacing": 0.0,
      "paraIdx": 0,
      "paraShapeId": 12,
      "ratio": 1.0,
      "secIdx": 0,
      "strikethrough": false,
      "text": "",
      "textColor": "#000000",
      "underline": false,
      "w": 566.9,
      "x": 113.4,
      "y": 132.3
    },
    {
      "bold": false,
      "charShapeId": 1,
      "charStart": 0,
      "charX": [
        0.0
      ],
      "fontFamily": "함초롬바탕",
      "fontSize": 13.3,
      "h": 13.3,
      "italic": false,
      "letterSpacing": 0.0,
      "paraIdx": 1,
      "paraShapeId": 12,
      "ratio": 1.0,
      "secIdx": 0,
      "strikethrough": false,
      "text": "",
      "textColor": "#000000",
      "underline": false,
      "w": 566.9,
      "x": 113.4,
      "y": 166.7
    },
    {
      "bold": false,
      "charShapeId": 1,
      "charStart": 0,
      "charX": [
        0.0
      ],
      "fontFamily": "함초롬바탕",
      "fontSize": 13.3,
      "h": 26.5,
      "italic": false,
      "letterSpacing": 0.0,
      "paraIdx": 2,
      "paraShapeId": 12,
      "ratio": 1.0,
      "secIdx": 0,
      "strikethrough": false,
      "text": "",
      "textColor": "#000000",
      "underline": false,
      "w": 566.9,
      "x": 113.4,
      "y": 188.1
    },
    {
      "bold": false,
      "charShapeId": 1,
      "charStart": 0,
      "charX": [
        0.0
      ],
      "fontFamily": "함초롬바탕",
      "fontSize": 13.3,
      "h": 13.3,
      "italic": false,
      "letterSpacing": 0.0,
      "paraIdx": 3,
      "paraShapeId": 12,
      "ratio": 1.0,
      "secIdx": 0,
      "strikethrough": false,
      "text": "",
      "textColor": "#000000",
      "underline": false,
      "w": 566.9,
      "x": 113.4,
      "y": 222.5
    },
    {
      "bold": false,
      "charShapeId": 1,
      "charStart": 0,
      "charX": [
        0.0
      ],
      "fontFamily": "함초롬바탕",
      "fontSize": 13.3,
      "h": 19.3,
      "italic": false,
      "letterSpacing": 0.0,
      "paraIdx": 4,
      "paraShapeId": 12,
      "ratio": 1.0,
      "secIdx": 0,
      "strikethrough": false,
      "text": "",
      "textColor": "#000000",
      "underline": false,
      "w": 566.9,
      "x": 113.4,
      "y": 243.8
    },
    {
      "bold": false,
      "charShapeId": 1,
      "charStart": 0,
      "charX": [
        0.0
      ],
      "fontFamily": "함초롬바탕",
      "fontSize": 13.3,
      "h": 13.3,
      "italic": false,
      "letterSpacing": 0.0,
      "paraIdx": 5,
      "paraShapeId": 12,
      "ratio": 1.0,
      "secIdx": 0,
      "strikethrough": false,
      "text": "",
      "textColor": "#000000",
      "underline": false,
      "w": 566.9,
      "x": 113.4,
      "y": 271.2
    },
    {
      "bold": false,
      "charShapeId": 1,
      "charStart": 0,
      "charX": [
        0.0
      ],
      "fontFamily": "함초롬바탕",
      "fontSize": 13.3,
      "h": 26.5,
      "italic": false,
      "letterSpacing": 0.0,
      "paraIdx": 6,
      "paraShapeId": 12,
      "ratio": 1.0,
      "secIdx": 0,
      "strikethrough": false,
      "text": "",
      "textColor": "#000000",
      "underline": false,
      "w": 566.9,
      "x": 113.4,
      "y": 292.5
    },
    {
      "bold": false,
      "charShapeId": 1,
      "charStart": 0,
      "charX": [
        0.0
      ],
      "fontFamily": "함초롬바탕",
      "fontSize": 13.3,
      "h": 13.3,
      "italic": false,
      "letterSpacing": 0.0,
      "paraIdx": 7,
      "paraShapeId": 12,
      "ratio": 1.0,
      "secIdx": 0,
      "strikethrough": false,
      "text": "",
      "textColor": "#000000",
      "underline": false,
      "w": 566.9,
      "x": 113.4,
      "y": 327.0
    },
    {
      "bold": false,
      "charShapeId": 1,
      "charStart": 0,
      "charX": [
        0.0
      ],
      "fontFamily": "함초롬바탕",
      "fontSize": 13.3,
      "h": 26.5,
      "italic": false,
      "letterSpacing": 0.0,
      "paraIdx": 8,
      "paraShapeId": 12,
      "ratio": 1.0,
      "secIdx": 0,
      "strikethrough": false,
      "text": "",
      "textColor": "#000000",
      "underline": false,
      "w": 566.9,
      "x": 113.4,
      "y": 348.3
    },
    {
      "bold": false,
      "charShapeId": 1,
      "charStart": 0,
      "charX": [
        0.0
      ],
      "fontFamily": "함초롬바탕",
      "fontSize": 13.3,
      "h": 13.3,
      "italic": false,
      "letterSpacing": 0.0,
      "paraIdx": 9,
      "paraShapeId": 12,
      "ratio": 1.0,
      "secIdx": 0,
      "strikethrough": false,
      "text": "",
      "textColor": "#000000",
      "underline": false,
      "w": 566.9,
      "x": 113.4,
      "y": 382.7
    },
    {
      "bold": false,
      "charShapeId": 1,
      "charStart": 0,
      "charX": [
        0.0
      ],
      "fontFamily": "함초롬바탕",
      "fontSize": 13.3,
      "h": 13.3,
      "italic": false,
      "letterSpacing": 0.0,
      "paraIdx": 10,
      "paraShapeId": 12,
      "ratio": 1.0,
      "secIdx": 0,
      "strikethrough": false,
      "text": "",
      "textColor": "#000000",
      "underline": false,
      "w": 566.9,
      "x": 186.4,
      "y": 404.1
    },
    {
      "bold": false,
      "charStart": 0,
      "charX": [
        0.0
      ],
      "fontFamily": "한컴바탕",
      "fontSize": 13.3,
      "h": 13.3,
      "italic": false,
      "letterSpacing": 0.0,
      "paraIdx": 10,
      "paraShapeId": 12,
      "ratio": 1.0,
      "secIdx": 0,
      "strikethrough": false,
      "text": "",
      "textColor": "#000000",
      "underline": false,
      "w": 0.0,
      "x": 113.4,
      "y": 404.1
    },
    {
      "bold": false,
      "charX": [
        0.0,
        13.3,
        26.7,
        40.0,
        46.7,
        60.0,
        73.3
      ],
      "fontFamily": "한컴바탕",
      "fontSize": 13.3,
      "h": 13.3,
      "italic": true,
      "letterSpacing": 0.0,
      "paraShapeId": 12,
      "ratio": 1.0,
      "strikethrough": false,
      "text": "여기에 입력",
      "textColor": "#ff0000",
      "underline": false,
      "w": 73.0,
      "x": 113.4,
      "y": 404.1
    },
    {
      "bold": false,
      "charShapeId": 1,
      "charStart": 0,
      "charX": [
        0.0
      ],
      "fontFamily": "함초롬바탕",
      "fontSize": 13.3,
      "h": 13.3,
      "italic": false,
      "letterSpacing": 0.0,
      "paraIdx": 11,
      "paraShapeId": 12,
      "ratio": 1.0,
      "secIdx": 0,
      "strikethrough": false,
      "text": "",
      "textColor": "#000000",
      "underline": false,
      "w": 566.9,
      "x": 113.4,
      "y": 425.4
    },
    {
      "bold": false,
      "charShapeId": 1,
      "charStart": 0,
      "charX": [
        0.0
      ],
      "fontFamily": "함초롬바탕",
      "fontSize": 13.3,
      "h": 13.3,
      "italic": false,
      "letterSpacing": 0.0,
      "paraIdx": 12,
      "paraShapeId": 12,
      "ratio": 1.0,
      "secIdx": 0,
      "strikethrough": false,
      "text": "",
      "textColor": "#000000",
      "underline": false,
      "w": 566.9,
      "x": 113.4,
      "y": 446.7
    }
  ],
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-text-layout",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "runs"
  ],
  "version": "0.8.4"
}
```

`form-01.hwp` 0쪽은 run 15개다. 대부분 빈 `text` 자리 run 이고, 보이는
본문은 「여기에 입력」(italic, `#ff0000`, `charX` 7개) 하나다.
`runCount: 15` 는 배열 길이와 같다.

사용법 오류 실측 (종료 코드 2, stdout 비어 있음):

```
$ rhwp-q-text-layout samples/form-01.hwp
오류: --page 가 필요합니다.
사용법: rhwp-q-text-layout <파일> --page <N> [--json]
```

```
$ rhwp-q-text-layout --page 0
오류: 파일 경로가 필요합니다.
사용법: rhwp-q-text-layout <파일> --page <N> [--json]
```

```
$ rhwp-q-text-layout --page 0 --fill-fields samples/form-01.hwp
오류: 알 수 없는 옵션입니다 - --fill-fields
사용법: rhwp-q-text-layout <파일> --page <N> [--json]
```

쪽 범위 밖은 실행 오류 1 이다. `--page 99` 는
`오류: 쪽 글자 배치를 조회할 수 없습니다 - 페이지 99을(를) 찾을 수 없습니다.`

## 6. 시험

```
cargo test --bin rhwp-q-text-layout
```

실측: `11 passed; 0 failed` (0.09s).

- `form01_page0_text_layout_is_success` — 봉투 계약, run 좌표 필드
- `--page`·`--json` 파일
- `--page` / 파일 누락 → 2
- `--fill-fields` 같은 편집 플래그 거부 → 2
- 음수 쪽 번호 → 2
- `--page` 값 없음·파일 두 개 → 2
- 99쪽 → 1
- 소스에 `apply_` / `insert_` / `delete_` / `set_*` 없음

`rust-unit-test-tiers --check` 는 신규 source-side test 총량 증가로
거부한다. 시험은 요청된 위치(`src/bin/rhwp-q-text-layout.rs`)에만 둔다.

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
| `src/bin/rhwp-q-text-layout.rs` | CLI + 같은 파일 계약 시험 |
| `mydocs/working/agent_q_text_layout.md` | 이 기록 |

만지지 않은 것:

- `Cargo.toml` · `Cargo.lock`
- `src/main.rs` · capabilities · 출처 지도
- `src/bin/rhwp-agent/**`
- `gym/` · `crates/`
- DocumentCore 편집 구현
- `get_page_text_layout_native` 본문
