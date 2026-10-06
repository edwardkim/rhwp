# Stage 168: saved object frame fit의 좌표 비교 계약 분석

## 목적

saved object bottom fit, saved anchor split, declared float defer에 남은 `16px`와 `1px`
비교를 폐기하고, stored anchor·declared object bottom·current flow body의 직접 좌표 관계로
판정한다.

## 분석 범위

- `DECLARED_FLOAT_FIT_TOLERANCE_PX`
- `saved_object_bottom_fits_current`
- `saved_anchor_splits_here`
- saved anchor와 `current_height`, `available`의 base/offset 관계

## 원칙

- 동일 HWP 좌표계의 stored object frame 비교에 임의 px slack을 더하지 않는다.
- source anchor와 current flow가 다른 경우, 그 차이는 measured growth 또는 explicit frame
  boundary로 설명돼야 한다.
- object bottom이 body를 넘는 source는 whole-fit/defer 예외가 아니라 RowBreak cut을 따른다.

## 완료 기준

- anchor top/bottom 및 current body의 좌표계를 직접 비교할 수 있는지 확인한다.
- fixed tolerance를 source/flow predicate로 대체할 수 있을 때만 코드와 결과를 같은 Stage
  커밋에 남긴다.

## 구현

- `DECLARED_FLOAT_FIT_TOLERANCE_PX`를 제거했다.
- `measured_declared_excess = (table_total - declared_total).max(0.0)`를 도입해
  saved anchor 지연량과 비교한다.
- `saved_object_bottom_fits_current`:
  - `top_px <= current_height`
  - `anchor_delay = (current_height - top_px).max(0.0) <= measured_declared_excess`
  - `bottom_px <= available`
- `saved_anchor_splits_here`:
  - `top_px <= current_height`
  - `anchor_delay <= measured_declared_excess`
  - `bottom_px > available`
- 1px 완급치 비교를 쓰는 `native_hwp5_own_footnote_fragment_can_start_before_reservation`
  조건도 정확 비교로 정리한다.

## 결과

- 고정 px 슬랙 없이 stored object bottom/anchor 가드를 source 좌표 계약으로 정렬했다.
- 전체 테스트 미실행 상태에서 해당 블록의 정밀 임계치 의존을 제거해 다음 단계에서
  Stage 168 결과를 바탕으로 #3820 잔여 fidelity 후보를 계속 처리할 준비가 됐다.

## 상태

Stage 168 분석+코드 반영 완료(커밋 대상 준비).
