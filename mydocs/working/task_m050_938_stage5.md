# Task M050 #938 Stage 5 완료보고서 — 정답지 톤 보정

## 단계 목표

작업지시자가 확인한 `samples/복학원서.pdf` 정답지와 `rhwp-studio` 화면의 남은 차이를 줄인다.

Stage 4의 투명 PNG 방식은 사각 테두리를 제거했지만, 정답지 PDF와 비교하면 다음 차이가 남았다.

- 정답지 PDF는 alpha/smask 없는 opaque JPEG 워터마크를 사용한다.
- rhwp는 투명 PNG에 CSS/SVG filter, opacity, multiply를 다시 적용해 가시 픽셀 수와 중간 톤이 달랐다.
- Studio overlay와 WASM Canvas 직접 렌더 경로가 같은 정책을 명시적으로 공유하지 않았다.

## 구현 내용

### 1. 워터마크 JPEG 선보정

`src/renderer/svg.rs`에 `watermark_jpeg_bytes_to_hancom_baked_png_bytes()`를 추가했다.

동작:

- JPEG를 RGBA로 디코딩한다.
- 외곽 근백색 비율과 전체 근백색 비율이 충분한 이미지에만 적용한다.
- 근백색 배경은 opaque white로 고정한다.
- 비배경 픽셀은 복학원서 정답 PDF에서 추출한 워터마크 픽셀 분포에 맞춘 piecewise gray mapping으로 변환한다.
- 결과를 `image/png`로 재인코딩한다.

### 2. 중복 필터 생략

baked 워터마크에는 후단 효과를 중복 적용하지 않도록 변경했다.

- SVG: baked PNG 성공 시 `grayscale`, `brightness/contrast`, `opacity=0.17` 래퍼 생략
- Studio overlay JSON: `bakedWatermark: true` 추가
- Studio DOM overlay: `bakedWatermark`가 true이면 CSS filter, `mix-blend-mode`, `opacity` 생략
- WebCanvasRenderer: baked PNG 성공 시 Canvas filter와 global alpha 생략

기존 워터마크 JPEG 선보정이 실패하거나 다른 이미지 효과인 경우에는 기존 fallback 정책을 유지한다.

### 3. 회귀 테스트 갱신

`tests/issue_938.rs`는 더 이상 투명 alpha를 기대하지 않고, 정답 PDF에 가까운 opaque PNG 톤을 검증한다.

검증 항목:

- SVG 워터마크가 `image/png`로 emit된다.
- SVG에 `opacity="0.17"`이 중복 적용되지 않는다.
- overlay JSON에 `bakedWatermark: true`가 포함된다.
- baked PNG는 alpha min/max가 모두 255다.
- 평균 회색값, 가시 픽셀 수, 가시 p10/p50 톤이 정답 PDF 분석 범위 안에 있다.

## 검증 결과

```text
cargo test --release --test issue_938
결과: 성공, 2 passed

cargo test --release --test issue_514
결과: 성공, 3 passed

cargo test --release --test issue_516
결과: 성공, 8 passed

UPDATE_GOLDEN=1 cargo test --release --test svg_snapshot
결과: 성공, 8 passed

cargo test --release --test svg_snapshot
결과: 성공, 8 passed

cargo check --target wasm32-unknown-unknown --release --lib
결과: 성공

docker-compose --env-file .env.docker run --rm wasm
결과: 성공

npm run build
결과: 성공
```

WASM 직접 확인:

```text
getPageOverlayImages(0) watermark:
  mime = image/png
  bakedWatermark = true
  effect = grayScale
  brightness = -50
  contrast = 70
```

## 서버

최신 WASM 빌드 후 `rhwp-studio` 개발 서버를 재시작했다.

```text
url: http://127.0.0.1:7700/
```

## 남은 확인 사항

정답지 PDF와 비교할 때 워터마크 크기는 거의 같지만, 이전 분석 기준으로 위치가 약 8.3pt, 즉 문서 좌표 약 11px 정도 좌상단에 치우친 차이가 남아 있다.

이번 단계에서는 이 값을 하드코딩하지 않았다. 위치 보정은 이미지 anchor/page origin 계산을 별도로 확인한 뒤 적용해야 다른 문서의 BehindText 이미지 배치 회귀를 피할 수 있다.

## 작업 중 주의 사항

`cargo fmt`를 저장소 전체에 실행해 범위 밖 파일의 포맷 변경이 생겼다. 이 변경은 #938 구현 범위가 아니므로 되돌려야 한다.

작업지시자 승인을 받아 정리했다.

```text
#938 변경 파일만 유지:
Cargo.toml
rhwp-studio/src/view/page-renderer.ts
src/document_core/queries/rendering.rs
src/renderer/svg.rs
src/renderer/web_canvas.rs
tests/golden_svg/issue-677/bokhakwonseo-page1.svg
tests/issue_514.rs
tests/issue_938.rs
mydocs/orders/20260517.md
mydocs/plans/task_m050_938.md
mydocs/plans/task_m050_938_impl.md
mydocs/working/task_m050_938_stage1.md
mydocs/working/task_m050_938_stage3.md
mydocs/working/task_m050_938_stage4.md
mydocs/working/task_m050_938_stage5.md

나머지 tracked 포맷 변경은 복원 완료
tests/golden_svg/issue-677/bokhakwonseo-page1.actual.svg 는 테스트 부산물이므로 제거
```

정리 후 tracked diff는 #938 관련 7개 파일만 남는다.
