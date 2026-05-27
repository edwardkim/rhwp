# Task M100 #1139 Stage 11

## 목적

`3-09월_교육_통합_2022.hwp` 9쪽을 한컴오피스 기준으로 다시 비교한다.

작업지시자의 추가 판정:

- rhwp-studio 격자 설정의 종이 기준 세로 값이 한컴오피스와 다르다.
- 한컴오피스 정답은 `가로 9.00mm`, `세로 24.00mm`다.
- rhwp-studio는 `세로 24.02mm`로 표시되므로 `24.00mm`로 보정한다.
- 격자 기준을 맞춘 뒤에도 9쪽 전체 레이아웃이 한컴오피스와 다르다.
- 비교 기준은 작업지시자 스크린샷 기준으로 세 번째가 rhwp-studio, 네 번째가 한컴오피스다.

## 분석 계획

1. rhwp-studio 격자 기준 위치 기본값 계산 경로를 확인하고 `24.00mm`로 표시되도록 보정한다.
2. `target/debug/rhwp export-svg --show-grid=3mm`와 `dump-pages -p 8`을 다시 생성해 9쪽 기준 좌표를 확인한다.
3. 9쪽 차이를 다음 항목으로 나누어 본다.
   - 페이지/테두리/격자 기준 위치 차이
   - 본문 시작 위치와 머리말 위치 차이
   - 미주 시작 구분선 위치 차이
   - 문5) 이후 미주 단 흐름 차이
4. 페이지 수 23쪽을 깨지 않는 범위에서 원인을 좁힌다.

## 검증 예정

- `npm run build` (`rhwp-studio`)
- `npm test` (`rhwp-studio`)
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `wasm-pack build --target web --out-dir pkg`
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 8 --show-grid=3mm`

## 구현 결과

- rhwp-studio 격자 기준 위치 기본값을 `PageInfo` 픽셀값 재환산이 아니라 HWP 원본 `PageDef` HWPUNIT 기준으로 계산하도록 수정했다.
  - 대상 문서의 `marginTop=1984`, `marginHeader=4819`는 각각 7.00mm, 17.00mm로 표시되며 합산값은 24.00mm다.
  - 종이 기준 기본값은 `가로 9.00mm`, `세로 24.00mm`가 된다.
- 미주 조판에서 같은 미주 내부 paragraph의 `vpos`가 되감기고 현재 단이 하단에 가까우면 다음 단으로 넘기도록 보정했다.
  - 9쪽 `문5)` 뒤 풀이가 왼쪽 단 하단에 과도하게 남지 않고 오른쪽 단으로 이동한다.
- 후반 미주에서 페이지 수가 24쪽으로 늘어나는 부작용을 막기 위해, 단 상단 근처의 큰 `vpos` 점프와 내부 `vpos` 되감기 paragraph는 줄높이 합계보다 lineSeg 위치 span을 우선하도록 좁게 보정했다.
  - 전체 페이지 수는 한컴 기준 23쪽을 유지한다.
- 추가 시각 판정에서 `문5)`가 한컴보다 아직 위에 있어, 단 하단에서 다음 단으로 이어지는 `vpos` 되감김 미주 묶음이 시작될 때만 한컴 미주 설정의 `미주 사이 7mm`를 반영하도록 보정했다.
  - 일반 미주 전체에 `raw_unknown`을 일괄 가산하지 않아 23쪽 유지 조건을 깨지 않는다.

## 검증 결과

- `cargo build`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 통과, 5개 테스트 통과
- `npm run build` (`rhwp-studio`): 통과
- `npm test` (`rhwp-studio`): 통과, 38개 테스트 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
- `target/debug/rhwp info samples/3-09월_교육_통합_2022.hwp`: 23페이지
- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 8`: 9쪽 `문5)` 후속 풀이가 오른쪽 단에서 시작하며, 왼쪽 단 `문5)` 시작 위치가 하단으로 보정됨
- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 22`: 최종 23쪽에 문30) 후반까지 배치됨
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -o output/task1139_stage11b_svg -p 8 --show-grid=3mm`: 성공

## 산출물

- `output/task1139_stage11b_svg/3-09월_교육_통합_2022_009.svg`
- `output/task1139_stage11b_svg/3-09월_교육_통합_2022_009_558.png`
