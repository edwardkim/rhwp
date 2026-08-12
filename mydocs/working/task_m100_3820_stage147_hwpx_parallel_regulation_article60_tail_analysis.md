# Stage 147: HWPX 병렬 규정 표 제60조 tail owner 분석

## 목적

Stage 146 뒤에도 남은 2025 행정업무운영 편람 HWPX 병렬 규정 표의 최초 owner
차이를 분석한다. PDF p316은 왼쪽 제8조(r12)로 시작하지만 HWPX는 r11 제7조의
tail만 보인 뒤 r12를 p317로 미룬다. 383쪽과 Stage 146의 p361~p367 owner를
유지한 채, r12가 p316 하단에서 partial-row로 시작해야 하는지 확인한다.

## 고정 계약

- HWPX와 PDF는 모두 383쪽이다.
- p310 blank, p314 제2장 시작, p361 제61조, p362 제63조, p363 제63조의2·3,
  p364 제63조의3 tail, p365~p367 후반 조문·부칙 owner를 유지한다.
- 전역 reserve 160px와 r79 -360px는 382쪽을 만들어 이미 반증됐다. 이 Stage에서
  전역 reserve를 다시 낮추지 않는다.

## 분석 절차

1. PDF p315~p316과 HWPX render tree에서 r11~r12의 좌·우 셀 fragment와 y 범위를
   비교한다.
2. r11 continuation의 blank band와 r12 첫 partial-row cut의 budget을 확인한다.
3. PDF가 요구하는 제8조 prefix를 p316에 남길 수 있는 단일 row reserve 조건이 있을
   때만 코드를 변경한다.
4. 채택한 구현은 source HWPX page tree 회귀, 결과, focused integration test와 함께
   같은 커밋으로 완료한다.

## 상태

분석 결과 r11이 p316에서 442px을 소비한 뒤 r12에 약 87px이 남지만, 전역 180px
reserve가 cut budget을 0으로 만들어 r12의 첫 유닛만 orphan으로 남긴다. r12 전용
reserve 0px을 모든 continuation에 적용한 후보는 381쪽으로 붕괴해 폐기했다. 첫 r12
fragment(`row_start_cut`이 비어 있음)에만 적용하는 후보를 측정한다. 새 릴리스 준비
중이므로 merge, push, PR 생성 또는 원격 변경은 금지한다.

## 구현 및 결과

- r12의 첫 fragment에만 `HWPX_PARALLEL_REGULATION_R12_CUT_RESERVE_PX = 0px`을
  적용했다. r12 continuation에는 기존 전역 180px reserve를 그대로 적용한다.
- 이 범위는 r11 continuation 뒤 남은 약 87px에서 r12의 제8조 prefix를 시작하게 하며,
  모든 r12 continuation에 적용했던 381쪽 후보와 달리 한 fragment만 바꾼다.
- HWPX render tree는 383쪽이다. PDF와 같이 p316이 제8조 prefix로 시작한다.
- p310 blank, p314 제2장 시작, p361 제61조, p362 제63조, p363 제63조의2·3,
  p364 제63조의3 tail, p365~p367의 조문·부칙 owner도 유지했다.

## 회귀

- `issue_3930_preserves_page_count_and_inherited_even_master_page`에 PDF p316의
  제8조 prefix source owner를 추가했다.
- focused integration test는 3 passed, 0 failed로 통과했다.
