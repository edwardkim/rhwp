# Stage 146: HWPX 병렬 규정 표 중간 reserve 분석

## 목적

Stage 145에서 PDF p364의 제63조의3 본문 tail owner를 복원한 뒤에도 남아 있는
p361~p363의 제61조~제63조의3 지연 owner를 분석한다. 전역 cut reserve를 추가로
완화할 때 383쪽, p310 blank, p314 제2장, p365~p367 후반 조문 owner가 유지되는
범위를 확정한다.

## 현재 관측

- 200px reserve에서는 p364가 제63조의3 heading부터 시작했다.
- 180px reserve에서는 p364가 `관리하여야 한다` 본문 tail부터 시작해 PDF 방향으로
  개선됐다.
- 180px reserve에서도 p361~p363은 각각 제61조, 제63조, 제63조의2 heading이
  조기에 시작한다. r84의 남은 높이는 0px이므로 r84 local cut이 아니라 이전
  complete-row 누적이 원인이다.

## 분석 절차

1. 160px 후보를 렌더해 p361~p365의 조문 marker와 전체 쪽수를 측정한다.
2. 전역 160px가 쪽수나 후반 owner를 깨면 180px로 되돌리고, r79의 병렬 오른쪽
   셀 continuation만 별도 측정한다.
3. p365~p367, p310, p314 계약이 모두 유지되는 r79 후보만 PDF p361~p364 owner와
   비교한다.
4. 채택한 변경은 source HWPX page tree 회귀와 결과를 이 문서에 기록하고 같은
   커밋으로 완료한다.

## 분석 및 구현

- 전역 reserve 160px는 p361~p363을 앞당겼지만 총 382쪽이 되었고, 제65조·제69조·부칙
  owner도 각각 p364·p365·p366으로 한 쪽 앞당겨져 폐기했다.
- r79은 왼쪽 제57조와 오른쪽 제43조~제44조를 병렬로 가진 장문 행이다. 180px
  reserve에서는 p360에 오른쪽 제44조 heading만 남아 p361의 제60·61조가 늦어진다.
- r79만 -180px로 확장하면 p360에서 제44조 continuation을 추가 소비하고, 383쪽을
  유지한 채 p361에 제61조, p362에 제62·63조, p363에 제63조의2·3을 배치한다.
- r79 -360px는 위 조문을 더 앞당기지만 전체가 382쪽이므로 폐기했다. 따라서
  `HWPX_PARALLEL_REGULATION_R79_CUT_RESERVE_PX = -180.0`만 채택한다.

## 결과

- HWPX render tree는 한컴 PDF와 같이 383쪽이다.
- p310 blank, p314 제2장 시작, p365 제65~68조, p366 제69·70조와 시행규칙
  제47·48조, p367 부칙 owner를 유지한다.
- PDF p361~p363의 제61조, 제63조, 제63조의2·3 owner를 같은 쪽으로 복원했다.
- p361의 제60조는 PDF처럼 tail만 남는 대신 heading도 함께 남아 있다. 이 작은
  잔여 차이는 다음 Stage에서 별도 분석하며, 이번 r79 변경으로 전역 페이지 수나
  후반 owner를 보상하지 않는다.

## 회귀

- `issue_3930_preserves_page_count_and_inherited_even_master_page`에 p361 제61조,
  p362 제63조, p363 제63조의2·3 page owner를 추가했다.
- focused integration test 결과는 이 문서의 커밋 전 실행 결과로 기록한다.

## 상태

focused integration test는 3 passed, 0 failed로 통과했다. 새 릴리스 준비 중이므로
merge, push, PR 생성 또는 원격 변경은 금지한다.
