# Task #1310 완료 보고서 - 수식-only TAC 흐름 자동 줄바꿈

## 1. 결론

최종 채택안:

```text
가설 A 확장안 + 수식-only TAC 한정 60.5pt 내어쓰기 적용
```

수식-only TAC 흐름에서 연속 TAC 수식의 폭을 열 폭 기준으로 packing하고,
자동 줄넘김으로 생성된 virtual row에는 문단의 후속 줄 들여쓰기/내어쓰기 규칙을 적용한다.

Stage 4의 행 폭 기준 가운데 정렬 가설은 폐기했다. 이후 수식-only TAC 흐름에서
첫 시각 줄/후속 시각 줄을 raw `line_idx` 가 아니라 실제 출력되는 visual row 기준으로
계산했다. 최종 후보에서는 후속 수식 줄에 한컴 UI 내어쓰기 `60.5pt` 전체가 적용되도록
수식-only TAC 경로에만 `indent_scale=2.0` 을 적용했다.

작업지시자 시각 판정 통과로 완료 처리한다.

## 2. 처리 내용

수정 파일:

- `src/renderer/equation_tac_flow.rs`
- `src/renderer/mod.rs`
- `src/renderer/layout/paragraph_layout.rs`
- `src/renderer/typeset.rs`
- `src/renderer/height_measurer.rs`
- `src/document_core/queries/cursor_nav.rs`
- `src/document_core/queries/doc_tree_nav.rs`
- `tests/issue_1139_inline_picture_duplicate.rs`

문서:

- `mydocs/plans/task_m100_1310.md`
- `mydocs/working/task_m100_1310_stage1.md`
- `mydocs/working/task_m100_1310_stage2.md`
- `mydocs/working/task_m100_1310_stage3.md`
- `mydocs/working/task_m100_1310_stage4.md`
- `mydocs/working/task_m100_1310_stage5.md`
- `mydocs/working/task_m100_1310_stage6.md`
- `mydocs/working/task_m100_1310_stage7.md`
- `mydocs/working/task_m100_1310_stage8.md`
- `mydocs/working/task_m100_1310_stage9.md`
- `mydocs/working/task_m100_1310_stage10.md`

주요 구현:

- TAC 수식-only 줄의 line assignment와 width packing을 공통 helper로 분리했다.
- 첫 row 가용폭과 후속 row 가용폭을 구분했다.
- 후속 row는 실제 출력 visual row 기준으로 문단 들여쓰기/내어쓰기 effective margin을 다시 계산한다.
- 수식-only TAC 경로에 한해 후속 줄 내어쓰기 폭은 한컴 UI 지정값 `60.5pt` 전체와 맞추기 위해
  `indent * 2.0` 으로 계산한다.
- compact endnote 수식-only 줄에만 걸었던 x 예외와 non-cell 행 가운데 정렬을 제거했다.
- 셀 내부 수식-only 줄의 cell alignment는 유지했다.
- 렌더링, typeset, height measurer가 같은 helper를 사용해 줄 수와 높이 계산을 맞춘다.
- 기존 #1308 혼합 TAC/텍스트/고정탭 경로는 `runs.is_empty()` 가드로 건드리지 않았다.
- 수식-only TAC 경로의 디버그 조판부호(`↓`, `↵`)는 empty-run 앵커가 아니라 마지막
  visual row의 수식 끝 x에 표시되도록 보정했다.
- WASM 동작 테스트 중 확인된 미주 가상 문단 세로 커서 이동 panic을 방어했다.
  `moveVertical` 본문 경계 처리에서 본문 문단 수만 보던 경로를 렌더 문단 수
  (본문 + 미주 가상 문단) 기준으로 정정했다.
- 미주 영역에서 오른쪽 방향키가 조용히 이동 실패하던 문제를 보정했다.
  빈 context 의 `navigateNextEditable` 도 본문 문단 배열이 아니라 렌더 문단 인덱스
  공간(본문 + 미주 가상 문단)을 기준으로 현재 문단과 문단 수를 해석한다.
- 9쪽 미주 수식 문단에서 오른쪽 방향키 이동 중 커서 좌표가 반복적으로 첫 위치로 돌아가던
  문제를 보정했다. 수식 `EquationNode` 의 렌더 문단 위치와 원본 미주 위치(`note_ref`)를
  분리해, `getCursorRect` 가 수식 bbox 를 화면상의 렌더 문단 기준으로 찾도록 했다.
- 수식-only 문단에서 이전 수식 끝과 다음 수식 시작이 같은 x를 공유할 때, 오른쪽 방향키가
  같은 caret stop을 한 번 더 밟는 문제를 보정했다. 텍스트/탭/강제 줄바꿈이 섞인 문단은
  기존 offset 이동을 유지하고, 순수 인접 수식 문단에서만 중복 경계를 건너뛴다.

## 3. 가설 판정

| 가설 | 판정 | 이유 |
|---|---|---|
| A. layout 단계 TAC-only packing | 채택 | 자동 줄넘김과 후속 row 문단 내어쓰기 적용이 확인됨 |
| B. composer/IR line model 재정의 | 보류 | #1308 커서/강제 줄넘김 테스트가 통과하며, 현재 범위는 수식-only empty-run으로 충분함 |
| C. TAC 폭/단위 계산 문제 | 보류 | 기존 TAC 폭으로도 한컴 기준 줄넘김과 column overflow 해소가 가능함 |
| D. text/TAC/fixed-tab 혼합 순서 문제 | 보류 | 혼합 문단은 기존 #1308 순서 보존 경로를 유지하고 테스트가 통과함 |

## 4. 검증

통과:

```bash
cargo check
cargo check --target wasm32-unknown-unknown --lib
cargo fmt --all -- --check
cargo test --test issue_1308_forced_break_hanging_indent -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate issue_1256_2022_sep_page10_question12_keeps_between_notes_gap -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate issue_1139_endnote_virtual_paragraph_vertical_move_does_not_panic -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate issue_1139_endnote_virtual_paragraph_right_arrow_moves_within_text -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate issue_1139_endnote_equation_cursor_rects_do_not_rewind_to_line_start -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate issue_1139_endnote_equation_right_arrow_skips_duplicate_boundary_stop -- --nocapture
cargo test --test issue_1139_inline_picture_duplicate endnote_virtual -- --nocapture
cargo build --verbose
cargo test --features native-skia skia --lib --verbose
cargo test --verbose
cargo clippy -- -D warnings
git diff --check
docker compose --env-file .env.docker run --rm wasm
```

주요 결과:

- #1308 강제 줄넘김/내어쓰기/커서 이동 테스트 8개 통과
- #1256/#1310 문12 수식-only 미주 흐름 회귀 테스트 통과
- 미주 가상 문단 `para=602` 에서 `moveVertical` 문단 경계를 밟아도 panic 없이 커서 JSON 반환
- 미주 가상 문단 `para=602` 에서 `navigateNextEditable(..., +1, [])` 가 `Boundary` 대신
  같은 렌더 문단의 다음 편집 위치를 반환
- 9쪽 미주 수식 문단 `pi=471`, `pi=474`, `pi=479` 에서 `getCursorRect` x 좌표가
  오른쪽 이동 중 줄 시작으로 되감기지 않음
- 9쪽 미주 수식-only 문단 `pi=479` 에서 오른쪽 방향키가 같은 x의 인접 수식 경계를
  반복해서 밟지 않음
- `eq-002` 강제 줄바꿈 다음 줄 진입은 offset 3 -> 4 를 유지해 첫 수식 앞 caret stop을 보존
- 수정본 WASM 빌드 성공
- GitHub Actions `ci.yml` 기준 주요 사전점검 통과:
  - format/check/build/test/clippy/WASM target/native-skia
  - `git diff --check` 공백 오류 없음
- 로컬 `cargo test --verbose` 실행 중 cargo global cache last-use DB 쓰기 경고가 1회 출력되었으나
  테스트 종료 코드는 0이며, CI 실패 요인은 아닌 것으로 판단
- `render-diff.yml` 은 PR 이벤트에서 renderer 경로 변경 시 별도 동작한다. devel push 기준으로는
  `ci.yml`/CodeQL 경로가 주요 대상이며, 이번 사전점검에서는 PR 전용 render-diff 자동화는 실행하지 않았다.

## 5. 산출물

디버그 SVG:

- `output/poc/task1310/stage8_equation_only_full_indent/3-09월_교육_통합_2022_010.svg`

일반 SVG:

- `output/poc/task1310/stage8_equation_only_full_indent_plain/3-09월_교육_통합_2022_010.svg`

비교 이미지:

- `output/poc/task1310/visual_compare/hancom_stage8_plain_q12_formula_side_by_side_3x.png`

render-tree 좌표:

| 항목 | x | y | width | right |
|---|---:|---:|---:|---:|
| 첫 수식 row TAC 1 | 402.5 | 592.5 | 78.2 | 480.7 |
| 첫 수식 row TAC 2 | 480.7 | 593.1 | 102.2 | 582.9 |
| 자동 wrap row | 483.2 | 630.3 | 182.8 | 666.0 |
| 세 번째 물리 row TAC 1 | 483.2 | 668.2 | 112.2 | 595.4 |
| 세 번째 물리 row TAC 2 | 595.5 | 668.2 | 117.9 | 713.4 |
| 네 번째 물리 row TAC 1 | 483.2 | 703.6 | 102.2 | 585.4 |
| 네 번째 물리 row TAC 2 | 585.5 | 703.6 | 41.9 | 627.4 |
| 네 번째 물리 row TAC 3 | 627.4 | 711.6 | 17.5 | 644.9 |

첫 수식 row는 문단 첫 줄 원점 x=402.5에서 시작한다. 자동 wrap row와 후속 물리 row는
한컴 UI 내어쓰기 60.5pt 전체 기준 x=483.2에서 시작한다.

조판부호 보정 SVG:

- `output/poc/task1310/stage9_control_marker_fixed/3-09월_교육_통합_2022_010.svg`
- `output/poc/task1310/stage9_control_marker_fixed_tree/render_tree_010.json`

조판부호 확인 좌표:

| 항목 | x |
|---|---:|
| 첫 수식 block 강제 줄넘김 | 666.05 |
| 다음 수식 block 강제 줄넘김 | 713.40 |
| 마지막 수식 block 문단 끝 | 644.88 |

## 6. 시각 검증 보조 판정

한컴 PDF 10쪽 crop과 rhwp 일반 SVG crop을 좌우 비교했다.

비교 대상:

- 왼쪽: `pdf/3-09월_교육_통합_2022.pdf` 10쪽 crop
- 오른쪽: `output/poc/task1310/stage8_equation_only_full_indent_plain/3-09월_교육_통합_2022_010.svg` 변환 crop

관찰:

- 연속 TAC 수식 3개 중 세 번째 수식이 다음 visual row로 내려감
- 첫 수식 row는 문단 첫 줄 원점에서 시작함
- 자동 wrap row와 후속 물리 row가 한컴 UI 내어쓰기 60.5pt 전체 기준 원점에서 시작함
- Stage 4에서 보였던 3, 4번째 줄의 좌우 흔들림은 해소됨
- 작업지시자 시각 판정 통과

## 7. 완료 판정

작업지시자 시각 판정 통과:

- `output/poc/task1310/stage8_equation_only_full_indent_plain/3-09월_교육_통합_2022_010.svg`
- `output/poc/task1310/visual_compare/hancom_stage8_plain_q12_formula_side_by_side_3x.png`

WASM 동작 테스트 중 확인된 미주 가상 문단 세로 커서 이동 panic은 Stage 7에서 보정했다.
이후 확인된 미주 가상 문단 오른쪽 방향키 이동 실패는 Stage 8에서 보정했다.
9쪽 미주 수식 문단의 커서 좌표 되감김은 Stage 9에서 보정했다.
9쪽 미주 수식-only 문단의 인접 수식 경계 중복 caret stop은 Stage 10에서 보정했다.

최종 검증:

- 시각 판정 통과
- WASM 동작 테스트 통과
- GitHub Actions `ci.yml` 기준 사전점검 통과
