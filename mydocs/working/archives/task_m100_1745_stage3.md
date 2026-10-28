# Task #1745 Stage 3 완료보고서 — typeset/layout 적용 + 재현 검증

## 수행 내용
### typeset (`src/renderer/typeset.rs`)
1. **wrap zone 활성화**: Square wrap 표 처리 후 `text_anchor_square_table_strip` 이 Some 이면
   도출된 띠 (cs, sw) 를 `wrap_around_cs/sw` 에 저장 (None 이면 기존 첫 LINE_SEG 값 — 무변경).
   후속 문단(cs=45568, sw=2620)이 기존 정확 일치 매칭으로 흡수됨.
2. **`record_wrap_around_para` 신설**: 흡수된 WrapAroundPara 를, anchor 표의 첫 fragment
   (비연속 PartialTable/Table)가 현재 column 에 없으면 `st.pages` 에서 찾아 그 column 에
   소급 push (한글: 어울림 문단은 표 시작 쪽 옆 띠에 배치). 미발견 시 현행 유지.

### layout (`src/renderer/layout.rs`) — 계획서 "추가 변경 없음" 예상과 달리 필요 확인
조사 결과 wrap 렌더 x/폭도 anchor 첫 LINE_SEG 에서 취하고 있었고(전폭 제목 줄 → 띠 아님),
분할 표 경로는 호스트 텍스트를 일반 경로(분할 표 첫 부분)와 wrap 경로에서 이중 렌더할 위험.
- `layout_wrap_around_paras` 에 `strip_x/strip_width`(후속 어울림 문단 전용 띠)와
  `render_host_text`(호스트 이중 렌더 방지) 파라미터 추가. 표 단독 anchor 는 기존 값과
  동일하게 전달되어 무변경.
- 호출부 2곳(통짜 표 / 분할 표): 헬퍼로 띠 계산. 분할 표는 텍스트 혼합 anchor 일 때
  `render_host_text=false` (호스트 제목은 기존 일반 경로가 렌더).

## 재현 검증
| 항목 | 수정 전 | 수정 후 |
|------|---------|---------|
| dump-pages pi1 | 3쪽 FullParagraph | **1쪽 WrapAroundPara** (한글 정합) |
| 총 페이지수 | 3 | 3 (불변) |
| verify_pi_page_vs_hangul | PI_MISMATCH(pi1 p3↔p1) | **MATCH** |
| SVG 1쪽 (제목 렌더) | '약' 글리프 3회 | 3회 — 이중 렌더 없음, 크기 동일(286081B) |

## 상태
완료. Stage 4 (회귀 검증 + 최종보고) 진행.
