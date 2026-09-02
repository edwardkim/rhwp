# Stage 159: RowBreak visible tail geometry 계약

## 목적

`advance_row_cut_inner`와 `advance_row_block_cut`에 중복된
`ROWBREAK_VISIBLE_TAIL_OVERFLOW_TOLERANCE_PX`를 제거한다. 마지막으로 보이는 cell unit의
저장 line geometry와 실제 fragment 높이에서 컷 가능 여부를 판단한다.

## 분석 범위

- 단일 행 RowBreak cut과 rowspan block RowBreak cut의 동일한 tail 예외
- 저장 LINE_SEG, cell unit, padding, visible control geometry
- HWP/HWPX source profile이 아니라 실제 fragment geometry로 설명 가능한 조건

## 금지 조건

- 행 번호, 열 수, fixture, 문장, page index를 새 gate로 추가하지 않는다.
- 단일 행과 block 경로에 서로 다른 px allowance를 다시 만들지 않는다.
- 분석 문서만 커밋하지 않는다.

## 완료 기준

- 두 cut 경로가 하나의 visible-tail geometry 판정을 공유한다.
- 고정 120px tail allowance를 제거한다.
- 구현과 결과 보고서를 하나의 Stage 커밋으로 남긴다.

## 상태

완료.

## 분석 결과

- `ROWBREAK_VISIBLE_TAIL_OVERFLOW_TOLERANCE_PX=120`은 단일 행과 rowspan block cut에
  동일하게 복제되어 있었다.
- 두 경로 모두 현재 unit이 가시 line이고, 뒤에 구조적 spacer가 있어야만 tail을
  유지했다. 즉 허용 대상은 임의 높이가 아니라 **현재 fragment의 마지막 가시 unit 하나**다.
- 기존 120px 상한은 낮은 line에서는 과대 허용이고, 더 큰 line이나 visible object에서는
  source geometry와 무관한 별도 기준이었다.

## 구현

- `visible_tail_fits_before_spacer`를 두 RowBreak cut 경로의 단일 판정으로 만들었다.
- 현재 fragment가 이미 page budget을 넘지 않았고, 대상이 가시 unit이며, 구조적 spacer
  tail 계약을 만족할 때만 계속한다.
- overflow는 대상 unit의 실제 `height` 이내인지 판정한다. 따라서 한 fragment에서
  추가로 보존할 수 있는 범위는 실제 마지막 unit 하나로 제한된다.
- 중복된 고정 `120px` tail allowance를 제거했다.

## 검증

- 이번 Stage에서는 사용자 지시에 따라 build 또는 test를 실행하지 않았다.

## 결과

RowBreak tail 보존은 상수값이 아니라 cell unit의 실측 높이와 spacer 구조를 공통으로
사용한다. 단일 행과 rowspan block이 같은 조건으로 fragment 컷을 결정한다.
