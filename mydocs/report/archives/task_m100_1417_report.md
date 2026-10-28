# 완료 보고서 — Task M100-1417

- 이슈: https://github.com/edwardkim/rhwp/issues/1417
- 제목: Typeset pagination cursor와 render placement 높이 모델 불일치로 TAC 이미지 묶음이 다음 페이지로 밀림
- 브랜치: `local/task_m100_1417`
- 작성일: 2026-06-16

## 1. 결과 요약

`samples/hwpx/pagenation-001.hwpx`에서 2페이지에 함께 출력되어야 할 `pi=27` TAC 이미지 묶음이
3페이지로 밀리는 문제를 수정했다.

원인은 `pi=16`처럼 텍스트가 공백-only인 단일 라인 TAC table host 문단을 post-table 본문 텍스트가 있는
문단으로 취급한 것이다. 이 때문에 `Table pi=16` 뒤에 `PartialParagraph pi=16`이 추가되어 table
높이와 host line advance가 중복 누적되었고, pagination cursor가 실제 render placement보다 약
한 줄 높이만큼 아래로 밀렸다.

수정 후 문서는 2페이지로 paginate되며, `dump-pages` 기준 2페이지에 `Shape pi=27`이 포함된다.
작업지시자 시각 확인도 완료했다.

## 2. 변경 사항

| 파일 | 내용 |
|---|---|
| `src/renderer/typeset.rs` | TAC table post-text 판정에 공백-only 문단을 제외하는 좁은 helper 추가 |
| `tests/issue_1417_pagination_cursor_render.rs` | `pi=27` 이미지 묶음이 2페이지에 남고 `PartialParagraph pi=16` 중복 emission이 사라지는 회귀 테스트 추가 |
| `samples/hwpx/pagenation-001.hwpx` | #1417 재현 fixture 추가 |
| `mydocs/plans/task_m100_1417.md` | 수행 계획서 |
| `mydocs/plans/task_m100_1417_impl.md` | 구현 계획서 |
| `mydocs/orders/20260616.md` | 진행 기록 |

## 3. 원인과 수정

기존 `place_table_with_text`는 post-table 텍스트 존재 여부를 `!para.text.is_empty()`로 판정했다.
`pi=16`은 텍스트가 `" "`인 공백-only 단일 라인 문단이므로 실제 렌더할 본문 텍스트가 없음에도
post `PartialParagraph`가 추가되었다.

이번 수정은 전역 `para_has_visible_text` 의미를 바꾸지 않고, TAC table post-text emission 경로에만
단일 라인 공백-only TAC host 중복 emission 제외 조건을 적용했다. 공백/탭 등으로 구성된 별도
post line이 있는 문단은 기존 동작을 유지하도록 조건을 좁혔다.

적용 지점:

- `should_add_post_text`
- `has_post_text`

이로써 공백-only TAC table host는 table line 높이를 한 번만 소비하고, 필요한 trailing spacing
복원은 기존 정책대로 유지한다.

## 4. 검증

실행한 검증:

```bash
cargo fmt --check
cargo test --test issue_1417_pagination_cursor_render -- --nocapture
cargo test --test issue_1070_tac_table_post_text_overflow -- --nocapture
cargo test --test issue_1152_intra_para_vpos_reset -- --nocapture
cargo test --test issue_986 -- --nocapture
cargo test --test issue_1071_tac_cursor_nav -- --nocapture
cargo test --test issue_1285_tac_sequence_right_align -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate -- --nocapture
cargo build --bin rhwp
docker compose --env-file .env.docker run --rm wasm
git diff --check
```

결과: 모두 통과.

CLI 확인:

```bash
target/debug/rhwp dump-pages samples/hwpx/pagenation-001.hwpx -p 1
target/debug/rhwp export-svg samples/hwpx/pagenation-001.hwpx -o output/poc/task1417-final --debug-overlay
target/debug/rhwp export-render-tree samples/hwpx/pagenation-001.hwpx -o output/poc/task1417-render-tree
```

확인 결과:

- 문서 로드 결과가 3페이지에서 2페이지로 변경되었다.
- 2페이지 `dump-pages`에 `Shape          pi=27`이 포함된다.
- 2페이지 `dump-pages`에서 `PartialParagraph  pi=16`이 사라졌다.
- `output/poc/task1417-final/pagenation-001_002.svg`에서 `s0:pi=27` 라벨을 확인했다.
- WASM 산출물 생성 완료: `pkg/rhwp_bg.wasm` 약 5.5MB.

## 5. 시각 판정

시각 판정 대상:

- `output/poc/task1417-final/pagenation-001_002.svg`

작업지시자 시각 확인 완료.

## 6. 남은 사항

소스 수정과 로컬 검증, 시각 확인, `devel` 반영, 이슈 close를 완료했다.

후속 기록:

- `local/devel` → `devel` merge + push: `b39d680e`
- Issue #1417 close
- close 코멘트 정정 기록: https://github.com/edwardkim/rhwp/issues/1417#issuecomment-4715086216
- WASM 빌드 완료: `docker compose --env-file .env.docker run --rm wasm`
