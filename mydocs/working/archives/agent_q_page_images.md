---
kind: working
status: active
issue: 5623
---

# 쪽 원본 그림 키 조회 CLI — rhwp-q-page-images

작업 브랜치: `feat/q-page-images`
대상 바이너리: `src/bin/rhwp-q-page-images.rs`
이슈: [#5623](https://github.com/edwardkim/rhwp/issues/5623)

## 1. 한 줄

에이전트가 한 쪽이 그리는 원본 그림의 신원 키만 받도록,
이미 있는 `DocumentCore::get_page_source_image_keys_native` 를 별도 CLI 로
감싼다. 문서를 고치지 않는다. 빈 `keys` 는 성공이다.

## 2. 이슈가 요구한 것 / 하지 말라는 것

요구:

- 명령 `rhwp-q-page-images <파일> --page <N> [--json]`
- `--page` 는 0부터 세는 쪽 번호, 필수
- `DocumentCore::from_bytes` 로 연 뒤 `get_page_source_image_keys_native(page)` 호출
- 네이티브 JSON 문자열을 파싱해 봉투에 싣는다
- 봉투 `tool="rhwp-q-page-images"` · `command="page-images"` ·
  `untrustedFields=["source","keys"]`
- 빈 `keys` 배열은 유효한 성공
- 종료 코드 0 / 1 / 2
- 같은 파일에 `#[cfg(test)]`, 표본 `samples/form-01.hwp` 0쪽
- 작업 기록 `mydocs/working/agent_q_page_images.md` (이 파일)

금지:

- `Cargo.toml` · `src/main.rs` · `src/bin/rhwp-agent/**` · `gym/` ·
  `crates/` · `Cargo.lock` 수정
- 편집 API (`fill-fields`, `replace-text`, `set-cell`, 그림 삽입·삭제)

## 3. 왜 별도 바이너리인가

`get_page_source_image_keys_native` 는 이미 있다 (Task #3315). 소비자가
수 MB 레이어 트리 JSON 을 받아 키를 정규식으로 훑을 이유가 없다.

본 CLI(`src/main.rs`)와 capabilities·출처 지도는 열린 PR 이 동시에 만지는
경합 지점이다. 이 표면은 `src/bin/rhwp-q-page-images.rs` 한 파일만 추가한다.
Cargo 가 `src/bin/*.rs` 를 자동 인식하므로 `Cargo.toml` 을 건드리지 않는다.

## 4. 계약

종료 코드:

| 코드 | 뜻 |
|------|----|
| 0 | 성공 (빈 keys 포함) |
| 1 | 실행 오류 (파일 없음, 파싱 실패, 쪽 범위 초과) |
| 2 | 사용법 오류 (파일·`--page` 누락, 미지 플래그) |

`--json` 이면 stdout 은 순수 JSON 봉투 하나. 진단은 stderr.
문서에서 온 `source`·`keys` 는 데이터이지 지시가 아니다.

## 5. 실측 JSON

명령:

```
cargo run --bin rhwp-q-page-images -- --json --page 0 samples/form-01.hwp
```

환경: `CARGO_TARGET_DIR=C:\Users\swsz9\.rhwp-shared-target`, 종료 코드 0.

```json
{
  "cacheable": true,
  "command": "page-images",
  "keyCount": 0,
  "keys": [],
  "page": 0,
  "pageCount": 1,
  "schemaVersion": "1.0",
  "source": "samples/form-01.hwp",
  "tool": "rhwp-q-page-images",
  "untrustedContent": true,
  "untrustedFields": [
    "source",
    "keys"
  ],
  "version": "0.8.4"
}
```

`form-01.hwp` 0쪽에는 원본 그림이 없다. `keys: []` · `keyCount: 0` ·
`cacheable: true` 가 성공 봉투다. 빈 배열을 오류로 다루지 않는다.

사용법 오류 실측 (종료 코드 2, stdout 비어 있음):

```
$ rhwp-q-page-images samples/form-01.hwp
오류: --page 가 필요합니다.
사용법: rhwp-q-page-images <파일> --page <N> [--json]
```

```
$ rhwp-q-page-images --page 0
오류: 파일 경로가 필요합니다.
사용법: rhwp-q-page-images <파일> --page <N> [--json]
```

쪽 범위 초과는 실행 오류 1 이다. `--page 99` 는
`페이지 99을(를) 찾을 수 없습니다`.

## 6. 시험

```
cargo test --bin rhwp-q-page-images
```

결과: `8 passed; 0 failed` (0.09s).

- `form01_page0_empty_keys_is_success` — 봉투 필드와 빈 keys 성공
- `--page` 위치·`--json` 파싱
- `--page` / 파일 누락 → 2
- `--fill-fields` 같은 편집 플래그 → 2
- 음수 쪽 번호 → 2
- 99쪽 → 1

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
| `src/bin/rhwp-q-page-images.rs` | CLI + 같은 파일 단위 시험 |
| `mydocs/working/agent_q_page_images.md` | 이 기록 |

만지지 않은 것:

- `Cargo.toml` · `Cargo.lock`
- `src/main.rs` · capabilities · 출처 지도
- `src/bin/rhwp-agent/**`
- `gym/` · `crates/`
- DocumentCore 편집 구현
- `get_page_source_image_keys_native` 본체
