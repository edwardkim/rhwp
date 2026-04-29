# Task #455 구현 계획서 — 인라인 글상자 본문 누락 수정

## 단계 구성

총 4단계.

### Stage 1 — 진단: 어디서 line 2 본문이 사라지는지 정확히 식별

목적: 다음 셋 중 어느 것이 원인인지 확정한다.

1. `composer::compose_paragraph` 의 `composed.lines[1].runs` 가 비어 있음 (composer 버그).
2. 비어 있지 않은데 `paragraph_layout::layout_paragraph` 가 line 2 의 text run 렌더링을 스킵함 (layout 버그).
3. 렌더링은 했는데 좌표/clip 으로 화면 밖에 그려짐 (rendering 버그).

방법:
- 임시 stderr 디버그 prints 를 `compose_paragraph` 직후, `layout_paragraph` 의 `for line_idx in start_line..end` 루프 진입 직후, 그리고 텍스트 런 렌더 직전에 삽입.
- 트리거 조건: `para_index==33 && section_index==0` 만.
- 출력: 각 line 의 line_idx, runs.len(), runs.text 합계, line.line_height, line.segment_width, 그리고 tac_offsets_px.
- `cargo run --release -- export-svg samples/exam_kor.hwp` 로 실행.

산출물: `mydocs/working/task_m100_455_stage1.md` — 어느 단계가 원인인지 + 다음 수정 위치 확정.

검증: 디버그 prints 제거 후 빌드 정상.

### Stage 2 — 수정: 원인 단계의 코드 변경

Stage 1 결과에 따라 다음 중 하나(이상)를 수정:

- **Composer 버그라면**: `compose_lines` 또는 `find_control_text_positions` 에서 `Shape` + `treat_as_char` 컨트롤이 line text 추출을 막지 않도록 정리.
- **Layout 버그라면**: `paragraph_layout.rs` 의 `has_tac_shape` 분기, 또는 inline 위치 계산에서 line 2 텍스트가 통째로 빠지는 경로를 수정. 가능성 높은 영역: text run 의 x 시작 위치가 inline shape 폭만큼 잘못 점프하거나, `clipPath` / `available_width` 계산이 음수가 되는 경로.
- **상위 레벨 버그라면**: `layout.rs` 의 `prev_has_overlay_shape` 등 wrap=TopAndBottom 분기가 tac=true 인 글상자를 체커에 포함시켜 다음 줄 vpos 보정이 어긋나는 경우. 해당 분기에 `&& !cm.treat_as_char` 가 누락되었는지 확인.

수정 후:
- `cargo build --release`.
- `rhwp export-svg samples/exam_kor.hwp` → `output/svg/exam_kor/exam_kor_002.svg` 좌측 단 y≈321 줄에 39자가 다시 나타나는지 확인.

산출물: `mydocs/working/task_m100_455_stage2.md`.

### Stage 3 — 회귀 검증

1. `cargo test` 전체 통과.
2. 페이지 수 점검:
   - `exam_kor.hwp` 20페이지 유지.
   - 다른 주요 샘플(`exam_eng.hwp`, `2010-01-06.hwp`, `21_언어_기출_편집가능본.hwp`) 페이지 수 변화 없음.
3. `ir-diff` (해당 샘플의 hwpx 가 있다면) 차이 0 유지.
4. Task #332/#409/#412/#452 등 회귀 핫스팟 SVG 비교(눈으로 좌·우 단 정렬 점검).

산출물: `mydocs/working/task_m100_455_stage3.md` — 페이지 수 표 + 통과한 회귀 항목.

### Stage 4 — 최종 보고

- `mydocs/report/task_m100_455_report.md` — 원인 요약, 수정 패치, 회귀 결과.
- `mydocs/orders/{오늘날짜}.md` 갱신.
- 최종 보고서 + 오늘할일 갱신은 task 브랜치에서 커밋.
- `local/task455` → `local/devel` merge 는 작업지시자 승인 후.

## 위험 / 롤백

- 수정 범위는 `treat_as_char=true` 인 Shape/Picture 한정.
- 회귀 발생 시 변경된 `if` 조건만 되돌리면 즉시 롤백 가능.
- 글상자 외 다른 inline 컨트롤(Picture/Equation/Table tac) 동작은 변경 금지.
