# 구현 계획서 — Task #866 (M100)

GitHub Issue: edwardkim/rhwp#866 · 브랜치: `local/task866` (← `pr-task853`) · 수행 계획서: `task_m100_866.md`

## 단계 (4단계)

### Stage 1 — PDF 측정 + IR 대조 (코드 미수정)
- 빌드: `pr-task853` 상태 그대로(PR #868 적용). `cargo build --release` (필요 시).
- shortcut.hwp 의 `쪽나누기` 시작 페이지(2쪽 "파일", 그리고 같은-페이지 `다단나누기` zone 들)에서 헤더 TAC 띠 상단/하단/본문 첫 줄 y 를 `mutool draw -r 100` 픽셀 측정(@96dpi 환산) → `dump`/`dump-pages` 의 IR(line_segs vpos/lh/ls, 표 size·outer_margin·`쪽나눔` attr, ColumnDef 간격·column_type)과 대조.
- 검증 항목: ① ~28px gap 이 모든 `쪽나누기` 헤더 띠에서 일정한가 / ② `다단나누기`(쪽나누기 아닌)로 시작한 헤더 띠에서도 나타나는가 / ③ 표 size·outer_margin·`쪽나눔=RowBreak(0x04...)` 비트와 상관관계 / ④ 후속 본문 zone 의 첫 LINE_SEG vpos 가 0 인가(= 본문이 zone 상단에 붙는가) → 28px 가 zone 진입 offset 인지 본문 paragraph 의 leading 인지.
- 산출: `task_m100_866_stage1.md` — 측정표 + 후보 1~3 중 특정(있으면) + 정정 위치/방법 또는 "미특정 → 보류" 판정. (보류 시 Stage 2~3 생략하고 Stage 4 보고서로 마감.)

### Stage 2 — 구현 (Stage 1 에서 근원 특정 시)
- 규명된 경로(`typeset.rs::place_table_with_text` 또는 `process_multicolumn_break`)에 최소 수정. 룰로 명세 가능하면 단일 룰, 아니면 휴리스틱 vs 룰 자문 후 적용.
- 검증: shortcut.hwp 7~8쪽 SVG ↔ `pdf/basic/shortcut-2022.pdf` — 헤더 띠↔본문 거리 PDF ±6px 수렴.
- 산출: `task_m100_866_stage2.md` + 소스 커밋.

### Stage 3 — 광역 회귀
- `cargo test --release` 전건 + `cargo clippy --lib -- -D warnings`(pre-existing 무관 오류 명시). 전 fixture SVG sweep → shortcut.hwp 외 byte-identical 또는 의도 변경만. shortcut.hwp ↔ `pdf/`(2022) (+ 가능 시 `pdf-2020/`) 시각 정합.
- 산출: `task_m100_866_stage3.md` (+ 커밋).

### Stage 4 — 최종 보고서
- `mydocs/report/task_m100_866_report.md` + 커밋. 필요 시 `mydocs/tech/hancom_zone_paragraph_spacing.md` 갱신. (#867 은 #866 결과 보고 별도 재평가 — 본 타스크 범위 외.)

## 회귀 가드
- 매 단계 `cargo test --release` 통과 필수. Stage 3 sweep 필수. 한컴 정답지 등급(`pdf/` 2022 1차, `pdf-2020/` 보조; `pdf-2010/` 등급 미달).
- 측정만으로 출처 미특정 시 강행하지 않고 보류(`feedback_essential_fix_regression_risk`).
