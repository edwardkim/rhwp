# Task #1745 Stage 2 완료보고서 — wrap 띠 도출 헬퍼 + 단위테스트

## 수행 내용
- `src/renderer/mod.rs`: `text_anchor_square_table_strip(para) -> Option<(cs, sw)>` 추가.
  - 텍스트 혼합 anchor(첫 LINE_SEG cs=0 전폭 텍스트 줄) + 비-TAC Square wrap 좌측정렬 표에서
    띠 시작 cs = `horizontal_offset + margin.left + width + margin.right`,
    띠 폭 sw = `첫 seg 폭 − cs` 로 도출 (재현 파일 저장 LINE_SEG 와 정확 일치:
    cs=45568=45002+283×2, sw=2620=48188−45568).
  - 표 단독 anchor(첫 seg cs>0), 텍스트 없음, 좌측정렬 아님, 띠 폭 ≤0 → None (기존 경로).
- 단위테스트 2개: geometry 도출 / None 가드 3종(표 단독·무텍스트·전폭 표). 통과.

## 구현계획서 대비 조정
- 계획서의 "Stage 2 = typeset 활성화 수정"에서 **활성화 적용은 Stage 3 으로 이동**
  (typeset.rs·layout.rs 적용 변경을 한 단계로 묶어 파일 단위 커밋 정합). 본 단계는
  헬퍼 + 테스트만 포함.

## 상태
완료. Stage 3 (typeset/layout 적용) 진행.
