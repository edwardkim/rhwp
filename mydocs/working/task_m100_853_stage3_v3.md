# Stage 3-2 (2차 시도, 성공) 보고서 — Task #853 (M100) — 헤더 띠 line0 텍스트 렌더

GitHub Issue: edwardkim/rhwp#853 · 브랜치: `local/task853`
선행: Stage 3-1 분석(`task_m100_853_stage3.md`, `mydocs/tech/hancom_zone_paragraph_spacing.md`), Stage 3-2 1차 시도 실패(`task_m100_853_stage3_v2.md`).

## 1차 실패 원인 (규명)

1차는 `pagination/engine.rs::place_table_fits` 를 고쳤으나 효과 없음 — shortcut.hwp 는 **`typeset.rs` 경로**를 타며 `engine.rs::paginate_table_control` 은 호출되지 않음(디버그 print 미발화로 확인). 실제 헤더 띠 PageItem 생성 지점은 `typeset.rs::place_table_with_text`. 또 `control_text_positions()` 는 `char_offsets` 가 비면 무용(`[0, 0]` 반환)이라 "표가 놓인 줄 인덱스" 산출에 못 씀.

## 2차 변경 (성공) — `src/renderer/typeset.rs::place_table_with_text`

- `pre_table_end_line` 계산에 분기 추가: `table.common.treat_as_char && total_lines > 1 && para.text.chars().any(is_alphanumeric)` 이면, **표 줄의 높이(표 본체 + outer_margin top/bottom)와 일치하는 LINE_SEG 인덱스**를 `pre_table_end_line` 로 사용 → 그 앞 줄(텍스트)을 `PageItem::PartialParagraph{0..pre}` 로 표보다 먼저 emit.
  - `control_text_positions()` 대신 `line_height` 매칭으로 표 줄을 찾음(char_offsets 무관).
  - PUA 필러/공백만 있는 문단(복학원서.hwp pi=16 등 — 한컴이 표 폭만큼 필러로 줄바꿈시킨 케이스)은 `is_alphanumeric()` 가 false 라 제외 → `compute_tac_leading` 경로 유지(Task #842 결함 #2 정합).
- `tac_wrap_split` 플래그(`treat_as_char && pre>0 && pre<total_lines`): true 면 ① 높이 누적은 `table_total_height`(=`fmt.height_for_fit`, pre 줄 포함) 만 — `pre_height` 이중 계산 방지, ② `post_table_start = (pre+1).min(total).max(1)` — 표가 차지한 줄을 post-text 에서 제외.

## 결과

| 항목 | 변경 전 | 변경 후 | 한컴 PDF (2쪽) |
|------|---------|---------|----------------|
| dump-pages pi=36 PageItem 순서 | Table → PartialParagraph(1..2) | **PartialParagraph(0..1) → Table** | — |
| "파일" 헤더 띠(표) 상단 (body_top 기준) | +3.8px | **+19.8px** | +19.1px ✓ |
| 헤더 띠 하단 | +27.3px | **+43.3px** | +43.1px ✓ |
| 본문 첫 줄 "새 문서" 상단 | ~+29px | ~+45px | ~+75px (잔존 ~30px 부족) |
| shortcut.hwp LAYOUT_OVERFLOW 이벤트 | 25 | **16** | — |
| 페이지 수 | 8 | 8 | 7 |

- ✅ 헤더 띠 내부 배치(텍스트 line0 → 표 line1) 정합 — 띠 상/하단 위치가 한컴 PDF 와 일치.
- ✅ `cargo test --release` 전건 통과(34 suites ok), svg_snapshot 8/8(golden 무변경).
- ⚠ 잔존: 띠↔본문 ~28~30px gap(= 분석 문서 §5 미규명: 1단→2단 zone 전환 간격 / TAC `wrap=위아래` 추가 예약 / `쪽나눔=RowBreak` 중 하나) + 1단 ColumnDef `간격` 10mm(§3) + 페이지 수 7≠8 + 3쪽/6쪽 overflow 16건.

## 다음 (Stage 3-3) — 잔존 항목

분석 문서 `mydocs/tech/hancom_zone_paragraph_spacing.md` §3/§5/§6:
1. 띠↔본문 ~28px gap 의 출처 확정 — 3쪽 이후 띠들 PDF↔IR 추가 측정 필요.
2. 1단 ColumnDef `간격`(10mm = 37.8px) → zone 진입 top 간격 적용 여부 — 한컴 정답지 cross-check 필요, 전 fixture sweep 회귀 점검.
3. page break 가드(누적 offset > body_bottom 시 다음 페이지) — overflow 16건 / 페이지 수 정합.

## 커밋
Stage 2 (`f0d34713`) → Stage 3-1 분석(`bd9d5148`/`0fca34ed`) → Stage 3-2 1차 실패 보고(`c765e36b`) → 본 변경(`typeset.rs` +33/-2) + 보고서.
