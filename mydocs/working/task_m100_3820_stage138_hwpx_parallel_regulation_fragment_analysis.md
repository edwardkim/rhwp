# Stage 138: HWPX 병렬 규정 표 fragment와 Q26 쪽 소유권 보정

## 목적

2025 행정업무운영 편람 HWPX 출력에서 native HWP/PDF와 다른 쪽 소유권의 다음 최초 원인을
확인하고, HWPX와 저장 HWP가 모두 383페이지를 유지하도록 고정한다.

기준 PDF는 이미 확보된
`pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf`이며, 재생성하지 않았다.

## 분석

Stage 137 뒤 HWPX와 native HWP의 최종 페이지 수는 383으로 같았지만, Q16 이후의 누적
소유권 지도를 비교하면 다음 최초 차이가 남았다.

| 문단 항목 | native 시작 | HWPX 시작 | 누적 차이 |
| --- | ---: | ---: | ---: |
| pi53 (Q26) | p294 | p294 | 0 |
| pi54 (Q27) | p295 | p296 | +1 |
| pi57 | p295 | p297 | +2 |
| pi69 | p298 | p301 | +3 |
| pi75 | p299 | p303 | +4 |
| pi99 | p305 | p310 | +5 |

문제의 pi53은 HWPX와 native 모두 `RowBreak`, 6x5, 15-cell, 높이 19355, outer bottom
566인 같은 Q26 표이다. 응답 행에는 저장된 3줄 문단 두 개가 있으며 HWPX browser metrics에서는
두 번째 문단의 tail을 p294에 수용할 수 있는데도 보수적 cut이 발생했다. 그 결과 HWPX만
71.4px tail을 p295로 넘기고 Q27을 p296에서 시작했다.

한편 11절의 103x2 병렬 규정 표는 최초 원인이 아니라 앞서 누적된 +5페이지를 최종 383페이지로
상쇄하던 위치였다. Q26을 p294에 수용하면 이 상쇄 표는 fragment 하나를 더 생성해야 한다.
Stage 137의 reserve 8px에서 HWPX는 51 fragment였고, Q26 보정 뒤 reserve 28px에서는 첫
fragment의 end cut이 `[9, 1]`에서 `[8, 1]`로 이동했지만 총 수는 여전히 51이었다. 다음
stored unit의 실제 slack은 12.2px이어서, 42px reserve가 다음 cut 임계값이다.

## 구현

`src/renderer/typeset.rs`에 다음 HWPX 전용 계약을 추가했다.

- Q26의 정확한 표 형태와 두 개의 3줄 문단 tail을 식별한다.
- physical tail allowance 64px 안에서 마지막 continuation을 현재 fragment에 수용한다.
- 103x2, 11절 병렬 규정 표는 42px reserve를 사용해 다음 stored cut 단위를 다음 fragment로
  넘긴다. 이 값은 총 페이지 수를 맞추기 위한 임의 상수가 아니라, 위 cut slack 측정으로 정한
  최소 다음 임계값이다.

범위는 HWPX adapter가 만드는 동일한 표 계약으로 한정한다. 일반 HWP, 모든 6x5 표, 모든
103x2 표에 적용하지 않는다.

## 결과

- HWPX Q26은 p294에서 끝나고 Q27은 p295에서 시작한다.
- native HWP, HWPX, 기준 PDF의 전체 페이지 수는 모두 383이다.
- 11절 병렬 규정 표 HWPX fragment 수는 51에서 52가 되었고, 12절 시작은 native/PDF와 같은
  p367이다.
- 첫 병렬 규정 fragment의 HWPX end cut은 `[7, 1]`이며, native의 `[6, 1]`과 한 stored unit
  차이다. 이는 앞 구간의 잔여 조판 차이를 나타내므로 이후 Stage에서 별도로 다룬다.
- p294와 p295의 PDF/native/HWPX 비교 이미지는
  `/tmp/rhwp-3820-stage138-q26-visual-1/p294-comparison.png` 및
  `/tmp/rhwp-3820-stage138-q26-visual-1/p295-comparison.png`에 남겼다. 두 페이지에서 Q26/Q27
  소유권은 일치한다. 글꼴 raster 차이는 별도의 fidelity 잔여 항목이다.

## 회귀 고정

`tests/issue_3930_hwpx_hwp_save_layout.rs`는 원본 HWPX와 저장 HWP에 대하여 다음을 함께
확인한다.

- Q26의 3+3줄 tail 뒤 Q27 표제가 p294가 아니라 p295에 있어야 한다.
- p294와 p295의 저장 전후 render tree가 동일해야 한다.
- 전체 페이지 수가 383이어야 한다.

실행 명령:

```bash
CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test \
  --test issue_3930_hwpx_hwp_save_layout --quiet
```

## 잔여 범위

이번 Stage는 Q26 이후 첫 page owner 차이와 그 보상 fragment를 고정한다. PDF 대비 글꼴
metric, 이후 표 fragment cut, 시각적 fidelity의 잔여 차이는 계속 분석한다. 따라서 #3820은
닫지 않는다.
