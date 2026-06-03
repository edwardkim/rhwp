# 구현계획서 v2 — Task #1062: 미주(Endnote) 누적 vpos 정합

- 이슈: edwardkim/rhwp#1062 (미주 레이아웃으로 정정)
- 브랜치: `local/task1062`
- 선행: revised Stage 1(`_stage1_v2`), revised Stage 2(`_stage2_v2`) — 근본 원인=미주 누적 trailing_ls 과소.
- 이전 impl(본문 trailing_ls/vpos-delta)은 **폐기**.

## 핵심

미주 페이지네이션(`typeset.rs:1395-1453`)의 누적이 `height_for_fit`(trailing_ls 제외)이라
렌더러 vpos 전진(lh+ls)보다 미주당 6px 과소 → 페이지당 미주 과밀 → 본문 하단 overflow.
미주 누적을 **렌더러 vpos 전진과 통일**한다. 본문/표 누적은 불변.

## 단계 (3단계)

### Stage 3 — 구현
- `typeset.rs:1452` 미주 누적: `if col_count>1 { height_for_fit } else { total_height }` →
  **미주 vpos 전진**(`last.vpos+last.lh+last.ls − first.vpos`, line_segs 기반). 1425-1432의
  끝위치 계산과 동일 식 재사용.
- `typeset.rs:1440` fit 판정: 누적과 일관되게 정합(마지막 항목 trailing_ls 제외 의미 유지).
- 미주 line_segs 부재 시 total_height fallback.
- 단위 TDD: 단일 줄 미주 누적 = lh+ls 검증, 미주 없는 다단 불변 검증.

### Stage 4 — 회귀 검증
- 대상 4종(×2): LAYOUT_OVERFLOW 대폭 감소 + `pdf/3-09월_교육_통합_2022.pdf` 등 쪽수·미주 배치 정합.
- 비회귀: endnote-01, footnote-01, exam_eng/exam_kor/k-water-rfp, 복학원서, 골든 SVG 회귀 0.
- 전 251 샘플 LAYOUT_OVERFLOW 합계(devel 1624) 악화 없음.
- `cargo test --release` 0 fail, `cargo fmt`(변경 파일).

### Stage 5 — 최종 보고
- `report/task_m100_1062_report.md` + orders 갱신은 작업지시자 관할.

## 완료 기준
대상 4종 미주 overflow 해소(쪽수 PDF 정합) + 비회귀 0 + 골든 회귀 0.

## 리스크
- 미주 vpos 전진이 다중 줄에서 formatter와 미세 차이 → Stage 4 실측 확정.
- 미주가 단(column)을 넘는 분할(PartialParagraph) 경로도 동일 정합 필요 — Stage 3에서 점검.
