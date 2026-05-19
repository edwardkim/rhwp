# Task M050 #938 Stage 4 완료보고서 — 테스트 및 시각 검증 산출물

## 단계 목표

Stage 3 구현이 `복학원서.hwp` 중앙 워터마크 사각 배경 문제를 해결하는지 테스트와 산출물로 검증한다.

## 수행 항목

- #938 전용 회귀 테스트 실행
- 관련 기존 회귀 테스트 실행
- `svg_snapshot` golden 갱신 및 재검증
- WASM Canvas 경로 컴파일 확인
- release CLI 재빌드 후 SVG 산출물 생성
- 사용자가 직접 확인할 수 있는 비교용 HTML 작성

## 테스트 결과

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

cargo build --release
결과: 성공
```

참고: 최초 WASM check는 로컬 Rust toolchain에 `wasm32-unknown-unknown` target이 없어 실패했다. target 설치 후 재실행하여 성공했다.

## Studio 재검토 결과

작업지시자 시각 확인에서 `rhwp-studio` 화면에 사각 배경이 여전히 남는 문제가 확인되었다.

원인은 이전 서버 실행이 아니라 **native 빌드와 WASM 빌드의 `image` crate feature 차이**였다.

기존 `Cargo.toml`:

```text
image features = ["bmp", "png"]
```

native 테스트에서는 `svg2pdf`의 native-only dependency가 `image/jpeg` feature를 함께 켜서 JPEG decode가 가능했다. 반면 `wasm32-unknown-unknown` target에서는 `svg2pdf`가 제외되므로 `image/jpeg` feature가 빠지고, `watermark_jpeg_bytes_to_transparent_png_bytes()`가 JPEG decode 실패로 `None`을 반환했다.

수정:

```text
image features = ["bmp", "jpeg", "png"]
```

수정 후 확인:

```text
cargo tree --target wasm32-unknown-unknown -e features -i image
결과: image feature "jpeg" 포함 확인

Node에서 최신 pkg/rhwp_bg.wasm 직접 호출:
  getPageOverlayImages(0) behind[1].mime = image/png

Vite 제공 WASM hash:
  b565cc15eea88a8426f9285b1b7a6eabb8a41f6e7a48f9582b14f218f6454a9f
```

## 기존 테스트 갱신

`tests/issue_514.rs`의 `issue_514_jpeg_watermark_unchanged` 기대값은 #938 구현과 충돌했다.

기존 의미:

```text
복학원서 워터마크 JPEG는 변경 없이 emit 되어야 함
```

갱신 의미:

```text
#514의 핵심은 PCX가 octet-stream으로 떨어지지 않는 것
#938 이후 복학원서 워터마크 JPEG는 transparent PNG로 emit 되어야 함
```

따라서 테스트 이름과 assertion을 새 의도에 맞게 갱신했다.

## 스냅샷 갱신

`tests/golden_svg/issue-677/bokhakwonseo-page1.svg`를 갱신했다.

diff의 실질 변경은 중앙 워터마크 image data URI가 다음처럼 바뀐 것이다.

```text
기존: data:image/jpeg;base64,...
변경: data:image/png;base64,...
```

좌표, 크기, filter, opacity 구조는 유지된다.

## 시각 검증 산출물

생성 파일:

```text
output/debug/task938/stage4/복학원서.svg
output/debug/task938/stage4/index.html
```

비교 기준 파일:

```text
output/debug/task938/stage1/복학원서.svg
output/debug/task938/stage1/pdf_2022_page1_2x.png
```

확인 결과:

```text
output/debug/task938/stage4/복학원서.svg image MIME:
  data:image/png
  data:image/png
```

즉 1페이지의 학교 로고 PCX와 중앙 워터마크 JPEG가 모두 브라우저 렌더 가능한 PNG로 출력된다. 중앙 워터마크 PNG는 #938 테스트에서 alpha min 0, alpha max 255, 투명 픽셀 수 100,000 초과를 검증했다.

## 로컬 서버

최초에는 `output/debug/task938` 정적 서버를 실행했으나, 작업지시자 요청에 따라 실제 `rhwp-studio` 개발 서버로 전환했다.

```text
server: rhwp-studio Vite dev server
url:    http://127.0.0.1:7700/
```

`rhwp-studio`에서 #938 Rust 변경이 반영되도록 Docker 기반 WASM 빌드를 먼저 실행해 `pkg/`를 최신화했다.

```text
docker-compose --env-file .env.docker run --rm wasm
결과: 성공

npm run dev -- --host 127.0.0.1 --port 7700
결과: 성공, HTTP 200 응답 확인
```

feature 수정 후에는 Vite 서버를 `--force`로 재시작했다.

```text
npm run dev -- --host 127.0.0.1 --port 7700 --force
결과: 성공
```

## PR #939 반영 후 재검증

작업지시자 요청에 따라 [edwardkim/rhwp#939](https://github.com/edwardkim/rhwp/pull/939)의 head를 `local/task938`에 fast-forward로 반영했다.

```text
base before: b8710d92
PR #939 head: a8d80fed
merge mode: fast-forward
```

#939가 변경한 주요 파일:

```text
rhwp-studio/src/view/canvas-view.ts
rhwp-studio/src/view/page-renderer.ts
src/renderer/web_canvas.rs
```

#938과 직접 겹친 파일은 `src/renderer/web_canvas.rs`였고, stash 재적용 시 자동 병합되었다. 최종 diff 기준으로 #939 위에 #938의 변경은 다음 범위로 유지된다.

```text
Cargo.toml
src/document_core/queries/rendering.rs
src/renderer/svg.rs
src/renderer/web_canvas.rs
tests/golden_svg/issue-677/bokhakwonseo-page1.svg
tests/issue_514.rs
tests/issue_938.rs
```

재검증 결과:

```text
cargo test --release --test issue_938
결과: 성공, 2 passed

cargo test --release --test issue_514
결과: 성공, 3 passed

cargo test --release --test issue_516
결과: 성공, 8 passed

cargo test --release --test svg_snapshot
결과: 성공, 8 passed

cargo check --target wasm32-unknown-unknown --release --lib
결과: 성공

docker-compose --env-file .env.docker run --rm wasm
결과: 성공

Node에서 최신 pkg/rhwp_bg.wasm 직접 호출:
  getPageOverlayImages(0) behind[1].mime = image/png

npm run build
결과: 성공

npm run dev -- --host 127.0.0.1 --port 7700 --force
결과: 성공, HTTP 200 응답 확인

Vite 제공 WASM hash:
  62a61f337b715ed06cad049815d1904e5e129f656dc91d00ac17803eb6462d28
```

## 남은 단계

작업지시자 시각 확인 후 Stage 5로 진행한다.

- 필요 시 threshold/soft alpha ramp 보정
- 최종 보고서 작성
- `mydocs/orders/20260517.md` 상태 갱신
