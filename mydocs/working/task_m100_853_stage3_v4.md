# Stage 3-3 조사 보고 — Task #853 (M100) — 잔존 zone-transition gap

GitHub Issue: edwardkim/rhwp#853 · 브랜치: `local/task853` · 상태: **조사 — 미수정. 한컴 정답지 cross-check 필요(RFC #774).**

## 측정 (Stage 3-2 적용 후 상태)

`process_multicolumn_break` 가 새 zone 의 `zone_y_offset` 를 `직전 문단 last_seg.vertical_pos + line_height + line_spacing` 로만 계산 → 한컴 PDF 대비 부족:

| 페이지 | 요소 | 한컴 PDF (body_top 기준) | rhwp | 차이 | rhwp 산식 |
|--------|------|------------------|------|------|-----------|
| 1 | "커서 이동" 헤더 띠 | +87.6px | +72.9px | ~15px 부족 | 제목 zone vpos_end(69.1) |
| 1 | 본문 첫 줄 | +137.9px | +100.2px | **~38px 부족** | 69.1 + 헤더 zone vpos_end(31.1) |
| 2 | "파일" 헤더 띠 | +19.1px | +19.8px | ✓ (Stage 3-2) | line0(16) + outer_margin |
| 2 | 본문 첫 줄 | ~+75px | ~+45px | **~28~30px 부족** | 헤더 zone vpos_end(47.1) |

## 분석

### 페이지 1 — ~38px ≈ ColumnDef `간격` 10mm (37.8px)
pi=1(헤더 띠)의 `다단나누기` ColumnDef = `1단, 간격=10.0mm(2835HU=37.8px)`. pi=2(본문)의 ColumnDef = `2단, 배분, 간격=1.0mm`. pi=0(제목)의 ColumnDef = `1단, 간격=0mm`. 한컴은 이 1단 ColumnDef 의 `간격`(원래 단 사이 가로 간격이지만 1단이라 가로로는 무의미)을 **세로 zone 진입 간격**으로 쓰는 것으로 보임 — 본문 부족분(~38px) = pi=1 의 `간격` 10mm 와 일치. 다만 제목↔헤더 +14.7px / 헤더↔본문 +23px 로 정확히 50/50 분할은 아님(≈40/60) → 정확한 분배 규칙 미확정.

### 페이지 2 — ~28px 미규명
pi=36(헤더 띠)의 ColumnDef = `1단, 간격=0mm`. 따라서 §1 의 ColumnDef-간격 가설로는 0px → 페이지 2 본문 부족분 ~28px 가 설명 안 됨. 후보(분석 문서 §5): TAC 표 `wrap=위아래(TopAndBottom)` 가 글자처럼 취급이면서도 위아래 어울림으로 추가 예약 / `쪽나눔=RowBreak(attr=0x04000006)` 처리 / 1단→2단 zone 전환 고정 간격. 3쪽 이후 띠들 PDF↔IR 추가 측정 + 한컴 편집기(Windows) cross-check 필요.

## 권고

§1(ColumnDef 간격 → zone 진입 간격)은 페이지 1 을 개선하지만 분배 규칙이 불확정이고, §2(페이지 2 ~28px)는 출처 미규명이라, 추측 구현 시 광역 회귀 위험 큼(`feedback_essential_fix_regression_risk`, `feedback_rule_not_heuristic`). **이 두 항목은 한컴 정답지 cross-check + 추가 샘플 측정 후 처리** 권고.

본 타스크는 **Stage 2(섹션-top 제목 정정) + Stage 3-2(헤더 띠 line0 텍스트 배치)** 로 마무리 — 둘 다 한컴 PDF 정합 + `cargo test` 전건 통과 + svg_snapshot 8/8. 잔존(zone-transition gap §1/§2, 페이지 수 7≠8, overflow 16건)은 분석 문서 `mydocs/tech/hancom_zone_paragraph_spacing.md` 에 정리 — 후속 타스크에서 RFC #774 와 함께 처리.

## 커밋
Stage 2(`f0d34713`) → Stage 3-1 분석(`bd9d5148`/`0fca34ed`) → Stage 3-2(`c765e36b` 실패보고 + `1f7328b2` 성공) → 본 조사 보고. 소스 추가 변경 없음.
