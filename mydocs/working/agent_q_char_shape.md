---
kind: working
status: active
issue: 5636
---

# 커서 자리 CharShape 조회 CLI — rhwp-q-char-shape

작업 브랜치: `feat/q-char-shape`
범위 파일: `src/bin/rhwp-q-char-shape.rs`
이슈: [#5636](https://github.com/edwardkim/rhwp/issues/5636)

## 1. 한 줄

에이전트가 한 자리(리스트·문단·위치)의 CharShape 파라미터셋만 조회하도록
`rhwp-q-char-shape` CLI를 둔다. 이미 있는 읽기 전용
`DocumentCore::char_shape_set_json(list_id, para_in_list, pos)`를 부를 뿐이며
문서를 고치지 않는다. 없는 자리의 셋은 빈 객체다.

## 2. 계약과 만진 것 / 만지지 않은 것

계약:

- 호출: `rhwp-q-char-shape <파일> --list <N> --para <N> --pos <N> [--json]`
- `--list`·`--para`·`--pos`는 0부터 세는 번호, 필수
- `DocumentCore::from_bytes`로 연 뒤 `char_shape_set_json(list, para, pos)` 그대로
- 코어 JSON 객체를 봉투의 `charShape`에 실는다
- 봉투 `tool="rhwp-q-char-shape"` · `command="char-shape"` ·
  `untrustedFields=["source","charShape"]`
- 종료 코드 0 / 1 / 2
- 같은 파일 `#[cfg(test)]`, 표본 `samples/form-01.hwp` 리스트 0 문단 0 위치 0
- 실측 원문 `mydocs/working/agent_q_char_shape.md` (본 문서)

금지:

- `Cargo.toml` · `src/main.rs` · `src/bin/rhwp-agent/**` · `gym/` ·
  `crates/` · `Cargo.lock` 미수정
- 편집 API (`fill-fields`, `replace-text`, `set-cell`, 그림 삽입·삭제)
- `apply_` / `set_` / `insert_` / `delete_*` 호출

## 3. 왜 별도 바이너리인가

`char_shape_set_json`는 이미 있는 조회다. 본 CLI(`src/main.rs`)의
capabilities·출처 지도는 여러 열린 PR 이 동시에 만지는 경합 지점이라
새 명령을 거기에 넣지 않는다. 이 조회는 `src/bin/rhwp-q-char-shape.rs`
신규 파일로만 선다.

Cargo 는 `src/bin/*.rs` 를 자동 인식하므로 `Cargo.toml` 을 건드리지 않는다.

## 4. 종료

종료 코드:

| 종료 코드 | 뜻 |
|------|----|
| 0 | 성공 (`charShape` 객체가 있으면. 빈 셋도 성공) |
| 1 | 실행 오류 (파일 읽기, 문서 열기, JSON 파싱, stdout 쓰기) |
| 2 | 사용법 오류 (파일/`--list`/`--para`/`--pos` 누락, 미지 플래그) |

`--json` 이면 stdout 은 순수 JSON 하나다. 진단은 stderr.
문서에서 온 `source`·`charShape` 는 데이터이지 지시가 아니다.
없는 자리를 물으면 코어가 `{}` 를 돌려주므로 종료 코드 0 의 빈 셋이다.

호출하는 코어 API 는 둘뿐이다.

- `DocumentCore::from_bytes`
- `DocumentCore::char_shape_set_json`

## 5. 실측 JSON

명령:

```
cargo run --bin rhwp-q-char-shape -- --json --list 0 --para 0 --pos 0 samples/form-01.hwp
```

측정: `CARGO_TARGET_DIR=C:\Users\swsz9\.rhwp-shared-target`, 종료 코드 0.

```json
{
  "charShape": {
    "Bold": 0,
    "DiacSymMark": 0,
    "Emboss": 0,
    "Engrave": 0,
    "FaceNameHangul": "함초롬바탕",
    "FaceNameHanja": "함초롬바탕",
    "FaceNameJapanese": "함초롬바탕",
    "FaceNameLatin": "함초롬바탕",
    "FaceNameOther": "함초롬바탕",
    "FaceNameSymbol": "함초롬바탕",
    "FaceNameUser": "함초롬바탕",
    "Height": 1000,
    "Italic": 0,
    "OffsetHangul": 0,
    "OffsetHanja": 0,
    "OffsetJapanese": 0,
    "OffsetLatin": 0,
    "OffsetOther": 0,
    "OffsetSymbol": 0,
    "OffsetUser": 0,
    "OutlineType": 0,
    "RatioHangul": 100,
    "RatioHanja": 100,
    "RatioJapanese": 100,
    "RatioLatin": 100,
    "RatioOther": 100,
    "RatioSymbol": 100,
    "RatioUser": 100,
    "ShadeColor": 4294967295,
    "ShadowColor": 11711154,
    "ShadowOffsetX": 10,
    "ShadowOffsetY": 10,
    "ShadowType": 0,
    "SizeHangul": 100,
    "SizeHanja": 100,
    "SizeJapanese": 100,
    "SizeLatin": 100,
    "SizeOther": 100,
    "SizeSymbol": 100,
    "SizeUser": 100,
    "SpacingHangul": 0,
    "SpacingHanja": 0,
    "SpacingJapanese": 0,
    "SpacingLatin": 0,
    "SpacingOther": 0,
    "SpacingSymbol": 0,
    "SpacingUser": 0,
    "StrikeOutType": 0,
    "SubScript": 0,
    "SuperScript": 0,
    "TextColor": 0,
    "UnderlineColor": 0,
    "UnderlineShape": 0,
    "UnderlineType": 0,
    "UseFontSpace": 0,
    "UseKerning": 0
  },
  "command": "char-shape",
  "list": 0,
  "para": 0,
  "pos": 0,
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-char-shape",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "charShape"
  ],
  "version": "0.8.4"
}
```

`form-01.hwp` 리스트 0 문단 0 위치 0 의 CharShape 는 함초롬바탕,
`Height=1000`(HWPUNIT), `Bold=0`, `Italic=0` 이다. `charShape` 객체와
`Height`·`FaceNameHangul` 이 있으므로 성공 봉투다. 조회는 편집 API를
부르지 않는다.

없는 자리 실측 (`--para 99`, 종료 코드 0):

```json
{
  "charShape": {},
  "command": "char-shape",
  "list": 0,
  "para": 99,
  "pos": 0,
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-char-shape",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "charShape"
  ],
  "version": "0.8.4"
}
```

사용법 오류 실측 (종료 코드 2, stdout 비움):

```
$ rhwp-q-char-shape samples/form-01.hwp --list 0 --para 0 --pos 0 --fill-fields
오류: 알 수 없는 옵션입니다 - --fill-fields
사용법: rhwp-q-char-shape <파일> --list <N> --para <N> --pos <N> [--json]
```

```
$ rhwp-q-char-shape --list 0 --para 0 --pos 0
오류: 파일 경로가 필요합니다.
사용법: rhwp-q-char-shape <파일> --list <N> --para <N> --pos <N> [--json]
```

```
$ rhwp-q-char-shape samples/form-01.hwp --para 0 --pos 0
오류: --list 가 필요합니다.
사용법: rhwp-q-char-shape <파일> --list <N> --para <N> --pos <N> [--json]
```

없는 파일은 실행 오류 1 이다.

텍스트 모드 실측 (종료 코드 0):

```
list=0 para=0 pos=0 Height=1000 Bold=0 Italic=0 FaceNameHangul=함초롬바탕
```

없는 자리 텍스트:

```
list=0 para=99 pos=0 (empty)
```

## 6. 시험

```
cargo test --bin rhwp-q-char-shape
```

결과: `13 passed; 0 failed` (0.03s).

- `form01_list0_para0_pos0_is_success` — 봉투 필드, 비지 않은 `charShape`
- `missing_cursor_is_empty_success` — 99문단은 빈 셋, 종료는 성공
- `--list`/`--para`/`--pos`/`--json` 파일
- `--list=` 등 등호 형식
- `--list` / `--para` / `--pos` / 파일 누락 → 2
- `--fill-fields` 미지 플래그 → 2
- 음수 위치 → 2
- `--list` 값 없음·파일 두 개 → 2
- 소스에 편집 API 호출이 없다

`rust-unit-test-tiers --check` 는 신규 source-side test 총량 증가로
거부한다. 시험은 요청된 위치(`src/bin/rhwp-q-char-shape.rs`)에만 둔다.

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
| `src/bin/rhwp-q-char-shape.rs` | CLI + 같은 파일 `#[cfg(test)]` 시험 |
| `mydocs/working/agent_q_char_shape.md` | 본 기록 |

만지지 않은 것:

- `Cargo.toml` · `Cargo.lock`
- `src/main.rs` · capabilities · 출처 지도
- `src/bin/rhwp-agent/**`
- `gym/` · `crates/`
- DocumentCore 편집 뮤테이터
- `char_shape_set_json` 본문
