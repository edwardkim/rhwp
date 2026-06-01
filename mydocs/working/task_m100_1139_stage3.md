# Stage 3 재시각 검증 대응 — Task #1139

## 배경

작업지시자가 Stage 2 수정 후에도 `3-09월_교육_통합_2022.hwp` 5쪽 화면이 한컴오피스와 다르다고 재보고했다. 두 번째 스크린샷이 한컴오피스 기준 화면이다.

## 현재 판단

Stage 2의 괄호 path 보정은 명령 문자열 누출을 줄이는 방향이었지만, 한컴 기준으로는 문24 수식의 전체 조판이 아직 다르다. 스크린샷 비교상 우선 확인할 후보는 다음과 같다.

- `x\`cos`의 backtick spacing이 한컴 대비 과하게 벌어지는지 확인한다.
- `LEFT ( {pi} over {2} -x RIGHT )`의 괄호가 한컴 대비 너무 크거나 굵은지 확인한다.
- 괄호와 분수/`-x`/`dx` 사이 가로 간격이 한컴보다 넓은지 확인한다.
- 문27의 `△△` 항목은 Stage 1에서 TAC picture로 분리했으나, 위치 차이가 있는지 재확인한다.

## 다음 액션

1. 문24 수식의 AST/layout 좌표를 재확인했다.
2. stretched round parenthesis 폭/패딩/선폭을 한컴 화면 기준으로 더 보수적으로 줄였다.
3. SVG/Canvas 양쪽 경로를 다시 맞췄다.
4. 자동 검증 후 작업지시자에게 재시각 확인을 요청한다.

## 변경 내용

- `layout_paren`에서 큰 둥근 괄호 전용 폭을 `fs * 0.333`에서 `fs * 0.27`로 줄였다.
- 큰 둥근 괄호 내부 좌우 패딩을 `fs * 0.08`에서 `fs * 0.03`으로 줄였다.
- SVG/Canvas stretched round parenthesis 선폭을 `fs * 0.055`에서 `fs * 0.042`로 줄였다.
- 텍스트 높이 괄호 glyph 경로는 기존 `fs * 0.333` 폭을 유지해 Task #283 회귀를 피했다.

## 재생성 확인

```bash
./target/release/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 4 -o output/diag_1139_stage3
```

문24 수식의 괄호 path:

```svg
<path d="M45.55,1.80 C42.79,7.07 42.79,25.81 45.55,31.08" fill="none" stroke="#000000" stroke-width="0.50" stroke-linecap="round"/>
<path d="M74.84,1.80 C77.60,7.07 77.60,25.81 74.84,31.08" fill="none" stroke="#000000" stroke-width="0.50" stroke-linecap="round"/>
```

Stage 2 대비 괄호 폭과 stroke가 줄었고, `LEFT`, `RIGHT`, `it`, `ANGLE` 텍스트 누출은 재확인되지 않았다.

## 검증

```bash
cargo fmt --check
cargo test issue_1139 --lib
cargo build --release
./target/release/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 4 -o output/diag_1139_stage3
cargo test renderer::equation::svg_render::tests --lib
wasm-pack build --target web --out-dir pkg
cargo test --lib
```

결과:

- `cargo fmt --check`: 통과
- `cargo test issue_1139 --lib`: 1 passed
- `cargo build --release`: 성공
- 대상 페이지 SVG export: 성공
- SVG 렌더러 테스트: 13 passed
- WASM build: 성공
- 전체 lib 테스트: 1406 passed, 0 failed, 6 ignored

## 주의

이번 단계는 시각 정합 보정이므로 자동 테스트 통과만으로 완료 처리하지 않는다. 작업지시자의 한컴 대비 시각 판단을 기다린다.
