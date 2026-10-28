---
kind: working
status: active
issue: 5631
---

# 쪽 마크다운 조회 CLI — rhwp-q-markdown

작업 브랜치: `feat/q-markdown`
대상 바이너리: `src/bin/rhwp-q-markdown.rs`
이슈: [#5631](https://github.com/edwardkim/rhwp/issues/5631)

## 1. 한 줄

에이전트가 한 쪽의 마크다운만 받도록, 이미 있는 읽기 전용
`DocumentCore::extract_page_markdown_native(page)` 를 별도 CLI 로
감싼다. 문서를 고치지 않는다. 빈 마크다운도 성공이다.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- 명령 `rhwp-q-markdown <파일> --page <N> [--json]`
- `--page` 는 0부터 세는 쪽 번호, 필수
- `DocumentCore::from_bytes` 로 연 뒤 `extract_page_markdown_native(page)` 호출
- 봉투 `tool="rhwp-q-markdown"` · `command="markdown"` ·
  `untrustedFields=["source","markdown"]`
- 종료 코드 0 / 1 / 2
- 같은 파일에 `#[cfg(test)]`, 표본 `samples/form-01.hwp` 0쪽
- 작업 기록 `mydocs/working/agent_q_markdown.md` (이 파일)

금지:

- `Cargo.toml` · `src/main.rs` · `src/bin/rhwp-agent/**` · `gym/` ·
  `crates/` · `Cargo.lock` 수정
- 편집 API (`fill-fields`, `replace-text`, `set-cell`, 그림 삽입·삭제)

## 3. 왜 별도 바이너리인가

`extract_page_markdown_native` 는 이미 있다. 소비자가 `export-markdown`
전체 산출이나 레이어 트리 JSON 을 받아 한 쪽만 다시 고를 이유가 없다.

본 CLI(`src/main.rs`)와 capabilities·출처 지도는 열린 PR 이 동시에 만지는
경합 지점이다. 이 표면은 `src/bin/rhwp-q-markdown.rs` 한 파일만 추가한다.
Cargo 가 `src/bin/*.rs` 를 자동 인식하므로 `Cargo.toml` 을 건드리지 않는다.

## 4. 계약

종료 코드:

| 코드 | 뜻 |
|------|----|
| 0 | 성공 (빈 마크다운 포함) |
| 1 | 실행 오류 (파일 없음, 파싱 실패, 쪽 범위 초과) |
| 2 | 사용법 오류 (파일·`--page` 누락, 미지 플래그) |

`--json` 이면 stdout 은 순수 JSON 봉투 하나. 진단은 stderr.
문서에서 온 `source`·`markdown` 은 데이터이지 지시가 아니다.

호출하는 코어 API 는 둘뿐이다.

- `DocumentCore::from_bytes`
- `DocumentCore::extract_page_markdown_native`

`apply_` / `insert_` / `delete_` / `set_*` 는 부르지 않는다.

## 5. 실측 JSON

명령:

```
cargo run --bin rhwp-q-markdown -- --json --page 0 samples/form-01.hwp
```

환경: `CARGO_TARGET_DIR=C:\Users\swsz9\.rhwp-shared-target`, 종료 코드 0.

```json
{
  "charCount": 37,
  "command": "markdown",
  "markdown": "명령 단추\n\n선택 상자\n\n계절 선택\n\n라디오 단추\n\n\n\n여기에 입력",
  "page": 0,
  "pageCount": 1,
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-markdown",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "markdown"
  ],
  "version": "0.8.4"
}
```

`form-01.hwp` 0쪽은 누름틀 안내 문구와 「여기에 입력」 이 마크다운으로
나온다. `charCount` 37 은 그 문자열의 유니코드 문자 수다.

사용법 오류 실측 (종료 코드 2, stdout 비어 있음):

```
$ rhwp-q-markdown samples/form-01.hwp
오류: --page 가 필요합니다.
사용법: rhwp-q-markdown <파일> --page <N> [--json]
```

```
$ rhwp-q-markdown --page 0
오류: 파일 경로가 필요합니다.
사용법: rhwp-q-markdown <파일> --page <N> [--json]
```

```
$ rhwp-q-markdown --page 0 --fill-fields samples/form-01.hwp
오류: 알 수 없는 옵션입니다 - --fill-fields
사용법: rhwp-q-markdown <파일> --page <N> [--json]
```

쪽 범위 초과는 실행 오류 1 이다. `--page 99` 는
`오류: 쪽 마크다운을 읽지 못했습니다 - 페이지 99을(를) 찾을 수 없습니다`.

## 6. 시험

```
cargo test --bin rhwp-q-markdown
```

결과: `11 passed; 0 failed` (0.04s).

- `form01_page0_markdown_is_success` — 봉투 필드와 마크다운 문자열 성공
- `--page` 위치·`--json` 파싱
- `--page` / 파일 누락 → 2
- `--fill-fields` 같은 편집 플래그 → 2
- 음수 쪽 번호 → 2
- `--page` 값 없음·파일 두 개 → 2
- 99쪽 → 1
- 소스에 편집 API 호출이 없다

`rust-unit-test-tiers --check` 는 신규 source-side test 총량 증가로
거부한다. 시험은 요청된 위치(`src/bin/rhwp-q-markdown.rs`)에만 둔다.

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
| `src/bin/rhwp-q-markdown.rs` | CLI + 같은 파일 단위 시험 |
| `mydocs/working/agent_q_markdown.md` | 이 기록 |

만지지 않은 것:

- `Cargo.toml` · `Cargo.lock`
- `src/main.rs` · capabilities · 출처 지도
- `src/bin/rhwp-agent/**`
- `gym/` · `crates/`
- DocumentCore 편집 구현
- `extract_page_markdown_native` 본체
