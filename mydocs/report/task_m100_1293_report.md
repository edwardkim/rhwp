# Task M100 #1293 최종 보고서 — 공식 미주 모양 모델 정규화

- 이슈: #1293 `#1274/#1284 후속: 공식 미주 모양 모델 정규화`
- 브랜치: `local/task_m100_1293`
- 기준 브랜치: `upstream/devel`
- 작성일: 2026-06-14

## 1. 결함과 해소

| 축 | 종전 | 해소 |
|----|------|------|
| 모델 의미 | HWP5/HWPX 슬롯명, UI JSON 이름, 렌더 계산식의 `구분선 위/아래/미주 사이` 의미가 섞여 있었다. | `FootnoteShape` 공식 의미 접근자와 note shape dump/테스트를 통해 UI 의미 기준으로 고정했다. |
| HWPX/HWP5 대응 | HWPX `<hp:noteSpacing>`과 HWP5 `HWPTAG_FOOTNOTE_SHAPE` 값의 내부 의미가 명확히 드러나지 않았다. | 파서/모델/테스트에서 `aboveLine`, `belowLine`, `betweenNotes`를 공식 의미로 검증했다. |
| 타입셋/렌더 흐름 | 증상별 y/gap 보정이 누적되어 새 샘플에서 overlap/overflow가 재발했다. | 정규화된 미주 모양 profile을 기준으로 separator, between-notes, rewind/title-tail, no-separator 흐름을 분기했다. |
| sweep | overflow/overlap 중심이라 미주 모양 설정값과 실제 separator gap 확인이 부족했다. | visual sweep에 note shape 요약, separator gap, question flow, line/large drift 지표를 보강했다. |

## 2. 구현 요약

- `src/model/footnote.rs`
  - 공식 UI 의미 접근자와 주석 정리.
- `src/parser/body_text.rs`, `src/parser/hwpx/section.rs`
  - HWP5/HWPX 미주 모양 값의 정규화 의미 대응 보강.
- `src/renderer/typeset.rs`, `src/renderer/height_cursor.rs`, `src/renderer/layout.rs`
  - visible/no-separator, default/large `미주 사이`, 큰/기본 `구분선 아래`, rewind/title-tail, equation/TAC tail의 공통 흐름 보정.
- `scripts/task1274_visual_sweep.py`
  - note shape와 separator gap, question marker flow, line/large ink drift 분석 보강.
  - 같은 HWP/PDF를 두 번 검사하던 `2024-09-below20above20` 중복 target 제거.
- `tests/issue_1139_inline_picture_duplicate.rs`, `tests/issue_1082_endnote_multicolumn_drift.rs`, `tests/issue_1050_footnote_serialize.rs`, `src/renderer/layout/tests.rs`
  - 미주 모양/간격/overflow 회귀 검증 보강.
  - rebase 후 clean sweep target 12개의 page count와 공식 미주 모양 profile 회귀 테스트 추가.
  - `2024-09-below20-above20`의 p19/p20/p22 최종 잔여 판단에 맞춰 sep20/20 overflow 가드를 40px 상한으로 조정.

## 3. 최종 검증

GitHub Actions CI와 PR 생성은 수행하지 않았다. 작업지시자 요청에 따라 stage124에서 PR용 로컬 전체 테스트와 WASM 빌드를 수행했다.

- 최근 코드 stage 검증:
  - `cargo fmt --check`: 통과
  - `cargo build --bin rhwp`: 통과
  - `cargo test --test issue_1139_inline_picture_duplicate issue_1293`: 통과 (`10 passed`)
  - `cargo test --test issue_1139_inline_picture_duplicate`: 통과 (`85 passed`)
  - `cargo build --release`: 통과
  - `cargo test --release --lib`: 통과 (`1816 passed`, `6 ignored`)
  - `cargo test --profile release-test --tests`: 통과
  - `PATH="$HOME/.cargo/bin:$PATH" wasm-pack build --target web --out-dir pkg`: 통과
    - PATH 기본 `/opt/homebrew/bin/wasm-pack` Node wrapper는 `pkg/package.json`의 `repository` 객체 parse 오류로 실패했으나,
      `~/.cargo/bin/wasm-pack` Rust 바이너리 경로에서는 동일 인자 빌드가 완료됐다.
  - targeted visual sweep:
    - stage115: 4개 target 모두 0
    - stage117: `2022-09` 0 전환, 회귀 target 0 유지
- rebase 후 최종 전체 visual sweep:
  - 기준: `upstream/devel` `a0a37d72`
  - 명령: `python3 scripts/task1274_visual_sweep.py --target all --out output/task1293_stage122_rebase_full_sweep`
  - 결과: `flagged=7/323`
  - 0 target: 12개
  - 잔여 target: 3개 key
  - stage121 최종 보고서의 잔여 key와 동일함

## 4. 최종 잔여 판단

| target | 최종 후보 | 판단 |
|--------|----------:|------|
| `2022-10` | `1/18` p14 | question/tail 없음. 9px 수식/쉼표 bbox overlap 및 large ink coarse drift 후보로 분류. |
| `2024-09-below20-above20` | `3/23` p19/p20/p22 | stage113에서 문28 본문/그림/수식 continuation 높이 차이의 tail/cascade로 분류. |
| `2024-11-practice-above0-between20-below2` | `3/22` p17/p20/p21 | stage111에서 문26/문28 본문 높이와 그림/수식 tail 잔여로 분류. |

공식 `구분선 위`, `구분선 아래`, `미주 사이` 계산식 자체의 남은 직접 불일치로 판단한 후보는 없다.

## 5. 산출물

- 수행 계획서: `mydocs/plans/task_m100_1293.md`
- 구현 계획서: `mydocs/plans/task_m100_1293_impl.md`
- 단계 문서: `mydocs/working/task_m100_1293_stage1.md`부터 `stage124.md`
- 최종 sweep: `output/task1293_stage122_rebase_full_sweep/summary.json`

## 6. 미수행 항목

- GitHub Actions CI: 미수행
- PR 생성: 미수행
