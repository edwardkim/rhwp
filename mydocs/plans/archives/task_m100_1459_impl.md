# Task M100 #1459 구현계획서

## 1. 분석 순서

1. `rhwp-studio/src/ui/picture-props-dialog.ts`의 속성 적용 경로를 확인한다.
2. WASM API의 `setPictureProperties`와 Rust `object_ops` 갱신 범위를 확인한다.
3. `renderer`의 inline picture 위치 계산과 비-TAC `TopAndBottom` flow reservation 계산을 추적한다.
4. 샘플에서 두 번째 그림 속성을 변경했을 때 문단 레이아웃 입력값이 바뀌는지 확인한다.

## 2. 구현 방향

- 모델 속성 변경 자체가 부족하면 `object_ops`에서 공통 속성 동기화를 보정한다.
- 모델은 바뀌지만 배치가 stale이면 레이아웃 계산 입력 또는 캐시 무효화 경로를 보정한다.
- 비-TAC `TopAndBottom` 그림의 예약 공간은 같은 문단의 TAC 그림 위치 계산보다 먼저 또는 같은 레이아웃 패스에서
  일관되게 반영되도록 한다.
- 임의 보정보다는 그림 공통 속성의 실제 크기, 외부 여백, 기준 offset을 사용해 계산한다.

## 3. 테스트 방향

- 샘플 파일을 fixture로 추가한다.
- 그림 속성 변경 전후 render tree 또는 cursor/page layout 결과를 비교하는 focused 테스트를 추가한다.
- 테스트는 첫 번째 TAC 그림과 두 번째 자리차지 그림이 같은 문단 내에서 함께 재배치되는 조건을 검증한다.

## 4. 위험 관리

- 기존 `TopAndBottom` 표/그림 흐름 회귀 위험이 있으므로 기존 관련 테스트를 함께 실행한다.
- `treat_as_char=true` 개체의 일반 inline 배치를 깨지 않도록 그림 혼합 문단 케이스에 국소화한다.
