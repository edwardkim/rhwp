# Stage 3 보고서 — Task #1082: pi 인덱싱 정합 해결 + C군 = 다단 미주 드리프트 특정

- 브랜치: `local/task1082`
- 수정: `src/document_core/queries/rendering.rs` (dump_page_items 미주 인덱싱 정합)

## pi 인덱싱 미스터리 해결
`DIAG_PI` 측정: `section.paragraphs.len()=488` 인데 typeset 결과 `max_para_index=992`.
- 원인 확정 (`typeset.rs:1419`): `en_para_idx = paragraphs.len() + endnote_paragraphs.len()`.
  → **para_index 0~487 = 본문, 488~992 = 미주(endnote) 문단** (시험지 정답/해설 ~504개).
- `dump_page_items` 는 `section.paragraphs`(488)만 인덱싱 → 미주(488+) `.get()` None → "" 표시
  (본문이 빈 것처럼 보이던 착시의 정체). 렌더는 `combined = body + endnote_paragraphs`(rendering.rs
  :2531, Task #836)로 정상 해석.

## 수정 (dump_page_items 정합)
- 미주가 있으면 `combined = body_paragraphs + pr.endnote_paragraphs` 로 인덱싱.
- 미주 항목은 `FullParagraph[미주]` 로 마킹.
- 검증: page 16 pi=818~827 이 이제 미주 텍스트 표시
  (`"문25) 23_09 교육 25) ②"`, `"등차수열 의 공차를 ..."`). 본문 아님 확정.

## C군 재특정 (결정적)
- "빈 문단 다수 페이지"의 정체 = **다단 배치 미주(정답/해설)**. C군 overflow(col 1 드리프트)는
  **typeset.rs 의 다단 미주 레이아웃 드리프트** (#1062 시험지 미주 영역의 다단 케이스).
- engine.rs(RHWP_USE_PAGINATOR=1) 는 전 파일 0 overflow → 미주 다단 레이아웃이 정상.
  typeset.rs 미주 다단 경로(typeset.rs:1402~1500, en_fit/en_advance)만 드리프트.

## 검증
- lib 1336 passed, clippy/fmt clean. (dump_page_items 표시 정합만 변경, 렌더/페이지네이션 무영향.)

## 다음 (Stage 4 — 실제 수정)
typeset.rs 다단 미주 레이아웃(1402~1500) vs engine.rs 미주 다단 비교 → 드리프트 원인(컬럼
advance/vpos 누적) 특정 → 정합. 이제 dump-pages 가 미주를 정확히 보여주므로 단계별 비교 가능.
