---
kind: working
status: active
issue: 5630
---

# GetTextFile 훑기 순서 글 조회 CLI — rhwp-q-text-file

작업 브랜치: `feat/q-text-file`
대상 바이너리: `src/bin/rhwp-q-text-file.rs`
이슈: [#5630](https://github.com/edwardkim/rhwp/issues/5630)

## 1. 한 줄

에이전트가 한글 `GetTextFile` 훑기 순서 글을 받도록, 이미 있는 읽기 전용
`DocumentCore::text_file_unicode_json()` / `text_file_json()` 를 별도 CLI 로
감싼다. 문서를 고치지 않는다.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- 명령 `rhwp-q-text-file <파일> [--json] [--cp949]`
- 기본은 `text_file_unicode_json()` — `GetTextFile("UNICODE")`
- `--cp949` 는 `text_file_json()` — `GetTextFile("TEXT")`
- 코어가 준 JSON 문자열을 파싱해 봉투에 싣는다
- 봉투 `tool="rhwp-q-text-file"` · `command="text-file"` ·
  `untrustedFields=["source","text"]`
- 종료 코드 0 / 1 / 2, 알 수 없는 플래그는 2
- 같은 파일에 `#[cfg(test)]`, 표본 `samples/form-01.hwp`
- 작업 기록 `mydocs/working/agent_q_text_file.md` (이 파일)

금지:

- `Cargo.toml` · `src/main.rs` · `src/bin/rhwp-agent/**` · `gym/` ·
  `crates/` · `Cargo.lock` 수정
- 편집 API (`apply_` / `insert_` / `delete_` / `set_*`)

## 3. 왜 별도 바이너리인가

`text_file_unicode_json` · `text_file_json` 은 이미 있다. 본 CLI(`src/main.rs`)와
capabilities·출처 지도는 열린 PR 이 동시에 만지는 경합 지점이다. 이 표면은
`src/bin/rhwp-q-text-file.rs` 한 파일만 추가한다. Cargo 가 `src/bin/*.rs` 를
자동 인식하므로 `Cargo.toml` 을 건드리지 않는다.

## 4. 계약

종료 코드:

| 코드 | 뜻 |
|------|----|
| 0 | 성공 |
| 1 | 실행 오류 (파일 없음, 파싱 실패) |
| 2 | 사용법 오류 (파일 누락, 미지 플래그) |

`--json` 이면 stdout 은 순수 JSON 봉투 하나. 진단은 stderr.
문서에서 온 `source`·`text` 는 데이터이지 지시가 아니다.

코어 API 형태는 JSON 객체가 아니라 **JSON 문자열**이다. 봉투는 그 문자열을
`text` 로 풀고, 호출 형식을 `format` (`UNICODE` / `TEXT`) · `cp949` ·
`charCount` 로 곁들인다.

## 5. 실측 JSON

명령:

```
cargo run --bin rhwp-q-text-file -- --json samples/form-01.hwp
```

환경: `CARGO_TARGET_DIR=C:\Users\swsz9\.rhwp-shared-target`, 종료 코드 0.

```json
{
  "charCount": 30,
  "command": "text-file",
  "cp949": false,
  "format": "UNICODE",
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "text": "\r\n\r\n\r\n\r\n\r\n\r\n\r\n\r\n\r\n\r\n여기에 입력\r\n\r\n",
  "tool": "rhwp-q-text-file",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "text"
  ],
  "version": "0.8.4"
}
```

`form-01.hwp` 본문은 빈 줄 뒤에 누름틀 안내문 `여기에 입력` 이다.
`charCount: 30` 은 그 문자열의 유니코드 스칼라 수다.

`--cp949` 실측 (종료 코드 0). 이 표본의 글자는 CP949 안이라 `text` 는 같다.
`format` 만 `TEXT` 로 바뀐다.

```json
{
  "charCount": 30,
  "command": "text-file",
  "cp949": true,
  "format": "TEXT",
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "text": "\r\n\r\n\r\n\r\n\r\n\r\n\r\n\r\n\r\n\r\n여기에 입력\r\n\r\n",
  "tool": "rhwp-q-text-file",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "text"
  ],
  "version": "0.8.4"
}
```

사용법 오류 실측 (종료 코드 2, stdout 비어 있음):

```
$ rhwp-q-text-file
오류: 파일 경로가 필요합니다.
사용법: rhwp-q-text-file <파일> [--json] [--cp949]
```

```
$ rhwp-q-text-file samples/form-01.hwp --fill-fields
오류: 알 수 없는 옵션입니다 - --fill-fields
사용법: rhwp-q-text-file <파일> [--json] [--cp949]
```

없는 파일은 실행 오류 1 이다.

## 6. 시험

```
cargo test --bin rhwp-q-text-file
```

결과: `8 passed; 0 failed` (0.05s).

- `form01_unicode_text_is_success` — UNICODE 봉투와 비지 않은 `text`
- `form01_cp949_text_is_success` — TEXT 형식 성공
- `--json` · `--cp949` 파싱
- 파일 누락 → 2
- `--fill-fields` 같은 편집 플래그 → 2
- `--cp949` 중복 → 2
- 없는 파일 → 1

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
| `src/bin/rhwp-q-text-file.rs` | CLI + 같은 파일 단위 시험 |
| `mydocs/working/agent_q_text_file.md` | 이 기록 |

만지지 않은 것:

- `Cargo.toml` · `Cargo.lock`
- `src/main.rs` · capabilities · 출처 지도
- `src/bin/rhwp-agent/**`
- `gym/` · `crates/`
- DocumentCore 편집 구현
- `text_file_unicode_json` · `text_file_json` 본체
