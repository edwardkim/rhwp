---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 9 — p728 RowBreak 표의 fragment별 table-footnote 예약

## 문제와 독립 기준

Stage 8 뒤 개인정보 제거 원본의 쪽수는 HWP 225/HWPX 224이고, 각각의 한컴오피스 2020 PDF는
215쪽이었다. 원본 HWP·HWPX, 두 기준 PDF의 SHA-256과 보관 위치는
[증적 보관 목록](../../pdf/pr3740/README.md)에 고정돼 있다.

첫 물리 흐름 분기는 HWP PDF p66–p67의 p728 7×2 `RowBreak` 표 23이다.

| 기준 PDF | 수정 전 rhwp | 수정 후 rhwp |
| --- | --- | --- |
| p66: 표 0–4행(Organ Donation까지), 각주 76·77 | 모든 table-cell 각주 294.0px를 선예약해 표 전체가 p67로 이월 | 표 0–4행과 각주 76·77을 p66에 배치 |
| p67: Stephanie/Policy 5–6행부터 재개 | p66에서 밀린 표와 후속 내용이 연쇄 이동 | 5–6행부터 재개 |

## 원인 경로와 설계

`src/renderer/typeset.rs`의 `FormattedTable`은 표 셀 footnote를 height/count 하나로 합산하고,
`typeset_block_table_inner`는 fragment row range를 고르기 전에 그 전체를 예약했다. p728에서는
표 전 본문 cursor가 `713.8px`, 전체 예약 뒤 table 가용 높이가 `622.2px`가 되어 첫 row의
실제 잔여 영역도 사라졌다. 반면 table body와 전체 각주는 새 page 한 장에는 들어간다.

수정은 아래의 좁은 형상에 한정한다.

1. 비-inline, `RowBreak`, 다행, rowspan 없는 표이며, table body+모든 table-cell footnote가
   fresh page에는 들어가지만 현재 page에서는 전체 예약 때문에 첫 row를 못 넣는 경우만 queue한다.
2. `FormattedTable`이 각 note의 source와 content height를 순서대로 보존하고,
   continuation cursor가 이미 page에 등록한 note index를 보유한다.
3. 첫 fragment가 확정된 뒤 현재 page에 들어가는 note만 등록한다. 단일단의 중간 fragment는
   다음 fragment가 새 page에서 시작하므로 일반 본문 후속용 safety margin을 중복 차감하지 않는다.
   이 조건으로 p66의 table footnote 77까지 수용한다.
4. non-queue table의 기존 마지막-page 일괄 수집은 그대로 두며, queue table만 중복 수집을 건너뛴다.

진단 trace에서 p728은 `end_row=5`와 `end_row=7`을 기록했다. 즉 0-based row 0–4/5–6의 두
fragment다. 전체 HWP 쪽수도 225→224로 감소했다.

## 회귀 경계

focused fixture test
`tests/issue_3738_rowbreak_table_footnote_fragment.rs`는 실제 HWP의 p66/p67 text ownership과
`page_count() <= 224`를 고정한다. Stage 9 gate는 표 전체가 fresh page에 들어가고 rowspan이 없는
작은 표로 제한되어 거대 `#1937` 형상에는 적용되지 않는다.

기존 `tests/issue_1937_rowbreak_footnote_overpagination.rs`는 이 수정 전 `HEAD`를 `git archive`로
별도 빌드해도 41쪽으로 이미 실패했다(테스트의 역사적 하한 45와 불일치). 따라서 이 Stage의
pass/fail 근거로 사용하지 않았고, 별도 기존 테스트 기대값 정리가 필요하다.

## 결과와 다음 단계

p66의 표·각주 ownership은 기준 PDF에 맞았지만 p67 automatic sweep에는 `frame_overflow_pixels`
후보(하단 35px)가 남는다. 전체도 HWP 224/HWPX 224/PDF 215로 아직 9쪽 차이다. 상세 PNG와 지표는
[Stage 9 visual sweep](task_m100_3738_stage9_visual_sweep.md)에 기록했다.

따라서 이 Stage는 첫 table-footnote 선예약 분기만 해결한 것으로 커밋하고, 남은 p67 footnote
area와 다음 page-count 분기는 커밋 뒤 Stage 10에서 새로 분석한다.
