# Task 1129 Stage 25 - 격자 점 1px 및 첫 그림 간격 보정

## 사용자 지적

- 격자 보기의 점 표시가 한컴오피스 대비 굵게 보이므로 `.` 표시를 1px 기준으로 수정한다.
- `hwp3-sample16-hwp5.hwp` 상단 첫 그림이 한컴오피스보다 외곽선에 더 붙어 보인다.

## 확인 계획

- `rhwp-studio/src/view/grid-overlay.ts`의 점 격자 radial-gradient 반경을 1px 지름 기준으로 조정한다.
- `samples/hwp3-sample16.hwp`, `samples/hwp3-sample16-hwp5.hwp`의 첫 그림 좌표와 SVG의 페이지 테두리 좌표를 계측한다.
- 차이가 그림 crop/원본 크기 해석인지, 쪽 테두리 기준 좌표 해석인지 분리한다.

## 계측 결과

- `target/debug/rhwp export-svg ... --show-grid=3mm` 기준, 수정 전 첫 그림은 `y=56.21px`, 쪽 테두리 안쪽 상단선은 `y=54.74px`라 간격이 `1.47px`뿐이었다.
- HWP3 원본과 HWP5 변환본의 첫 그림 저장 좌표는 모두 `용지 기준 x=11.64mm, y=14.87mm`로 동일했다.
- 원인은 그림 좌표가 아니라 쪽 기준 테두리 세로 계산이 `상단 여백 + 머리말 여백 - 테두리 간격`을 사용한 데 있었다.

## 수정 내용

- Studio 격자 점 표시를 1px 지름 기준으로 축소했다.
- 쪽 기준 페이지 테두리의 세로 기준을 `body_area`가 아니라 쪽 상/하단 여백 기준으로 보정했다.
- 동일 계산을 `get_page_info_native`의 `pageBorderTop/Bottom`에도 반영했다.

## 재계측

- 수정 후 첫 그림은 `y=56.21px`, 쪽 테두리 안쪽 상단선은 `y=16.93px`로 이동했고 간격은 `39.28px`가 됐다.
- `--show-grid=3mm`의 SVG pattern은 HWP3 원본/HWP5 변환본 모두 `11.3386px`로, 96DPI 기준 정확히 `3.00mm` 간격이다.

## 검증 계획

- 관련 Rust 단위 테스트
- `wasm-pack build --target web --out-dir pkg`
- `npm run build`
- `cargo fmt --all -- --check`
- `git diff --check`
