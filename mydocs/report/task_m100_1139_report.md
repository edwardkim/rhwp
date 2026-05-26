# 최종 보고서 — Task #1139

## 요약

`3-09월_교육_통합_2022.hwp` 5쪽 문24 수식에서 한컴 대비 이상 문자처럼 보이던 부분을 조사했다. 수식 명령어가 문자로 출력되는 문제가 아니라, 큰 둥근 괄호가 너무 얇은 단일 곡선으로 렌더되어 세로 막대처럼 보이는 문제로 확인했다.

## 변경

- SVG 수식 렌더러의 stretched round parenthesis를 cubic path로 변경했다.
- Canvas 수식 렌더러도 동일한 path 정책으로 맞춰 rhwp-studio 표시 경로와 SVG export 경로를 일치시켰다.
- 문24 fixture 기반 회귀 테스트를 추가해 `LEFT/RIGHT` 명령 문자열이 누출되지 않고 큰 괄호가 곡선 path로 출력되는지 검증했다.

## 진단 메모

- 문23 `lim` 수식은 명령 누출 없이 정상 구조로 렌더된다.
- 문24 `LEFT ( {pi} over {2} -x RIGHT )`의 기존 괄호 path가 한컴 대비 이상 문자처럼 보이는 핵심 후보였다.
- 문27의 작은 `△△` 모양은 Equation 텍스트 누출이 아니라 원본의 TAC `Control::Picture`로 분리했다.

## 검증

```bash
cargo test issue_1139 --lib
cargo test renderer::equation::svg_render::tests --lib
cargo build --release
./target/release/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 4 -o output/diag_1139_after
wasm-pack build --target web --out-dir pkg
cargo test --lib
```

결과:

- `issue_1139` 테스트: 1 passed
- SVG 렌더러 테스트: 13 passed
- release build: 성공
- 대상 페이지 SVG export: 성공
- WASM build: 성공
- 전체 lib 테스트: 1406 passed, 0 failed, 6 ignored

## 후속

자동 검증은 완료됐다. UI/렌더링 정합 작업이므로 한컴오피스 화면과 rhwp-studio 화면의 최종 시각 확인은 작업지시자 판정 대기 상태다.

