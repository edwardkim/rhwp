# Task #1116 Stage 6 보고서 — 3mm 격자 SVG 재비교

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: 한컴 눈금자/3mm 안내선 기준으로 SVG export 경로 재검증 및 보정

## 1. 재확인 명령

작업지시자 지시에 따라 debug overlay와 control code 없이 3mm 격자만 켠 clean SVG를 다시 생성했다.

```bash
target/debug/rhwp export-svg samples/hwp3-sample16-hwp5.hwp \
  -o output/poc/render-spacing/hwp3-sample16-hwp5-page3-grid-3mm-clean \
  -p 2 \
  --show-grid=3mm
```

함께 확인한 p2 목차:

```bash
target/debug/rhwp export-svg samples/hwp3-sample16-hwp5.hwp \
  -o output/poc/render-spacing/hwp3-sample16-hwp5-page2-grid-3mm-clean \
  -p 1 \
  --show-grid=3mm
```

산출물:

```text
output/poc/render-spacing/hwp3-sample16-hwp5-page2-grid-3mm-clean/hwp3-sample16-hwp5_002.svg
output/poc/render-spacing/hwp3-sample16-hwp5-page3-grid-3mm-clean/hwp3-sample16-hwp5_003.svg
output/poc/render-spacing/hwp3-sample16-hwp5-page3-grid-3mm/hwp3-sample16-hwp5_003.svg
```

## 2. 원인 추가 확인

Stage 5에서는 Web Canvas 경로의 영문 glyph 폭 겹침을 보정했다. 그러나 작업지시자가 재비교에 사용한 경로는 `export-svg`였고, SVG renderer는 브라우저가 실제 렌더링하는 라틴 glyph 폭을 layout advance 안으로 고정하지 않았다.

결과적으로 `Consolidation`, `High Availability`, `Business Continuity Planning`, `Disaster Recovery` 같은 영문 토큰이 한컴보다 넓게 보이며, 3mm 격자 기준으로 문장 폭과 줄 내부 밀도가 달라졌다.

## 3. 구현

수정 위치:

```text
src/renderer/svg.rs
tests/issue_1116.rs
```

변경:

- SVG 텍스트 cluster 중 ASCII 알파벳을 포함한 cluster에만 `textLength`와 `lengthAdjust="spacingAndGlyphs"`를 추가했다.
- `transform scale(x, 1)`이 이미 걸린 장평 텍스트는 `textLength`가 이중 적용되지 않도록 scale 값을 나누어 보정했다.
- 한글 cluster에는 `textLength`를 붙이지 않아 기존 한글 좌표와 폰트 렌더링을 유지했다.
- issue 1116 회귀 테스트에 p3 라틴 glyph 폭 고정 여부와 한글 미적용 여부를 추가했다.

## 4. 검증

통과:

```bash
cargo fmt --all -- --check
cargo build --bin rhwp
cargo test --test issue_1116 -- --nocapture
cargo test --test issue_874_ktx_toc_page_number_right_align -- --nocapture
cargo test --test issue_1086 -- --nocapture
cargo test --test issue_1105 -- --nocapture
cargo test --test issue_630 -- --nocapture
cargo check --target wasm32-unknown-unknown --lib
wasm-pack build --target web --out-dir pkg
git diff --check
```

참고:

- #874 테스트의 기존 `LAYOUT_OVERFLOW` 진단 1건은 계속 출력되지만 테스트는 통과한다.
- `wasm-pack`은 현재 플랫폼용 prebuilt `wasm-bindgen` 다운로드 실패 후 cargo install fallback 경고를 출력하지만 빌드는 성공한다.
