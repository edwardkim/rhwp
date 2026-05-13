# 구현 계획서 — Task #853 (M100)

GitHub Issue: edwardkim/rhwp#853 · 브랜치: `local/task853` (← upstream/devel `2bd50a3a`)
수행 계획서: `mydocs/plans/task_m100_853.md`

## 전제 (작업지시자 승인 — 기본값 적용)

- RFC #774 범위: **(a) shortcut.hwp 한정 정합만** 수행. RFC #774 의 일반 알고리즘 분석은 별도 유지.
- 3쪽 pi=94/95(`<편집 화면 분할에서>` ↔ "화면 이동") vpos=0 겹침: **#768 영역으로 분리** 가정. 단 Stage 1 진단에서 본 타스크 수정과 동일 근원으로 밝혀지면 범위 흡수(Stage 1 보고서에서 확정).

## 단계 구성 (5단계)

### Stage 1 — 진단 (코드 미수정)
- `dump-pages` / `dump` / `ir-diff`(필요 시 hwpx 변환본) / `export-svg --debug-overlay` 로 shortcut.hwp 1·2·3쪽 zone 배치·LINE_SEG vpos·TAC 표 outer_margin·anchor 문단 PS 정밀 관측.
- `pdf/basic/shortcut-2022.pdf` 와 px 단위 대조: ① 구분 칸 띠 위 여백 ② 띠 아래 → 첫 본문행 여백 ③ 제목 위 여백 ④ zone 전환(1단↔2단 `다단나누기`) 시 삽입 간격.
- 한컴 암묵 수직 간격의 **출처 후보 판정**: (가) zone 전환 시 고정/비율 간격, (나) TAC 표 anchor 문단 line-height(`line=100%`) 기반, (다) LINE_SEG `vpos`/`lh`/`th` 해석 누락, (라) 표 outer_margin top/bottom 적용 누락.
- 영향 코드 경로 특정: `src/renderer/layout.rs`(zone 배치 누적 offset, `start_new_column_band`, `layout_table_item` is_tac 분기, 페이지 break 판정), `src/renderer/layout/paragraph_layout.rs`, `src/document_core/` LINE_SEG.
- 3쪽 본문영역 초과(콘텐츠 y > body_bottom 758.4) 가 (1) 띠 압축으로 zone 과적재의 결과인지 (2) 페이지 break 판정 자체의 버그인지 분리.
- 산출: `mydocs/working/task_m100_853_stage1.md` — 진단 결과 + 규명된 근원 + (필요 시) 구현 계획 v2 간소화 + #768 흡수 여부 결정.

### Stage 2 — 구분 칸 암묵 수직 간격 재현
- Stage 1 에서 규명된 근원에 한해 최소 수정. 후보 우선순위: (라) 표 outer_margin top/bottom → (가) zone 전환 간격 → (나) TAC anchor line-height.
- shortcut.hwp 의 TAC 표 띠(`size=...×1766` 6.2mm, outer_margin 1mm, anchor PS `line=100%`)와 `다단나누기` zone 전환 양쪽에 적용.
- 변경 후 shortcut.hwp 7쪽 `export-svg` → 구분 칸 위·아래 여백 PDF 대비 ±6px 수렴 확인.
- 산출: `mydocs/working/task_m100_853_stage2.md` + 소스 커밋.

### Stage 3 — 페이지 break 판정 보강 (필요 시)
- Stage 2 적용 후에도 본문영역 초과(콘텐츠 y > body_bottom)가 남으면, 누적 zone offset 이 body_bottom 초과 시 다음 페이지로 break 하도록 `src/renderer/layout.rs` 페이지 break 판정 보강.
- TAC 표 띠 + 다단 zone 조합에서 break 검사가 우회되는 경로 차단.
- Stage 2 만으로 초과가 해소되면 본 단계는 "수정 불요" 보고로 마감.
- 산출: `mydocs/working/task_m100_853_stage3.md` (+ 소스 커밋 또는 불요 사유).

### Stage 4 — 광역 회귀 검증
- `cargo test`(svg_snapshot 8/8 포함) 전건 통과 확인.
- `cargo clippy --lib -- -D warnings` (pre-existing 무관 오류는 명시).
- 광역 fixture sweep(전 샘플 SVG 내보내기) → shortcut.hwp 외 변경 영향 0(byte-identical) 또는 의도된 변경만임을 입증.
- shortcut.hwp 7쪽 SVG ↔ `pdf/basic/shortcut-2022.pdf` 시각 정합 최종 확인. (한컴 2010/2020 환경 차이는 `feedback_pdf_not_authoritative` 정합으로 함께 점검 — macOS 환경이라 `pdf/`(2022) 1차, 가능 시 `pdf-2020/` 보조.)
- 산출: `mydocs/working/task_m100_853_stage4.md` + 소스 커밋(있다면).

### Stage 5 — 최종 결과보고서
- 결과 요약(증상1/2 해소 여부), 변경 내역, 회귀 검증 결과, 잔여 사항(#768 등).
- 산출: `mydocs/report/task_m100_853_report.md` + 커밋. (mydocs/orders 는 수정하지 않음 — 작업지시자 거버넌스 영역.)

## 회귀 가드 요약

- 매 단계 `cargo test` 통과 필수, Stage 4 광역 sweep 필수.
- 본질 정정(zone 배치/페이지 break)이므로 다단·단일 단·표분할 상호작용 회귀를 sweep 으로 입증(`feedback_essential_fix_regression_risk`).
- 한컴 spacing 규칙은 룰/휴리스틱 구분 후 적용(`feedback_rule_not_heuristic`).
