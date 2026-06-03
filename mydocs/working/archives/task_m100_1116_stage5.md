# Task #1116 Stage 5 보고서 — Web Canvas 영문 겹침 보정

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: localhost rhwp-studio p3 본문 영문 겹침 보정 및 검증 완료

## 1. 배경

작업지시자 캡처의 핵심 차이 중 하나는 p3 본문 혼합 한/영 문장에서 영문 토큰이 서로 겹쳐 보이는 점이었다.

대표 구간:

- `통합모델(Consolidation)`
- `고가용성(High Availability)`
- `BCP:Business Continuity Planning`

원인:

- layout 좌표는 한컴/HWP 메트릭 기준의 cluster advance를 사용한다.
- Web Canvas 실제 렌더링은 브라우저가 로드한 대체 웹폰트 glyph 폭으로 그린다.
- 실제 glyph 폭이 layout advance보다 큰 경우, cluster별 `fillText`가 다음 cluster 영역을 침범한다.

## 2. 구현

수정 위치:

```text
src/renderer/web_canvas.rs
```

기존:

- cluster 시작 x는 `compute_char_positions()` 결과를 따른다.
- 실제 Canvas glyph 폭이 advance보다 커도 그대로 그린다.

변경:

- 각 cluster의 layout advance를 계산한다.
- `CanvasRenderingContext2d.measureText(cluster)` 실제 폭이 `advance`보다 크면, 해당 cluster만 가로 스케일을 줄여 advance 안에 들어가게 그린다.
- 좌표, 줄바꿈, 탭 리더, 3mm 격자 기준 위치는 건드리지 않는다.

적용 범위:

- Web Canvas 기본 텍스트 렌더 경로.
- 조판부호/탭/줄바꿈 계산에는 영향 없음.
- Native SVG/Skia 메트릭은 변경하지 않음.

## 3. 실제 앱 경로 검증

WASM 갱신:

```bash
wasm-pack build --target web --out-dir pkg
```

rhwp-studio 전체 파일 로드 경로로 확인:

```text
http://localhost:7700/?url=/samples/hwp3-sample16-hwp5.hwp&filename=hwp3-sample16-hwp5.hwp
```

산출물:

```text
output/debug/task1116/browser-app-after-glyph-fit/visible0.png
output/debug/task1116/browser-app-after-glyph-fit/visible1.png
output/debug/task1116/browser-app-after-glyph-fit/visible2.png
```

확인:

- p2 목차 leader/page number 유지.
- p3 `Consolidation`, `High Availability`, `Business Continuity Planning`의 시각적 겹침 제거.

## 4. 검증

통과:

```bash
cargo fmt --all -- --check
cargo check --target wasm32-unknown-unknown --lib
wasm-pack build --target web --out-dir pkg
cargo test --test issue_1116 -- --nocapture
cargo test --test issue_874_ktx_toc_page_number_right_align -- --nocapture
cargo test --test issue_1086 -- --nocapture
cargo test --test issue_1105 -- --nocapture
cargo test --test issue_630 -- --nocapture
cargo build --bin rhwp
git diff --check
```

참고:

- #874 테스트의 기존 `LAYOUT_OVERFLOW` 진단 1건은 계속 출력되지만 테스트는 통과한다.
- `wasm-pack`은 현재 플랫폼용 prebuilt `wasm-bindgen` 다운로드 실패 후 cargo install fallback 경고를 출력하지만 빌드는 성공한다.

## 5. 남은 확인

작업지시자 화면에서 강력 새로고침 후 다음 두 산출물을 비교한다.

- p2 목차: 들여쓰기, leader 종료점, 페이지 번호 열.
- p3 본문: 3mm 격자 기준 상하 위치와 영문 토큰 겹침 제거 여부.
