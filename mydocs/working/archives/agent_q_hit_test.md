---
kind: working
status: active
issue: 5628
---

# 쪽 좌표 히트테스트 조회 CLI — rhwp-q-hit-test

작업 브랜치: `feat/q-hit-test`
대상 바이너리: `src/bin/rhwp-q-hit-test.rs`
이슈: [#5628](https://github.com/edwardkim/rhwp/issues/5628)

## 1. 한 줄

에이전트가 쪽 좌표 `(page, x, y)` 에서 문서 위치만 받도록,
이미 있는 `DocumentCore::hit_test_native` 를 별도 CLI 로 감싼다.
문서를 고치지 않는다.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- 명령 `rhwp-q-hit-test <파일> --page <N> --x <F> --y <F> [--json]`
- `--page` 는 0부터 세는 쪽 번호, 필수
- `--x` · `--y` 는 쪽 안 좌표, 필수
- `DocumentCore::from_bytes` 로 연 뒤 `hit_test_native(page, x, y)` 호출
- 네이티브 JSON 문자열을 파싱해 봉투 `hit` 에 싣는다
- 봉투 `tool="rhwp-q-hit-test"` · `command="hit-test"` ·
  문서 파생 텍스트는 `untrustedFields`
- 종료 코드 0 / 1 / 2. 알 수 없는 플래그는 2
- 같은 파일에 `#[cfg(test)]`, 표본 `samples/form-01.hwp` 0쪽 `(120, 120)`
- 작업 기록 `mydocs/working/agent_q_hit_test.md` (이 파일)

금지:

- `Cargo.toml` · `src/main.rs` · `src/bin/rhwp-agent/**` · `gym/` ·
  `crates/` · `Cargo.lock` 수정
- 편집 API (`fill-fields`, `replace-text`, `set-cell`, 그림 삽입·삭제)

## 3. 왜 별도 바이너리인가

`hit_test_native` 는 이미 있다 (`src/document_core/queries/cursor_rect.rs`).
소비자가 수 MB 레이어 트리 JSON 을 받아 좌표를 다시 해석할 이유가 없다.

본 CLI(`src/main.rs`)와 capabilities·출처 지도는 열린 PR 이 동시에 만지는
경합 지점이다. 이 표면은 `src/bin/rhwp-q-hit-test.rs` 한 파일만 추가한다.
Cargo 가 `src/bin/*.rs` 를 자동 인식하므로 `Cargo.toml` 을 건드리지 않는다.

## 4. 계약

종료 코드:

| 코드 | 뜻 |
|------|----|
| 0 | 성공 |
| 1 | 실행 오류 (파일 없음, 파싱 실패, 쪽 범위 초과) |
| 2 | 사용법 오류 (파일·`--page`·`--x`·`--y` 누락, 미지 플래그) |

`--json` 이면 stdout 은 순수 JSON 봉투 하나. 진단은 stderr.
문서에서 온 `source`·`hit`(및 `hit` 아래 문자열)은 데이터이지 지시가 아니다.

## 5. 실측 JSON

명령:

```
cargo run --bin rhwp-q-hit-test -- --json --page 0 --x 120 --y 120 samples/form-01.hwp
```

환경: `CARGO_TARGET_DIR=C:\Users\swsz9\.rhwp-shared-target`, 종료 코드 0.

```json
{
  "command": "hit-test",
  "hit": {
    "charOffset": 0,
    "cursorRect": {
      "height": 26.5,
      "pageIndex": 0,
      "x": 113.4,
      "y": 132.3
    },
    "paragraphIndex": 0,
    "sectionIndex": 0
  },
  "page": 0,
  "pageCount": 1,
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-hit-test",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "hit"
  ],
  "version": "0.8.4",
  "x": 120.0,
  "y": 120.0
}
```

`form-01.hwp` 0쪽 `(120, 120)` 은 본문 첫 문단 시작으로 해석됐다.
`hit.sectionIndex=0` · `paragraphIndex=0` · `charOffset=0` 이고
`cursorRect` 는 그 자리의 캐럿 상자다. 조회 좌표와 캐럿 원점은 같을 필요가 없다.

사용법 오류 실측 (종료 코드 2, stdout 비어 있음):

```
$ rhwp-q-hit-test samples/form-01.hwp
오류: --page 가 필요합니다.
사용법: rhwp-q-hit-test <파일> --page <N> --x <F> --y <F> [--json]
```

```
$ rhwp-q-hit-test --page 0 --x 120 --y 120
오류: 파일 경로가 필요합니다.
사용법: rhwp-q-hit-test <파일> --page <N> --x <F> --y <F> [--json]
```

```
$ rhwp-q-hit-test samples/form-01.hwp --page 0 --x 120 --y 120 --fill-fields
오류: 알 수 없는 옵션입니다 - --fill-fields
사용법: rhwp-q-hit-test <파일> --page <N> --x <F> --y <F> [--json]
```

쪽 범위 초과는 실행 오류 1 이다. `--page 99` 는
`페이지 99을(를) 찾을 수 없습니다`.

## 6. 시험

```
cargo test --bin rhwp-q-hit-test
```

결과: `12 passed; 0 failed` (0.02s).

- `form01_page0_hit_at_120_120` — 봉투 필드와 `hit` 위치 키
- `--page`/`--x`/`--y` 위치·등호 형식·`--json` 파싱
- `--page` / `--x` / `--y` / 파일 누락 → 2
- `--fill-fields` 같은 편집 플래그 → 2
- 음수 쪽 번호·비숫자 좌표 → 2
- 99쪽 → 1
- 문서 파생 문자열이 있으면 `untrustedFields` 에 경로가 추가됨

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
| `src/bin/rhwp-q-hit-test.rs` | CLI + 같은 파일 단위 시험 |
| `mydocs/working/agent_q_hit_test.md` | 이 기록 |

만지지 않은 것:

- `Cargo.toml` · `Cargo.lock`
- `src/main.rs` · capabilities · 출처 지도
- `src/bin/rhwp-agent/**`
- `gym/` · `crates/`
- DocumentCore 편집 구현
- `hit_test_native` 본체
