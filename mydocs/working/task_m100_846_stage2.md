# Stage 2 보고 — Task #846 (M100) — 구현 + 회귀 발견 (선행 이슈 #849 분리)

상태: **"마지막 단 단나누기 → 같은 페이지 새 밴드" 구현 완료(워킹트리 반영, 미커밋) → shortcut.hwp 정합, 그러나 exam_math 회귀 잔존 → 선행 이슈 #849 로 분리**.

## 1. 확정한 처리 규칙

다단 zone 에서 명시적 단나누기(`ColumnBreakType::Column`, 새 ColumnDef 없음)가 밴드의 **마지막 단**을 끝낼 때:

- 단이 그냥 꽉 차서 넘치는 경우(overflow)는 항상 새 페이지 — 같은 페이지 새 밴드를 만들지 않는다. **명시적 단나누기일 때만** "같은 페이지 새 밴드" 를 시도한다.
- 새 밴드 생성 가능 여부: `누적_밴드_높이 + 현_밴드_높이 < 본문_가용_높이` 이면 같은 페이지 새 밴드, 아니면 새 페이지.
  - `현_밴드_높이` = 그 밴드 각 단의 **채움 길이의 최댓값**.
- 다음 밴드로 들어갈 콘텐츠에 떠다니는(글자처럼 취급 아닌) 표·그림·그리기 개체가 있으면 무조건 새 페이지.
- 새 밴드는 직전 밴드의 가장 높은 단 바로 아래에 적층, col_count 유지.

## 2. 구현 (`src/renderer/typeset.rs` — 워킹트리 반영, 미커밋)

- `paginate` 명시적 단나누기 경로: `ColumnBreakType::Column` + `!has_diff_col_def` + 마지막 단(`current_column+1 >= col_count`) + `col_count > 1` → `advance_column_or_new_page`(→push_new_page) 대신 신규 `start_new_column_band`.
- `start_new_column_band`:
  1. `flush_column`
  2. 다음 밴드 콘텐츠(`para_idx` ~ 다음 나누기/새 ColumnDef 직전)에 떠다니는 표·그림·그리기 개체가 있으면 `push_new_page` 후 종료.
  3. 현 밴드 높이 = 그 밴드 각 단의 마지막 문단 `vpos_end` 중 최댓값 (실제 채움 길이의 근사 — §4 참조).
  4. `available_height() - 밴드높이 >= 이_문단_첫줄_높이` 이면 새 밴드(zone_y_offset 진행, col 0 리셋), 아니면 `push_new_page`.
- `process_multicolumn_break` 는 변경 없음 (`[다단나누기]` 동작 보존).

## 3. 결과

| 샘플 | 한컴 PDF | baseline | 구현 후 | 판정 |
|------|----------|----------|---------|------|
| `basic/shortcut.hwp` | 7 (2022) | 8 | **7** | ✅ pi=94/95(`<편집 화면 분할에서>` \| `화면 이동 ⟶ Ctrl+W,N`) 페이지 3 이동, 총 페이지 PDF 일치 |
| `exam_math.hwp` | 20 | 18 | **11** | ❌ baseline 보다 악화. 원인 §4 |
| `21_언어_기출_편집가능본.hwp` | 15(2020)/16(2010) | 15 | **15** | △ 총 페이지 수 동일하나 페이지 8/9 콘텐츠 시프트 — `test_539_partial_paragraph_after_overlay_shape`, `test_548_cell_inline_shape_first_line_indent_p8` FAILED ("페이지 8 셀 5 line 0 [푸코] rect 못 찾음") |
| `cargo test` 전건 | — | 1232 pass | **1229 pass / 3 fail** | 위 3건 |

## 4. exam_math 회귀 잔존 원인 — 다단 밴드 높이 과소추정

exam_math 는 `[단나누기]`×32(다단정의 0), 문제마다 큰 수식/도형 박스 포함. 페이지 3 예: 단 0(items=38) 의 vpos 기반 밴드 높이 ≈ 710px 로 산출되지만, 실제 콘텐츠(153px+ 짜리 수식·도형 박스를 가진 문제 3~4개)는 ~1100px 에 가까움. 즉 **`max(단별 마지막 문단 vpos_end)` 추정이 단 안에 인라인 수식·도형 개체가 있을 때 실제 단 채움 길이를 크게 과소추정** → 밴드가 너무 짧게 잡혀 한 페이지에 밴드가 과도하게 들어감 → 18→11.

- 이 과소추정은 본 변경 이전부터 존재(`process_multicolumn_break` 도 동일 추정 사용)했으나, exam_math 는 `[다단나누기]` 가 없어 그 경로를 안 타서 드러나지 않았음. 본 변경이 `[단나누기]` 경로에도 같은 밴드-적층을 도입하면서 노출됨.
- 21언어 페이지 8/9 시프트도 같은 류(단별 밴드 적층 시 높이 추정 부정확)로 추정.
- → **이슈 #849 (다단 밴드/단 실제 채움 높이 산출 정합)** 로 분리. 그 작업 후 본 §2 구현 합류 시 회귀 해소 예상.

## 5. 현재 상태 / 다음 단계

- `src/renderer/typeset.rs` 의 §2 구현이 워킹트리에 반영됨(미커밋). `cargo test` 3건 실패 상태.
- 본 타스크는 #849 완료 시 재개 (Stage 3 광역 회귀 검증 → Stage 4 최종 보고). 그 전까지 merge 금지.
