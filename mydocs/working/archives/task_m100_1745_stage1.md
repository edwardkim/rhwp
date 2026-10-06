# Task #1745 Stage 1 완료보고서 — 재현 샘플 동결 + 실패 재현

## 수행 내용
- `samples/task1745/table_text_anchor_wrap.hwp` 동결 (출처: 법령 별표 서식 18177503, 18KB) + README.
- 실패 재현 (HEAD 92a204f5 기준 빌드):
  ```
  rhwp dump-pages samples/task1745/table_text_anchor_wrap.hwp
  → 페이지 3:  FullParagraph  pi=1  vpos=2072  ls=1[... cs=45568 sw=2620]  "(빈)"
  ```
  pi1 이 표 아래 3쪽에 일반 배치됨. 한글(OLE 실측·저장 LINE_SEG)은 1쪽 표 오른쪽 잔여 띠.

## 확인 사항
- 저장 LINE_SEG 의 cs=45568 = 표폭 45002 + 바깥여백 283×2 → Stage 2 의 expected_cs 공식과 일치.
- dump-pages 는 흡수 시 `WrapAroundPara pi=` 행을 내므로(document_core/queries/rendering.rs:3262)
  수정 후 검증 도구가 페이지를 인식 가능.

## 상태
완료. Stage 2 (wrap strip cs/sw 표 geometry 도출) 진행.
