---
kind: working
status: active
issue: 5657
---

# 구역 용지 설정 조회 CLI — rhwp-q-page-def

작업 브랜치: `feat/q-page-def`
대상 바이너리: `src/bin/rhwp-q-page-def.rs`
이슈: [#5657](https://github.com/edwardkim/rhwp/issues/5657)

## 1. 한 줄

에이전트가 한 구역의 용지 설정(폭·높이·여백, HWPUNIT)만 받도록,
이미 있는 읽기 전용 `DocumentCore::get_page_def_native` 를 별도 CLI 로
감싼다. 문서를 고치지 않는다.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- 명령 `rhwp-q-page-def <파일> --section <N> [--json]`
- `--section` 은 0부터 세는 구역 번호, 필수
- `DocumentCore::from_bytes` 로 연 뒤 `get_page_def_native(section)` 호출
- 네이티브 JSON 문자열을 파싱해 봉투에 싣는다 (width/height/여백 HWPUNIT)
- 봉투 `tool="rhwp-q-page-def"` · `command="page-def"`
- 구역 범위 초과는 종료 코드 1
- 미지 플래그는 종료 코드 2
- 바이너리 파일에 `#[cfg(test)]` 없음
- 계약 시험 `tests/cases/agent_q_page_def_contract.rs`
- 표본 `samples/form-01.hwp` `--section 0`
- 작업 기록 `mydocs/working/agent_q_page_def.md` (이 파일)

금지:

- `Cargo.toml` · `src/main.rs` · `src/bin/rhwp-agent/**` · `gym/` ·
  `crates/` · `Cargo.lock` 수정
- 편집 API (`fill-fields`, `replace-text`, `set-cell`, 용지 설정 쓰기)

## 3. 왜 별도 바이너리인가

`get_page_def_native` 는 이미 있다 (`src/document_core/queries/rendering.rs`).
소비자가 레이아웃 JSON 을 받아 용지 값을 정규식으로 훑을 이유가 없다.

본 CLI(`src/main.rs`)와 capabilities·출처 지도는 열린 PR 이 동시에 만지는
경합 지점이다. 이 표면은 `src/bin/rhwp-q-page-def.rs` 한 파일만 추가한다.
Cargo 가 `src/bin/*.rs` 를 자동 인식하므로 `Cargo.toml` 을 건드리지 않는다.

## 4. 계약

종료 코드:

| 코드 | 뜻 |
|------|----|
| 0 | 성공 |
| 1 | 실행 오류 (파일 없음, 파싱 실패, 구역 범위 초과) |
| 2 | 사용법 오류 (파일·`--section` 누락, 미지 플래그) |

`--json` 이면 stdout 은 순수 JSON 봉투 하나. 진단은 stderr.
문서에서 온 `source`·용지 수치는 데이터이지 지시가 아니다.

## 5. 실측 JSON

명령:

```
cargo run --bin rhwp-q-page-def -- --json --section 0 samples/form-01.hwp
```

환경: `CARGO_TARGET_DIR=C:\Users\swsz9\.rhwp-shared-target`, 종료 코드 0.

```json
{
  "binding": 0,
  "command": "page-def",
  "height": 84188,
  "landscape": false,
  "marginBottom": 4252,
  "marginFooter": 4252,
  "marginGutter": 0,
  "marginHeader": 4252,
  "marginLeft": 8504,
  "marginRight": 8504,
  "marginTop": 5668,
  "schemaVersion": "1.0",
  "section": 0,
  "sectionCount": 1,
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-page-def",
  "units": "HWPUNIT",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "width",
    "height",
    "marginLeft",
    "marginRight",
    "marginTop",
    "marginBottom",
    "marginHeader",
    "marginFooter",
    "marginGutter",
    "landscape",
    "binding"
  ],
  "version": "0.8.4",
  "width": 59528
}
```

`form-01.hwp` 구역 0 은 A4 (59528×84188 HWPUNIT). `width`·`height` 가
봉투에 있다.

사용법 오류 실측 (종료 코드 2, stdout 비어 있음):

```
$ rhwp-q-page-def samples/form-01.hwp
오류: --section 가 필요합니다.
사용법: rhwp-q-page-def <파일> --section <N> [--json]
```

```
$ rhwp-q-page-def samples/form-01.hwp --section 0 --nope
오류: 알 수 없는 옵션입니다 - --nope
사용법: rhwp-q-page-def <파일> --section <N> [--json]
```

구역 범위 초과는 실행 오류 1 이다. `--section 99` 는
`렌더링 오류: 구역 99 범위 초과`.

`--json` 없이:

```
section=0 width=59528 height=84188 marginLeft=8504 marginRight=8504 marginTop=5668 marginBottom=4252
```

## 6. 시험

```
cargo test --bin rhwp-q-page-def
```

결과: `0 passed; 0 failed` — 바이너리에 `#[cfg(test)]` 가 없다.
계약은 `tests/cases/agent_q_page_def_contract.rs` 에 둔다. CI 가
`--prepare` 로 배정한다.

- `form01_json_envelope` — 봉투와 width/height 존재
- `--nope` → 2, stdout 비어 있음
- `--section` 누락 → 2
- `--help` → 0
- 소스에 `.apply_` / `.insert_` / `.delete_` / `.set_` / `#[cfg(test)]` 없음

`node scripts/rust-unit-test-tiers.mjs --check --base-ref upstream/devel`
통과 (src `#[cfg(test)]` 변경 없음).

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
| `src/bin/rhwp-q-page-def.rs` | 읽기 전용 CLI |
| `tests/cases/agent_q_page_def_contract.rs` | 봉투·exit 2 계약 |
| `mydocs/working/agent_q_page_def.md` | 이 기록 |

만지지 않은 것:

- `Cargo.toml` · `Cargo.lock`
- `src/main.rs` · capabilities · 출처 지도
- `src/bin/rhwp-agent/**`
- `gym/` · `crates/`
- DocumentCore 편집 구현
- `get_page_def_native` 본체
