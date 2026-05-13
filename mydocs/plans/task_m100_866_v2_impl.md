# 구현 계획서 — Task #866 v2 (M100): shortcut.hwp 구분 칸 전환 간격 정밀화 + 점선 단 구분선

GitHub Issue: edwardkim/rhwp#866 · 브랜치: `pr-task853` (PR #868 위, `local/task866` 연장)

## 배경

PR #868(Task #853/#866) 이후 잔존:

1. **band↔body / band↔band 전환 간격이 한컴 PDF 대비 일부 부족** — 사용자 보고("모든 구분 칸 위·아래 줄 간격 좁음"). 예: 1쪽 `오른쪽 단으로`↔`지우기` 띠 (rhwp ~30px vs PDF ~38px). #866 Stage 3 의 `tac_band_extra`(= `table.height + om_top + om_bot`)·zone 디자인 spacing /2 규칙이 일부 전환에서 과소.
2. **점선 단 구분선(`구분선 type=3`) 미렌더** — 1쪽 "세로 DOT 선 안 보임".
3. **(이미 코드에 반영, 미커밋) pi=94 `<편집 화면 분할에서>` 회귀** — 다단 zone 마지막 컬럼 `[단나누기]` 가 무조건 새 페이지로 밀던 것을 `process_multicolumn_break` 로 라우팅 → 3쪽 정합 + 페이지 수 8→7 (PDF 7쪽 정합). 본 v2 Stage 1 과 함께 커밋.

본문 항목(예: pi=2~15) 간 간격은 rhwp SVG 가 이미 ~20px(=`lh`+`ls`)로 렌더 → PDF 정합 (`dump-pages` 의 `diff=-86.7px` 는 metadata 측 별도 measurer 의 artifact, 렌더 현실 아님).

## 단계

### Stage 1 — 정밀 측정 + pi=94 커밋
- `pdftoppm -r 96` 로 `pdf/basic/shortcut-2022.pdf` 1~7쪽 PNG 추출 → 각 띠("커서 이동"/"지우기"/"파일"/"보기"/"입력"/"서식" …)·그룹·`<...>` 소제목의 y 좌표 측정.
- `rhwp export-svg shortcut.hwp` → 각 페이지 SVG 의 동일 요소 y 좌표 추출 (text transform translate-y).
- PDF↔SVG 1:1 대조표 → 전환별 Δ(px) 확정. `tac_band` 과대/과소, 디자인 spacing /2 규칙 적합성 판정.
- pi=94 회귀 수정 코드 + 측정 결과를 `working/task_m100_866_v2_stage1.md` 와 함께 커밋.

### Stage 2 — 전환 간격 정정
- Stage 1 측정 기반으로 `typeset.rs::process_multicolumn_break` 의 `tac_band_extra` / `st.current_zone_design_spacing_px` 규칙 + `layout.rs::build_columns` 의 미러를 조정.
- 매 변경: `cargo test --release` + shortcut.hwp 7쪽 SVG↔PDF 재확인. `LAYOUT_OVERFLOW` 추이 확인 (목표 0).

### Stage 3 — 점선 단 구분선
- `ColumnDef.separator_type` (1쪽 = type 3 = dotted) 인 다단 zone 의 단 사이 세로 구분선을 점선으로 렌더 (현재 미렌더 또는 실선 여부 먼저 확인).
- `samples/` 내 다른 단 구분선 사용처(있으면) 회귀 확인. svg snapshot 영향 확인.

### Stage 4 — 광역 회귀 + 보고서
- 전 fixture `cargo test --release`, svg snapshot, 전 샘플 sweep (`LAYOUT_OVERFLOW`).
- `pdf-2010/` 등 보조 자료와도 cross-check (가능 범위).
- `report/task_m100_866_report.md` v2 갱신, orders 갱신.

## 회귀 위험
- Stage 2 (zone 전환 간격) — `feedback_essential_fix_regression_risk` 영역. 전 fixture sweep 필수, 한컴 2010/2020 PDF cross-check.
- Stage 3 (구분선 렌더) — 신규 렌더 경로 추가라 기존 영향 작음. snapshot 확인으로 충분.
- pi=94 수정 — 다단 zone 마지막 컬럼 `[단나누기]` 경로 한정. 같은 페이지 여유 부족 시 종전과 동일(새 페이지).
