# 최종 보고서 — Task #1082: 다단 미주 vpos 간격 누락 본문 하단 overflow

- 이슈: edwardkim/rhwp#1082
- 브랜치: `local/task1082` (stream/devel `fbfcf682` rebase)
- 수정: `src/renderer/typeset.rs`(다단 미주 누적 + 본문 last bottom vpos 트래커),
  `src/document_core/queries/rendering.rs`(dump_page_items 미주 인덱싱 정합) + 회귀 가드
  `tests/issue_1082_*.rs`

## 증상
시험지 정답/해설 미주가 2단 영역에 다수 배치된 페이지에서, 미주 본문이 페이지 하단을 최대
~900px 초과(3-09/10/11월 교육·실전 통합 5파일, hwp/hwpx 양쪽).

## 근본 원인
다단 미주(endnote) 레이아웃에서 typeset 누적이 미주 para **내부** vpos span 만 더해 미주 간
vpos 간격(빈 줄/문단 간격)을 누락 → 단(특히 col 1) 과충전 → 렌더는 vpos 정규화로 정확히 배치
하므로 단 콘텐츠가 단 높이를 초과 → 본문 하단 overflow.
- engine.rs 는 미주 레이아웃 코드 부재(콘텐츠 누락, oracle 아님).
- 렌더는 정상(layout.rs:2233 컬럼별 page_base; combined paragraphs 993).
- 정확한 결함: typeset.rs `en_advance = advance.max(height_for_fit)`, advance = para 내부 vpos
  span. 미주 간 vpos 간격이 누락.

## 수정
typeset 다단 미주 누적을 **직전 배치 아이템 bottom 기준 vpos delta(px)** 로 정합:
- `TypesetState.prev_body_bottom_vpos`(신규) — 본문 FullParagraph 배치 시 갱신, `flush_column`
  시 리셋. body→endnote 전환의 초기 base 시드.
- 미주 루프 진입 시 `prev_en_bottom_vpos = st.prev_body_bottom_vpos`(시드). 미주 advance 시
  None(자체 높이).
- `compute_en_metrics(prev)` — `advance_px = px(this.bottom_offset − base)`,
  `.max(fmt.height_for_fit)` 안전 floor 유지(#1062). fit/acc 분리 산출.
- 단단(col_count==1)은 종전(`fmt.total_height`).

부산물 — `dump_page_items` 가 미주 paragraphs(combined) 도 인덱싱(`FullParagraph[미주]` 마킹)
→ pi 488+ 미주를 빈 문단으로 오인하던 디버깅 차단(pi 인덱싱 정합).

## 검증
- **C군 5파일 max overflow**: 626.9/626.9/277/158.5/561 → **24.1/24.1/25.7/16.9/8.9**.
- **전수 sweep**(samples 281 hwp/hwpx) baseline 1156 lines/46163px → **1024/17386** (회귀 0,
  악화 파일 0, **-62% px**).
- 회귀 가드 4 신규 통과, 골든 8/8, cargo test lib **1336** + 통합 0 failed, clippy/fmt clean.

## 잔여 (known limitation, 실용 한계 내)
~25px 잔여 = 본문 fmt 누적의 trailing_ls overcount 가 미주로 전파되는 작은 base 드리프트.
전 단계(627px) 대비 95%+ 해소. 단단 본문 fmt 누적의 vpos 정합은 별도 과제.

## 후속
overflow 인벤토리 C군(다단 미주) 처리 완료. A/B/D군(#1070/#1073/#1079)과 함께 전 군 처리 완료.
