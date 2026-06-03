# Task #1129 Stage 12 - 한컴 기준 격자 표시 차이 4종 보정

- 이슈: [#1129](https://github.com/edwardkim/rhwp/issues/1129)
- 브랜치: `local/task_m100_1129`
- 일자: 2026-05-26

## 배경

사용자가 첨부한 비교 화면의 순서는 다음과 같다.

1. rhwp-studio
2. 한컴오피스

Stage 11 이후에도 다음 네 가지 차이가 남아 있다.

1. rhwp-studio 격자가 하단 쪽 테두리 아래로 과하게 이어져 보인다.
2. rhwp-studio 쪽 테두리 위치가 한컴오피스 기준과 다르게 보인다.
3. rhwp-studio 외곽선/그림자가 한컴오피스보다 넓고 두껍게 보인다.
4. rhwp-studio 점 격자가 한컴오피스보다 연하고 커 보인다.

## 분석

현재 `쪽` 기준 격자는 `pageBorder*` 값을 사용해 시작 좌표와 clip을 모두 계산한다. 이 값 자체는 3mm 격자 점 개수를 한컴 쪽 기준에 가깝게 만든다.

다만 하단은 배경 패턴 반복 특성상 마지막 점 줄이 쪽 테두리 아래쪽에 걸쳐 보일 수 있다. 특히 `radial-gradient` 점 반경과 clip 경계가 맞물리면 하단에서 점이 더 내려온 것처럼 보인다.

페이지 외곽은 canvas 자체의 렌더된 선과 CSS `box-shadow`가 함께 보이면서 한컴보다 두껍게 보인다.

점 색상은 한컴 기준보다 밝고 넓게 퍼져 보인다.

## 수정 방향

- `쪽` 격자의 좌/상/우 기준은 `pageBorder*`를 유지한다.
- `쪽` 격자의 하단 clip은 점 반경과 한 칸 미만의 여유를 제거해 테두리 안에서 끝나도록 더 엄격히 보정한다.
- 점 격자는 더 작고 진하게 조정한다.
- 페이지 canvas 외곽 shadow는 제거해 한컴처럼 얇은 외곽선만 남긴다.

## 수정 내용

- `rhwp-studio/src/view/grid-overlay.ts`
  - 점 색상을 `rgba(0, 22, 135, 0.95)`로 더 진하게 조정했다.
  - 점 반경을 `0.55px`에서 `0.42px`로 줄였다.
  - `쪽` 기준 하단 clip은 `pageBorderBottom`과 `marginBottom` 중 큰 값을 사용하도록 조정했다.
- `rhwp-studio/src/styles/editor.css`
  - canvas `box-shadow`를 제거했다.

## 검증 계획

- 로컬 Playwright 기능 검증
  - `samples/hwp3-sample16-hwp5.hwp` 로드
  - HWPX 비표준 감지 모달에서 `그대로 보기`
  - `쪽/3mm/0,0` 설정
  - overlay `clip-path`, `background-position`, `background-size`, `background-image`, `opacity` 기록
  - canvas `box-shadow` 기록
  - 쪽 테두리 기준 예상 점 개수 기록
- `npm run build`
- `wasm-pack build --target web --out-dir pkg`
- `cargo fmt --all -- --check && git diff --check`

## 검증 결과

- 로컬 Playwright 기능 검증 통과
  - 샘플: `samples/hwp3-sample16-hwp5.hwp`
  - 모달: `그대로 보기`
  - 설정: `쪽/3mm/0,0`
  - overlay 수: `2`
  - `background-image`: `radial-gradient(circle, rgba(0, 22, 135, 0.95) 0px, rgba(0, 22, 135, 0.95) 0.42px, rgba(0, 0, 0, 0) 0.6px)`
  - `background-size`: `12.5714px 12.5714px`
  - `background-position`: `20.955px 20.955px`
  - `clip-path`: `inset(21.955px 21.955px 42.91px)`
  - `opacity`: `1`
  - canvas `box-shadow`: `none`
  - 보정 후 쪽 격자 영역: `199.99854166666668mm x 281.99291666666664mm`
  - 예상 점 개수: `67 x 94`
- `npm run build` 통과
  - 기존 큰 chunk 경고만 표시됨
- `wasm-pack build --target web --out-dir pkg` 통과
  - 추적 산출물 변경 없음
- `cargo fmt --all -- --check && git diff --check` 통과

## 대기

자동 검증 후 커밋하고, 최종 정합 여부는 작업지시자의 수동 비교를 기다린다.
