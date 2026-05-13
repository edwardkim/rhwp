# 구현 계획서 v3 (범위 확대) — Task #853 (M100)

GitHub Issue: edwardkim/rhwp#853 · 브랜치: `local/task853`
선행: v1 `task_m100_853_impl.md`, v2 `task_m100_853_impl_v2.md` · Stage 1 진단 `task_m100_853_stage1.md`

## 범위 (작업지시자 지시: "제목 + band 간격 + overflow 전부")

shortcut.hwp 의 3개 결함을 모두 정정한다 — ① 섹션-top 제목 spacing_before 압축, ② `다단나누기` 구분 칸 band 위·아래 간격 압축, ③ 3쪽 본문영역 초과(#768 패턴). 본질 정정 영역(RFC #774) — 광역 회귀 + 한컴 정답지 검증 필수(`feedback_essential_fix_regression_risk`).

## Stage 2 — 섹션-top 제목 spacing_before 클램프 (재적용, 완료)
- `src/renderer/layout/paragraph_layout.rs`: `is_column_top && para_index == 0` 일 때 `y += spacing_before.min(hwpunit_to_px(line_segs[0].vertical_pos).max(0.0))`. 페이지 break 후 column-top(`para_index>0`)은 종전대로 0.
- 효과: shortcut.hwp 제목 baseline 79.4 → 105.8 (top ≈ 83.8px ≈ 한컴 PDF 83.6px). svg_snapshot 2건(KTX p1, exam_kor p5)의 섹션-시작 문단도 LINE_SEG.vertical_pos 기준으로 재배치 → `UPDATE_GOLDEN` (한컴이 파일에 기록한 실제 렌더 위치와 정합하므로 개선). `cargo test --release` 전건 통과.
- 잔존: shortcut.hwp 7쪽 → 8쪽 (Stage 3 의 band 정정 후 재확인 — band 가 커지면 한컴처럼 7쪽 가능 여부 미정).

## Stage 3 — `다단나누기` 구분 칸 band 위·아래 간격 + 3쪽 overflow
- **진단(Stage 1/2 에서 파악)**: 헤더 띠 문단(예: pi=36 "파일")의 LINE_SEG 는 line0=텍스트(lh=1200HU≈16px) + line1=표(lh=2332HU = 표 1766 + outer_margin 283×2 ≈ 31px) = 47px 인데, rhwp 는 표를 line0 에 놓고 텍스트 line0 을 흡수해 ≈27px → ~20px 부족(= stage5 "헤더↔본문 ~20px"). 또 zone 사이(제목 zone↔헤더 띠↔본문 zone)에 한컴이 두는 간격(~17px)이 rhwp 에는 0. 3쪽 단3 pi=94/95 vpos=0 겹침(#768 패턴).
- **방향(미확정 — 추가 진단 필요)**: (a) 헤더 띠 문단을 LINE_SEG 순서대로(text line0 → table line1) 렌더하도록 composer/`layout_table_item` 정정, (b) zone 전환 간격의 출처 규명(LINE_SEG.vertical_pos / `다단나누기` 후 첫 zone offset / 빈 continuation line) 후 정정, (c) 3쪽 overflow = (a)(b) 정정 후 자연 해소되는지 확인 + 안 되면 page break 가드 보강.
- 이 단계는 composer/table layout 변경으로 회귀 위험이 크므로, **Stage 3-1(추가 진단·설계 문서) → 승인 → Stage 3-2(구현)** 로 쪼갠다. 진단·설계는 `mydocs/tech/` 또는 본 working 문서에 정리.
- 산출: `task_m100_853_stage3.md` (+ 소스 커밋, 필요 시 RFC #774 일부 흡수).

## Stage 4 — 광역 회귀 검증
- `cargo test --release` 전건 + `cargo clippy --lib -- -D warnings`(pre-existing 무관 오류 명시). 전 fixture SVG sweep → shortcut.hwp/KTX/exam_kor 외 byte-identical 또는 의도 변경만. shortcut.hwp 7~8쪽 SVG ↔ `pdf/basic/shortcut-2022.pdf`(+ 가능 시 `pdf-2020/`) 시각 정합. 한컴 2010/2020 환경 차이 점검(`feedback_pdf_not_authoritative`).
- 산출: `task_m100_853_stage4.md` + 커밋.

## Stage 5 — 최종 결과보고서
- `mydocs/report/task_m100_853_report.md` 갱신(v2) + 커밋. orders 미수정.

## 회귀 가드
- Stage 2 클램프: `vertical_pos`(파일 기록값) 상한 → 항상 `0 ≤ applied ≤ spacing_before`. 과도 적용 불가.
- Stage 3: composer/table layout 변경 — 매 변경 후 `cargo test --release` 통과 필수, Stage 4 sweep 필수, 한컴 정답지 비교.
