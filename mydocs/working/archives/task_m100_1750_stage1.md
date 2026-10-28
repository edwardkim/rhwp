# Task #1750 Stage 1 완료보고서 — 재현 샘플 동결 + 실패 재현

## 수행 내용
- `samples/task1750/split_guard_spacing_before.hwp` 동결 (법령 별표 서식 3024019, 20KB) + README.
- 실패 재현 (HEAD 57ea1ef0, upstream/devel + PR #1744/#1746 적용 베이스):
  ```
  페이지 1: 단 0 (items=23, used=1010.9px)   ← avail 1005.1px 초과 (5.8px overfill)
    PartialParagraph  pi=22  lines=0..1  vpos=700
  ```
  한글(OLE) 캐럿과 저장 LINE_SEG(다음 줄 vpos=700=새 쪽 상단)는 pi22 전체를 2쪽에 배치.

## fit 트레이스 근거 (조사 시 실증, 이슈 #1750 본문)
`FIT_DEBUG pi=22 cur_h=976.0 h_fit=41.6 avail=1005.1 normal_fits=false` — 전체 배치 탈락 후
분할 진입 가드 `remaining(29.1) < first_line_h(25.6)` 가 spacing_before(9.3px) 를 빼지 않아
페이지 넘김 생략 → 분할 루프 첫 줄 무조건 배치.

## 상태
완료. Stage 2 (가드 수정 + 테스트) 진행.
