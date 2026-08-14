# Stage 139: HWPX Q26 이후 잔여 page owner 분석

## 목적

Stage 138에서 Q26을 p294에 완결하고 Q27을 p295로 되돌렸지만, 이후 Q&A 표에서 남는
page owner 차이를 원인별로 분리한다. 전체 페이지 수 383과 12절 p367 시작은 이미 맞추고
있으므로, 이 Stage에서는 그 전역 계약을 깨지 않는 최소 원인만 찾는다.

## 출발점

- 기준 PDF, native HWP, HWPX의 총 페이지 수는 모두 383이다.
- Q26은 p294에서 끝나고 Q27은 p295에서 시작한다.
- Stage 138의 owner map에서 다음 후보는 pi56 이후다. 표의 저장 line segment, browser 조판
  line 수, 실제 fragment cut과 다음 표제의 쪽 소유권을 함께 비교한다.
- 11절 103x2 병렬 규정 표는 앞 구간의 잔여 차이를 상쇄하는 위치이므로, 먼저 원인으로 가정하지
  않는다.

## 분석 절차

1. native HWP와 HWPX의 pi56 이후 최초 시작 페이지와 해당 표 fragment를 덤프한다.
2. PDF와 native/HWPX p295 이후 대응 페이지의 시각 증적으로 owner 차이를 확인한다.
3. HWPX 전용으로 재현되는 정확한 table/paragraph 계약이 확인될 때만 조판 보정을 구현한다.
4. 383페이지와 12절 p367 시작, 저장 HWP 재로드 tree를 회귀 테스트로 유지한다.

## 분석 결과

pi56은 Q29 표였다. Q27과 같은 6x5, 15-cell, 높이 15224, `RowBreak`, outer bottom 566
계약이지만, Q27의 3줄 응답과 달리 Q29의 마지막 response cell은 2줄 단일 문단이다. HWPX는
p295에서 rows `0..4`만 그리고 rows `4..6`을 p296에 남겼다. native HWP/PDF는 Q29 전체를
p295에 유지하며 body 하한을 약 23.5px 넘긴다.

`advance_row_cut` 진단에서 Q29의 raw budget은 50.0px, 32px 논리 allowance 뒤 content cut은
완결됐지만 row 전체는 44.6px 더 필요했다. 따라서 48px이 이 저장 topology를 완결하는 최소
8px 단위 allowance다.

Q29을 p295에 수용하면 HWPX는 일시적으로 382페이지가 됐다. 11절 103x2 병렬 규정 표의
reserve를 42px에서 56px으로 옮기자 53번째 fragment가 생겨 이 한 페이지를 보상했다. 이는
Stage 138에서 관측한 14px stored-cut 간격의 다음 임계값이다.

## 구현

- HWPX 전용 Q29 predicate를 추가했다: 6x5/15-cell, 높이 15224, outer bottom 566, 마지막
  response 행, 2줄 단일 문단, trailing blank-bottom row 조합이다.
- 이 topology에만 48px logical/physical tail allowance를 적용해 Q29과 blank-bottom row의
  page owner를 native HWP/PDF와 같이 p295로 고정했다.
- 병렬 규정 표 reserve는 56px으로 조정했다. 일반 표 또는 일반 2줄 문단에는 영향을 주지 않는다.

## 결과

- Q29은 PDF/native/HWPX 모두 p295에 완결된다.
- Q30은 PDF/native/HWPX 모두 p296에서 시작한다.
- HWPX, native HWP, 기준 PDF의 총 페이지 수는 모두 383이다.
- 병렬 규정 표는 53 fragment이며, 12절은 p367에서 시작한다.
- p295-p296 시각 비교 증적은
  `/tmp/rhwp-3820-stage139-q29-visual-1/p295-comparison.png` 및
  `/tmp/rhwp-3820-stage139-q29-visual-1/p296-comparison.png`에 남겼다. Q29/Q30의 쪽
  소유권은 일치하며, 글꼴 raster 차이는 별도 fidelity 잔여 항목이다.

## 회귀 고정

`tests/issue_3930_hwpx_hwp_save_layout.rs`는 원본 HWPX와 저장 HWP의 p295/p296 render tree를
함께 비교하고 Q29 p295 완결, Q30 p296 시작, 전체 383페이지를 고정한다.

실행 명령:

```bash
CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test \
  --test issue_3930_hwpx_hwp_save_layout --quiet
```

## 상태

구현 및 회귀 검증 대기. 이 Stage는 로컬 커밋으로만 보존하며, 새 릴리스 준비 중에는 merge,
push, PR 생성을 하지 않는다.
