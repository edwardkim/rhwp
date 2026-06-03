# Task #1129 Stage 20 - 좌측 상단 그림 위치 미세 차이 분석

## 배경

작업지시자 시각 비교에서 `samples/hwp3-sample16-hwp5.hwp` 첫 페이지의 좌측 상단 로고 그림 위치가 한컴오피스와 rhwp-studio 사이에 미세하게 다르게 보인다.

한컴오피스 개체 속성 기준:

- 크기: 너비 `33.22mm`, 높이 `8.28mm`
- 가로: `종이`의 `왼쪽` 기준 `11.64mm`
- 세로: `종이`의 `위` 기준 `14.87mm`

## 확인 결과

`rhwp dump samples/hwp3-sample16-hwp5.hwp -s 0 -p 0` 기준 파싱값:

- 그림 common size: `9416 x 2348 HU` = `33.22mm x 8.28mm`
- 위치: `horizontal_offset=3300 HU` = `11.64mm`
- 위치: `vertical_offset=4216 HU` = `14.87mm`
- 기준: 가로 `Paper`, 세로 `Paper`
- crop: `(0, 0, 9960, 3000)`

최신 코드 기준 SVG 내보내기 좌표:

- 페이지 크기: `793.7067px x 1122.5067px`
- 로고 그림: `x=44.0px`, `y=56.2133px`, `w=125.5467px`, `h=31.3067px`
- 이는 각각 `11.64mm`, `14.87mm`, `33.22mm`, `8.28mm`와 일치한다.

쪽 기준 외곽선 좌표:

- top border line: `y=55.6433px`, `57.7433px`
- left border line: `x=36.71px`, `38.81px`
- 로고 top `56.2133px`는 이중선 사이에 걸친다.

## 현재 판단

객체 속성의 종이 기준 좌표 자체는 rhwp IR과 SVG 출력에서 일치한다. 따라서 이번 미세 차이는 로고 객체의 `horizontal_offset`/`vertical_offset` 파싱 오류라기보다 아래 중 하나일 가능성이 높다.

1. 쪽 기준 외곽선 위치와 로고의 상대 위치가 한컴오피스와 다르다.
2. 이중선 두께/중심선 처리 때문에 로고와 외곽선이 겹치는 시각 위치가 달라진다.
3. GIF 원본의 투명 여백 또는 crop 처리 방식이 한컴오피스와 다르다.
4. rhwp-studio Canvas/Web 렌더 경로가 SVG/native 좌표와 다르게 이미지 crop 또는 스케일을 적용한다.

## 다음 구현 후보

소스 수정 전에는 작업지시자 승인이 필요하다.

1. rhwp-studio WebCanvas 경로에서 첫 페이지 로고의 실제 `drawImage` 좌표와 crop source rect를 기록한다.
2. SVG/native 경로와 WebCanvas 경로의 좌표가 동일한지 테스트로 고정한다.
3. 좌표가 동일하면 쪽 기준 외곽선 배치 또는 이중선 stroke 배치를 보정한다.
4. 좌표가 다르면 `ImageNode.crop`/`compute_image_crop_src`/Canvas `draw_image_cropped` 경로를 보정한다.

## 검증 계획

- `samples/hwp3-sample16-hwp5.hwp` 첫 페이지 로고:
  - 객체 속성 mm 값 유지
  - SVG/native `ImageNode` 좌표 기록
  - WebCanvas/Playwright에서 이미지 DOM 또는 canvas replay 좌표 기록
- `wasm-pack build --target web --out-dir pkg`
- `npm run build`
- `cargo test --lib`
- `cargo fmt --all -- --check && git diff --check`
