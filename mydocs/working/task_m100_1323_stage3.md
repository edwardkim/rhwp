# Stage 3 보고서 — Task M100-1323

- 이슈: #1323
- 작성일: 2026-06-11
- 브랜치: `local/task1323`

## 1. 작업 요약

전체 회귀 검증(cargo test/clippy)과 WASM 빌드를 수행하고, 작업지시자 시각 판정을
보조하는 SVG 렌더링 자동 검증 테스트를 추가했다. 셀/글상자에 붙여넣은 그림이
렌더 트리를 거쳐 실제 SVG `<image>` 요소로 방출되는 것까지 자동으로 고정하여,
Stage 2의 모델 수준 검증(컨트롤 보존)과 수동 시각 판정 사이의 간극을 줄였다.

## 2. 변경 파일

| 파일 | 내용 |
|------|------|
| `src/wasm_api/tests.rs` | `test_paste_picture_into_cell_and_textbox_renders_in_svg` 추가 (+106줄) |

## 3. SVG 렌더링 자동 검증 테스트

`test_paste_picture_into_cell_and_textbox_renders_in_svg` (`src/wasm_api/tests.rs:2335`):

1. `insert_picture_native`로 실제 PNG(BinData 등록)를 본문에 삽입 — 기본 속성이
   **`treat_as_char: false`(부동, Para anchor)** 이므로 수행계획서 리스크 항목
   "떠 있는 개체 셀 anchor 렌더링"을 그대로 재현한다.
2. 기준 SVG 렌더에서 `<image>` 수 확인 (본문 그림 ≥ 1).
3. `copy_control_native` → 표 셀 `paste_internal_in_cell_native` → SVG 재렌더 →
   `<image>` +1 검증.
4. 글상자(`create_shape_control_native` textbox) → 동일 paste → SVG 재렌더 →
   `<image>` +1 추가 검증.

데이터 URI 이미지 방출 경로(BinData → base64)를 실 렌더러 그대로 사용하므로,
컨트롤이 모델에 남아 있어도 렌더에서 누락되는 회귀를 잡을 수 있다.

## 4. 검증

통과:

- `cargo test --lib test_paste_picture_into_cell_and_textbox_renders_in_svg` — 1 passed
- `cargo test` 전체 — **2139 passed, 0 failed** (Stage 2 대비 +1 = 신규 SVG 테스트)
- `cargo clippy --all-targets` — 무경고
- `rustfmt --check` (변경 파일 한정) — 통과
- `docker compose --env-file .env.docker run --rm wasm` — WASM 빌드 통과 (`pkg/rhwp_bg.wasm` 생성)

## 5. 작업지시자 시각 판정 대기 항목

WASM 빌드 산출물(`pkg/`)로 rhwp-studio에서 다음을 확인 부탁드립니다:

1. 본문 이미지 복사 → 글상자 안 붙여넣기 → 이미지 렌더 확인
2. 본문 이미지 복사 → 표 셀 안 붙여넣기 → 이미지 렌더 확인
3. 텍스트 붙여넣기(본문/셀/글상자), 본문 이미지 붙여넣기(pasteControl) 무회귀
4. 셀 안 백스페이스 문단 병합 시 그림 보존 확인
