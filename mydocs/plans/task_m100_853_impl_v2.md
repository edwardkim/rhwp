# 구현 계획서 v2 (Stage 1 진단 반영) — Task #853 (M100)

GitHub Issue: edwardkim/rhwp#853 · 브랜치: `local/task853`
원본 구현 계획서: `mydocs/plans/task_m100_853_impl.md` · Stage 1 진단: `mydocs/working/task_m100_853_stage1.md`

## 확정 사항 (작업지시자 승인 — "진행"으로 기본값 채택)

- Stage 2 = **옵션 A 변형**: `is_column_top` 시 `spacing_before` 를 통째 드롭하지 않고 **해당 문단의 첫 LINE_SEG `vpos`(한글이 실제 렌더한 위치)로 클램프** — `applied_before = min(spacing_before, hwpunit_to_px(line_segs[0].vpos))`. `vpos=0` 인 문단(본문 첫 줄 등)은 종전과 동일(0). `vpos>0` 인 섹션-top/`다단나누기` band-top(제목 vpos=1984 등)은 그만큼 적용.
- 3쪽 본문영역 초과(#768 패턴 pi=94/95 vpos=0 겹침)는 **본 타스크 흡수** — Stage 3 에서 처리. 단 한글 PDF 페이지별 콘텐츠 대조로 "한글도 3쪽 초과인가" 먼저 확인 후 범위 확정.
- 제목 PUA 첫 글자(`\u{f53a}`) 폰트 폴백은 본 타스크 제외.

## 단계 (5단계 — 원본과 동일, Stage 2/3 구체화)

### Stage 2 — column-top 문단 `spacing_before` 의 LINE_SEG.vpos 클램프
- `src/renderer/layout/paragraph_layout.rs:745-748` (`layout_composed_paragraph` 내):
  - 현행: `if start_line == 0 && spacing_before > 0.0 && !is_column_top { y += spacing_before; }`
  - 변경: `is_column_top` 일 때도 `start_line == 0 && spacing_before > 0.0` 이면 `y += min(spacing_before, vpos0_px)` 적용 (`vpos0_px = para.and_then(|p| p.line_segs.first()).map(|ls| hwpunit_to_px(ls.vpos as i32, self.dpi)).unwrap_or(0.0)`). `!is_column_top` 분기는 종전대로 `y += spacing_before` 전량.
- `src/renderer/height_measurer.rs` — column-top 문단 높이 계산이 `spacing_before` 전량을 포함하면, 위 클램프와 정합되도록 동일 클램프 적용(페이지네이션↔배치 비대칭 해소). 단 height_measurer 가 column-top 컨텍스트를 모르면, 보수적으로 "전량 포함" 유지 시 발생하는 영향(예약 과다 → 페이지 break 빨라짐)을 Stage 4 sweep 으로 점검 후 필요 시 수정.
- 검증: shortcut.hwp 7쪽 `export-svg` → 제목 top ≈ 83px, "커서 이동" 띠 ≈ 127px, 구분 칸 위·아래 여백 PDF ±6px 수렴.
- 산출: `mydocs/working/task_m100_853_stage2.md` + 소스 커밋.

### Stage 3 — 3쪽 본문영역 초과 (#768 패턴)
- 한글 PDF 3쪽 콘텐츠 범위 대조 → 한글도 초과면 "정상"(수정 불요), rhwp 만 초과면 처리.
- pi=94(`<편집 화면 분할에서>`) / pi=95("화면 이동") 둘 다 vpos=0 겹침 원인 규명: `다단나누기` zone 분할 시 두 번째 문단의 vpos 가 0 으로 리셋되어 첫 문단 위에 겹침 → `src/renderer/layout.rs` zone 분할/`start_new_column_band` 부근에서 zone 내 문단 누적 y 가 vpos=0 을 잘못 해석하는 경로 수정.
- 누적 zone offset 이 body_bottom 초과 시 다음 페이지로 break 하는 가드 점검/보강.
- 산출: `mydocs/working/task_m100_853_stage3.md` (+ 소스 커밋 또는 불요 사유).

### Stage 4 — 광역 회귀 검증
- `cargo test`(svg_snapshot 8/8 포함) 전건 통과. `cargo clippy --lib -- -D warnings`(pre-existing 무관 오류 명시).
- 전 fixture SVG sweep → shortcut.hwp 외 byte-identical 또는 의도 변경만 입증. (`feedback_essential_fix_regression_risk` 정합 — 다단/단일 단/표분할 상호작용 회귀 점검.)
- shortcut.hwp 7쪽 SVG ↔ `pdf/basic/shortcut-2022.pdf`(+ 가능 시 `pdf-2020/`) 시각 정합 최종 확인.
- 산출: `mydocs/working/task_m100_853_stage4.md` + 커밋.

### Stage 5 — 최종 결과보고서
- `mydocs/report/task_m100_853_report.md` + 커밋. orders 미수정.

## 회귀 가드

- column-top 클램프는 `vpos` 기록값을 상한으로만 쓰므로 "vpos 무시" 현행 대비 항상 ≥ 0, ≤ spacing_before — 과도 적용 위험 없음.
- 매 단계 `cargo test` 통과 필수, Stage 4 sweep 필수.
- vpos 해석 변경은 `respect_vpos_reset` 류 전면 변경과 무관(여기서는 column-top 첫 줄 한정 상한 클램프).
