# Stage 2 수정 및 검증 보고서 — Task #1139

## 수정 내용

- `src/renderer/equation/svg_render.rs`
  - 큰 둥근 괄호 `(`, `)`의 stretched path를 단일 quadratic 곡선에서 cubic 곡선으로 변경했다.
  - round cap과 약간 두꺼운 선폭을 적용해 한컴의 큰 괄호에 더 가깝게 보이도록 했다.
  - `test_issue_1139_integral_left_right_parens_are_curved` 회귀 테스트를 추가했다.
- `src/renderer/equation/canvas_render.rs`
  - rhwp-studio WASM Canvas 렌더 경로도 같은 cubic 곡선/선폭/line cap 정책으로 맞췄다.

## 샘플 재확인

```bash
./target/release/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 4 -o output/diag_1139_after
rg -n "stroke-linecap=\"round\"|>LEFT<|>RIGHT<|>it<|>ANGLE<" output/diag_1139_after/3-09월_교육_통합_2022_005.svg
```

확인된 문24 괄호 path:

```svg
<path d="M46.23,1.80 C42.83,7.07 42.83,25.81 46.23,31.08" fill="none" stroke="#000000" stroke-width="0.66" stroke-linecap="round"/>
<path d="M76.88,1.80 C80.27,7.07 80.27,25.81 76.88,31.08" fill="none" stroke="#000000" stroke-width="0.66" stroke-linecap="round"/>
```

`LEFT`, `RIGHT`, `it`, `ANGLE`의 텍스트 누출은 재확인되지 않았다.

## 자동 검증

```bash
cargo test issue_1139 --lib
cargo test renderer::equation::svg_render::tests --lib
cargo build --release
wasm-pack build --target web --out-dir pkg
cargo test --lib
```

결과:

- `cargo test issue_1139 --lib`: 1 passed
- `cargo test renderer::equation::svg_render::tests --lib`: 13 passed
- `cargo build --release`: 성공
- `wasm-pack build --target web --out-dir pkg`: 성공
- `cargo test --lib`: 1406 passed, 0 failed, 6 ignored

## 남은 판단

수정은 자동 검증과 SVG 구조 검증을 통과했다. 최종 한컴 대비 시각 정합성은 작업지시자의 화면 비교 판단이 필요하다.

