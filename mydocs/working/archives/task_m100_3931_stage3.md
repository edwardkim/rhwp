# Task M100 #3931 — Stage 3 declared 선이월 판정

- 날짜: 2026-08-15 KST
- 기준: Stage 2 commit `6845461ad`
- 대상: sec=10 `pi=14` native HWP5 다행 RowBreak 표
- 상태: 구현 및 focused 회귀 통과

## 결과

`pi=14`는 `cur_h=566.7px`, 선언 높이 175.2px로 합계 741.9px가 1px tolerance를 0.1px
넘는다. 기존 경로는 실제 fragment scan보다 먼저 표 전체를 새 쪽으로 보냈다. 저장 anchor는
545.5px, 저장 하단은 719.4px이며, cell 내부 문단 reset과 다음 source 문단 되감김이 모두 있다.

ordinary-row 전용 near-anchor 경로를 다음 조건으로만 rowspan 표에도 열었다.

- native HWP5, 비-TAC TopAndBottom, 다행 RowBreak, 표 각주 없음
- 저장 anchor drift 24px 이내, 저장 하단이 현재 본문 안에 있음
- 다음 source 문단 되감김
- rowspan이 있으면 cell 내부 저장 `vpos` reset이 반드시 존재

이 조건에서 기존 scanner는 현재 쪽 예산 174.1px 안에 155.8px 첫 조각을 만들고, row 4의
hard-break에서 `[1, 1, 1]` 컷을 남긴다. 질문 7의 첫 답변 문단은 page index 286, 나머지 답변은
index 287에 속한다. 한컴 2020 PDF의 물리 p284/p285 2조각 소유권과 같은 형태이며 두 조각 모두
본문 bbox 안에 있다.

## HWPX 판정

현재 HWPX 386쪽은 이미 declared 통이월 경로가 아니라 fragment scanner를 탄다. `pi=14`에서
`cur_h=591.2px`, 첫 예산 149.6px 중 rows 0..4가 101.1px를 소비하고, 남은 48.5px에 답변 첫
문단이 들어가지 않아 질문은 index 285, 답변은 index 286에 배치된다. 따라서 HWPX에 native
저장-anchor 특례를 적용하지 않았다.

한컴 PDF는 질문과 첫 답변 문단을 p284에 함께 둔다. 이 한 문단 차이는 `pi=14` 진입 시점의
HWPX 흐름 높이가 HWP보다 24.5px 큰 선행 조판 누적 문제이며, #3931의 declared 선이월 문턱과
다른 축이다. HWPX 386쪽과 기존 scanner 경로를 래칫해 이번 수정의 무회귀 조건으로 둔다.

## Red-check와 회귀

- rowspan의 내부 reset 허용 팔을 제거하면 `pi=14` head/tail이 모두 index 287로 돌아가 신규
  계약이 의도대로 실패했다. 팔을 복원하면 286/287로 통과한다.
- `issue_3931_declared_rowbreak`: 5 통과, 1 전체 페이지 오라클 ignore
- `issue_3738_rowbreak_table_footnote_fragment`: 33/33 통과
- #3930, #874, #2097, #2105, #2439, #3236, #1156, #1748 focused 회귀: 전건 통과
- HWP 전체는 392쪽, HWPX 전체는 386쪽이다. `pi=14` 뒤의 재조판 상쇄로 HWP 총쪽수는 Stage 2와
  같지만 대상 fragment의 물리 소유권은 교정됐다.

## Stage 4 실행 지점

총쪽수 383을 한 가지 문턱 수정으로 강제하지 않는다. #3931의 두 직접 좌표 `pi=14`와 `pi=23`,
HWPX 기존 scanner 경로, overflow 부재를 광범위 회귀와 비공개 코퍼스로 검증한다. 이후 Native
Skia와 WASM을 빌드하고 `output/3931/`에 한컴 PDF 대조 시각 근거를 만든다.
