# Stage 3-2 (1차 시도) 보고 — Task #853 (M100) — 헤더 띠 line0 텍스트 렌더

GitHub Issue: edwardkim/rhwp#853 · 브랜치: `local/task853` · 상태: **1차 시도 효과 없음 → revert. 추가 진단 필요.**

## 시도한 변경 (revert 함)

`src/renderer/pagination/engine.rs::place_table_fits`:
- `pre_table_end_line` 계산에 분기 추가: `is_tac_table && total_lines > 1` 이면 표 컨트롤이 놓인 LINE_SEG 인덱스(`control_text_positions()[ctrl_idx]` ↔ `line_segs[i].text_start` 비교, `find_inline_control_target_page` 와 동일 방식)를 `pre_table_end_line` 로 사용 → 표 앞 줄(텍스트)을 표보다 먼저 배치.
- `post_table_start` TAC 분기: `pre_table_end_line.max(1)` → `(pre_table_end_line + 1).min(total_lines).max(1)`.

## 결과

빌드 성공, **shortcut.hwp SVG byte-identical (변화 없음)** — pi=36("파일" 헤더 띠)이 여전히 표를 line0 에 놓고 텍스트를 흡수, 8쪽 유지. `cargo test` 미실행(효과 없어 의미 없음).

→ pi=36 의 TAC 표가 `place_table_fits` 의 `pre_table_end_line` 경로를 타지 않거나, `pre_table_end_line` 계산이 0 으로 떨어짐(추정: `control_text_positions()` 의 단위 ↔ `line_segs.text_start`(UTF-16) 불일치, 또는 pi=36 이 `쪽나누기`로 새 페이지 시작 시 다른 경로로 처리). revert.

## 다음 — 추가 진단 필요 (Stage 3-1b)

pi=36 의 레이아웃 경로를 디버그 계측으로 추적해야 함:
- `RHWP_LAYOUT_DEBUG=1` 등으로 pi=36 의 ComposedLine 수, MeasuredParagraph.line_heights, 어느 함수(`place_table_fits` / `split_table_rows` / 다른 경로)가 PageItem 을 생성하는지, `control_text_positions()`[table] 값과 `line_segs.text_start` 값을 출력.
- 그 결과로 "표가 놓인 줄 인덱스" 를 정확히 산출하는 위치/방법 확정 후 재시도.

회귀 위험: composer/pagination 변경은 전 문서 영향 → 매 변경 후 `cargo test --release` + 전 fixture sweep 필수(`feedback_essential_fix_regression_risk`).

## 현재 상태 정리

- Stage 2(섹션-top 제목 정정) = 커밋 `f0d34713`, 유지. shortcut.hwp 제목 정합, `cargo test` 전건 통과, golden 2건 갱신.
- Stage 3(band 간격 + overflow) = 미완. 1차 시도 효과 없음, 추가 진단 대기.
- 소스 추가 변경 없음(engine.rs revert 됨). 빌드 바이너리는 무효과 변경 포함 상태 → 다음 작업 시 재빌드 필요.
