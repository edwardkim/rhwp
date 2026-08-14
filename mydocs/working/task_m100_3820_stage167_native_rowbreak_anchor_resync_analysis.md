# Stage 167: native HWP5 RowBreak anchor resync의 source 계약 분석

## 목적

native HWP5 RowBreak 표의 near-anchor fragment scan 및 internal reset resync에 남은
`24px`/`16px` tolerance를 제거하고, stored object anchor·measured excess·cell frame reset의
실제 관계로 resync 여부를 판정한다.

## 분석 범위

- `NATIVE_HWP5_NEAR_ANCHOR_ROWBREAK_FRAGMENT_TOLERANCE_PX`
- `NATIVE_HWP5_INTERNAL_RESET_REWIND_RESYNC_TOLERANCE_PX`
- `saved_span`, current flow anchor, declared object bottom
- stored cell `lineSeg` reset과 measured/declared table height 차이

## 원칙

- anchor와 current flow 사이의 임의 px 간격으로 fragment scan을 시작하지 않는다.
- reset과 source object bottom이 명시한 frame 소유권이 있을 때만 current flow를 되감는다.
- source anchor가 불완전하면 일반 RowBreak cut을 우선한다.

## 완료 기준

- near-anchor와 internal-reset resync의 source 좌표계를 분리해 판정한다.
- fixed px tolerance를 공통 object/frame predicate로 대체할 수 있을 때만 코드와 결과를
  같은 Stage 커밋에 남긴다.

## 분석 결과

- near-anchor `24px`와 internal-reset resync `16px`는 모두 current flow가 saved object
  anchor보다 늦어진 양을 허용하기 위한 값이었다. 이 차이는 실제로 앞선 표의
  `table_total - declared_total` 팽창에서 발생하지만, 고정값은 다른 표의 독립적인
  source frame까지 fragment scan/resync로 승격할 수 있다.
- 두 경로는 이미 native HWP5, non-TAC, paragraph TopAndBottom, RowBreak, source rewind와
  object bottom/body 관계를 함께 요구한다. 남은 공통 판별은 flow anchor의 지연이 실제
  measured-declared excess로 완전히 설명되는지다.
- saved object bottom과 `available`은 동일 HWP 좌표계에서 변환되므로, 이 source bottom
  판정에 별도 px slack을 둘 근거가 없다.

## 구현

- `NATIVE_HWP5_NEAR_ANCHOR_ROWBREAK_FRAGMENT_TOLERANCE_PX=24`와
  `NATIVE_HWP5_INTERNAL_RESET_REWIND_RESYNC_TOLERANCE_PX=16`을 제거했다.
- near-anchor fragment scan과 internal-reset resync는 모두 다음을 요구한다.
  - saved anchor가 current flow보다 앞 또는 같음
  - anchor 지연량이 actual measured-declared excess 이하
  - saved object bottom이 현재 body bottom 안에 있음
- internal-reset 경로의 reset, next source rewind, footnote 등 기존 source 계약은 유지한다.

## 결과

- native HWP5 anchor resync가 fixture 크기의 px tolerance가 아니라 source anchor와
  measured growth의 회계 관계로 결정된다.
- 전체 export 및 test는 이 Stage에서 실행하지 않았다. 다음 Stage에서 saved object bottom
  fit 및 anchor split에 남은 `16px`/`1px` 비교를 같은 source geometry로 분석한다.
