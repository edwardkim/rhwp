---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 21 — p154–158 RowBreak 표의 page-map 분기 복원

## 출발 근거

[Stage 20 candidate ledger](task_m100_3738_stage20_candidate_ledger.md)는 기준 PDF p1–215와 현재 native
HWP p1–220을 text-only·render-tree로 전수 수집했다. 그 결과:

- `export-svg --json`의 `overflowCellLines=26`이 rhwp p157에 집중됐다.
- rhwp p158 Body `Table`은 Footer 상단을 넘고 page frame 밖까지 내려간다.
- p155 이후의 큰 text multiset 차이는 이 표 fragment와 같은 page-map 분기의 결과일 수 있다. 이를 개별
  문자 결함 60여 건으로 세지 않는다.

조사 중 p157–158의 clip 직전인 p154–155에서 같은 1×1 empty-host RowBreak 표가 40px 각주 safety
margin 때문에 불필요하게 쪼개지며 이후 physical page map을 한 쪽 밀고 있음을 확인했다. 따라서 이 Stage의
실제 범위는 p154–158이다. p43·54·67·85·106은 Stage 20이 자동 후보로 재포착했지만, 이 원인군과 같다는
근거가 생기기 전에는 함께 보정하지 않는다.

## 분석 계약

1. selected high-DPI sweep은 p154–155·p157–158을 실행하고, PDF/rhwp review PNG·overlay·render tree를 증적으로 남긴다.
2. `dump-pages`, render-tree의 table `pi`, raw `dump --para`로 해당 표의 RowBreak/fixed-height/footnote/caption
   shape와 physical fragment를 추적한다.
3. 기준 PDF에서 p157–158의 표 행 경계와 footer 사이의 실제 owner를 확인한다.
4. 원인 확정 전에는 page count를 220→215로 맞추는 전역 보정이나 p43·54·67·85·106의 footnote reservation
   경로 변경을 하지 않는다.

## 원인 확정

### pi=1682 — p154의 불필요한 tail page

`dump --para 1682`는 비문자형 empty host, 1×1, `RowBreak` 표를 보였다. 표 자체 각주는 없고 실측 전체
높이는 p154의 기존 본문 각주 210 separator 직전까지 들어간다. 그러나 block-table 일반 경로는 기존 각주가
있는 경우 40px safety margin을 더 남겨 마지막 두 줄만 p155로 보냈다. 기준 PDF는 p154에서 이 표와 각주
210을 함께 끝내고, p155는 `(3) 평가 절차`로 시작한다. 이 tail이 이후 표/본문의 physical page owner를
한 쪽씩 어긋나게 했다.

### pi=1723 — p157–158 frame escape

같은 형상의 `pi=1723`은 선언 객체 높이 363.8px에 비해 셀 본문 실측 높이가 1163.8px였다. 저장
`LINE_SEG` fast lane이 선언 높이만 예약해 p157에는 표 시작을, p158에는 실제 셀 전체를 그리면서 footer
아래까지 탈출시켰다. 일반 block-table 경로는 셀 문단 unit으로 분할할 수 있으므로, 선언 높이가 실측의
1/1.5보다 작은 이 형상은 fast lane을 통과시키면 안 된다.

## 수정

- `try_typeset_empty_para_float_table`은 저장된 단일 TopAndBottom 1×1 `RowBreak` 표의 실측 높이가 선언
  높이의 1.5배를 초과하면 fast lane을 포기하고 기존 일반 block-table fragment 경로로 보낸다.
- `typeset_block_table_inner`는 native HWP5의 non-TAC empty-host 1×1 `RowBreak` 표만, 표 자체 각주가 없고
  실측 전체가 **실제** 기존 각주 경계 안에 들어갈 때 safety margin을 풀어 tail page를 만들지 않는다.
- 두 특례는 `treat_as_char`, 다중 셀/행, 새 페이지 시작, 표 자체 각주, 실제 물리 경계 초과에는 적용하지
  않으므로 기존 일반 pagination 계약을 넓히지 않는다.

## 결과

- native HWP 페이지 수: **220 → 219**. 이 변화는 p154의 pi=1682 tail page 제거에서만 온다.
- p154에는 pi=1682의 마지막 셀 문단과 각주 210이 함께 남고, p155는 `(3) 평가 절차`로 시작한다.
- p157은 pi=1723의 첫 fragment(`BTS Guideline`~`OPTN policy`), p158은 continuation(`BC Canada`) 및 뒤
  본문을 소유한다. 두 fragment 모두 render-tree footer 경계 안에 있다.
- 회귀는 `issue_3738_rowbreak_table_footnote_fragment` focused fixture 11/11으로 고정했다.
- 4쪽 PDF 직접 대조와 장기 PNG·지표는 [Stage 21 visual sweep](task_m100_3738_stage21_visual_sweep.md)에
  남긴다.

## 완료 기준

- p157 clip과 p158 table frame escape의 source contract 및 기준 PDF 행 owner를 문서화했다.
- p154–155의 같은 원인군을 함께 고정하고 focused regression과 선택 visual sweep으로 검증했다.
- p155에 기준 PDF에는 없는 분홍 흐름도 그림이 표·본문·각주 영역 위로 겹쳐 보이는 별도 결함도 발견했다.
  전체 기준 PDF는 215쪽, native HWP는 아직 219쪽이다. 이 결과는 p43·54·67·85·106 등 남은 후보의 해결이나
  전수 page-count 정합을 주장하지 않는다.
