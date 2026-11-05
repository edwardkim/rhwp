# #8425 기본 개요 번호의 머리 모양 — 시각 검증 근거

- 측정일: 2026-11-01 (KST)
- 측정 소스: 수정 커밋 `9b3f4d3c2` (base `upstream/devel` `2adaa0859`)
- 회귀 테스트: `tests/cases/issue_8425_default_outline_head_distance.rs`

## 입력과 독립 기준

| 구분 | 파일 | SHA-256 | 쪽수 |
|---|---|---|---|
| 원본 (기존 저장소 표본) | `samples/task3307/issue3307_outline_number.hwpx` | `28a60bd05e152fd5292bdc8b75c6dbd980a04375c0d8890b752e32017fd1731d` | — |
| 독립 기준 (기존, 한컴 2020 Print) | `pdf/task3307/issue3307_outline_number-2020.pdf` | `0b176d3020e89d6a896b4efcab87bb9683298ea10b8593ad799bf99009668acb` | 9 |

원본은 번호 정의(`<hh:numbering>`) 없이 개요 수준만 지정한 문단을 갖는다. rhwp는 이 경우
`default_outline_numbering()`으로 기본 개요 번호를 합성한다(#3307).

### 수준별 탐침 (진단 보조)

수준 1~7 개요 제목과 본문을 번갈아 둔 `probe/levels.json`을 `rhwp scaffold`로 HWPX(`probe/levels.hwpx`,
번호 정의 없음·저장 줄 없음)로 만들고, 한컴 두 버전에서 Print 경로로 출력했다
(원격 변환 MCP, `pdf_print_method: 0`).

| PDF | 한컴 | SHA-256 |
|---|---|---|
| `pdf/issue-8425/levels-2020.pdf` | 2020 `11.0.0.9136` | `9ea35451788e550d56609cf9a5168a732fd6e51034660befc3b54fc5366bfc76` |
| `pdf/issue-8425/levels-2024.pdf` | 2024 `13.0.0.3901` | `588fbe6acc38479982909aff1dd7da1e49fe8633fc91f7db4e9b626d83d38399` |

2026-11-05 메인터너 재확인에서 기존의 “번호 글자 폭” 설명을 정정했다. 두 PDF의
실제 문자 원점은 `본문 시작 = 번호 시작 + 내장 ^N의 참조 형식 폭 + 글자 크기의 50%`이며,
참조 형식은 수준당 1em을 예약한다. 실제 숫자·마침표의 글리프 전진폭으로 예약 폭을
대신하지 않는다. 두 줄로 넘친 제목은 첫 줄 본문 시작에서 이어진다. 두 엔진의 결과는 같다.
[원본 관측·추가 보정·최신 증거](../../archives/pr_8433_review.md#기본-개요-예약-폭의-추가-보정--양쪽-실측-완료-필수-회귀-대기)를
따르며, 아래 원 작성 당시 측정은 당시 제품의 실행 기록으로 보존한다.

## 측정

`scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96` (pdftoppm), raw = `silhouette_raw_match_percent`.
rhwp 글꼴은 `RHWP_FONT_PATH=ttfs/opensource`(문서 글꼴 돋움·한양신명조 등 미설치, 대체).
Native는 `--features native-skia` release 바이너리, fresh WASM은 같은 소스에서
`scripts/wasm-pack-locked.sh --target web --out-dir pkg`로 새로 만든 package다
(`pkg/rhwp_bg.wasm` SHA-256 `cba1c210d5b62ca6881f7fbde2b6b38ecedbeb674cc525eb173d45b4b2f03445`).

| 쪽 | 수정 전 Native | 수정 후 Native | 수정 후 fresh WASM |
|---|---|---|---|
| 1 | 100.00% | 100.00% | 100.00% |
| 2 | 99.43% | 99.43% | 99.42% |
| 3 | 100.00% | 100.00% | 100.00% |
| 4 | 100.00% | 100.00% | 100.00% |
| 5 | 100.00% | 100.00% | 100.00% |
| 6 | 100.00% | 100.00% | 100.00% |
| **7** | **97.42%** | **99.80%** | **99.80%** |
| 8 | 99.95% | 99.95% | 99.95% |
| 9 | 99.94% | 99.94% | 99.94% |

- 수정 후 전쪽 최저: Native 99.43%(2쪽), fresh WASM 99.42%(2쪽). 90% 미만 0쪽.
- 전체 쪽수: 한컴 9쪽 = rhwp 9쪽.
- 7쪽 전체 모드(`--file-target ... --pages 7`) `pr_review_gate`: Native·fresh WASM 모두 `passed`.
- 변경이 없는 쪽(1~6, 8, 9)은 수정 전후 값이 같다.
- TSV 원본: `tsv/native/`, `tsv/wasm/`, `tsv/base_native/`.

## 직접 판독 (7쪽, 96dpi)

| 위치 | 한컴 2020 PDF | 수정 전 rhwp | 수정 후 rhwp |
|---|---|---|---|
| "1." 시작 | 117.54px | 117.5px | 117.5px |
| "인적사항" 시작 | 137.53px | 130.0px (번호에 붙음) | 136.6px |
| "4." 둘째·셋째 줄 시작 | 137.53px | 117.5px (번호 자리) | 136.3px |
| 리터럴 "5. 혐의자의" | 거리 없음 | 거리 없음 | 거리 없음 |

| 파일 | 내용 |
|---|---|
| `png/outline3307-p007-base-native-render.png` | 수정 전 rhwp 7쪽: "1.인적사항"처럼 번호와 본문이 붙고, "4." 제목의 넘친 줄이 번호 자리에서 시작한다 |
| `png/outline3307-p007-{native,wasm}-{review,overlay}.png` | 수정 후: 번호 뒤 간격과 넘친 줄의 시작 열이 한컴과 겹친다 |

남은 1px 안팎 차이는 번호 글자("1.")의 대체 글꼴 폭 차이(rhwp 12.5px, 한컴 원 글꼴)에서 온다.
머리 모양 거리 규칙과는 무관하다.

## 회귀 테스트 수정 전·후

수정 전 `src/renderer/layout/utils.rs`(base `2adaa0859`)로 같은 테스트를 실행한 결과:

| 테스트 | 수정 전 | 수정 후 |
|---|---|---|
| `default_outline_number_keeps_half_em_from_text` | FAIL: `1. "인적사항": 번호 뒤 거리 0.00px 가 글자 크기 13.33px 의 50%(6.67px)여야 한다` | PASS |
| `default_outline_wrapped_lines_hang_at_text_start` | FAIL: `"감경 대상 비위 해당 여부" 줄의 시작 117.47px 가 첫 줄 본문 시작 129.68px 와 같아야 한다 (번호 시작 117.47px)` | PASS |
| `literal_numbered_text_is_not_indented` (반례) | PASS | PASS |

기존 `tests/issue_3307_outline_default_numbering.rs`(글자 내용·쪽수)는 수정 후에도 통과하며
기대값을 바꾸지 않았다.

## 로컬 검증

최소 글꼴 환경: macOS, 사용자 글꼴 0개, `/Library/Fonts`는 OS 동봉 `Arial Unicode.ttf`뿐,
`RHWP_FONT_PATH` 해제.

| 검증 | 명령 | 결과 |
|---|---|---|
| 집중 회귀 | `cargo nextest run --locked --cargo-profile release-test --tests -E 'test(/issue_8425\|issue_3307\|outline\|numbering/)'` | 121 passed |
| 전체 회귀 | `cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast` | 12248 passed, 45 skipped, 0 failed (테스트 최종본 기준 재실행) |
| 포맷 | `cargo fmt --all -- --check` | 통과 |
| Clippy native | `cargo clippy --locked -- -D warnings` | 통과 |
| Clippy WASM32 | `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings` | 통과 |
| workspace build | `cargo build --locked --workspace` | 통과 |
| Clippy workspace | `cargo clippy --locked --workspace --all-targets -- -D warnings` | 통과 |
| suite manifest | `node scripts/rust-test-suite-manifest.mjs --check --base-ref 2adaa0859` | 통과 |

`src/**`의 `#[cfg(test)]`는 바꾸지 않아 `rust-unit-test-tiers` 검사는 비해당이다. `samples/`에 파일을
추가하지 않아 코퍼스 래칫도 비해당이다.
