# 작업기록 — Task M100 #1599 Stage 2

## 목표

- Stage 1에서 표시되기 시작한 중심선 표현을 한컴 2024 및 기준 PDF 표현에 맞게 보정한다.
- 현재 출력은 중심선을 셀 중앙 세로선으로 렌더링하지만, 작업지시자 확인 결과 한컴 2024 기준 표현 및 PDF 기준과 다르다.

## 관찰

- 한컴 2024 화면에서는 월별 셀 안의 일정 표시가 중심 세로선이 아니라 셀 내부의 가로 진행 막대처럼 보인다.
- 기준 PDF와 현재 rhwp SVG/PNG 출력의 표현 방향과 위치를 비교해야 한다.
- Stage 1 커밋: `e24ddc76b` (`task 1599: 표 중심선 1차 렌더링 보강`)
- 기준 PDF: `pdf/추진일정-2024.pdf`는 Hwp 2024 13.0.0.3622에서 생성되었다.
- HWPX 원문에서 문제 셀은 `centerLine="VERTICAL"` 및 `<hh:slash Crooked="1">`를 사용한다.
- 기준 PDF에서 이 값은 셀 중앙 세로선이 아니라 셀 중앙 가로 진행 막대로 표시된다.

## 분석 계획

1. `pdf/추진일정-2024.pdf`를 PNG로 렌더링한다.
2. 현재 rhwp 출력 `output/poc/issue1599/visual/추진일정-hwp.png`와 기준 PDF PNG를 비교한다.
3. HWPX `centerLine`/`slash`/`diagonal` 조합이 실제 한컴에서 어떤 선형 요소로 표시되는지 재해석한다.
4. 렌더러와 필요 시 모델 해석을 수정한다.

## 구현 요약

- 모델과 HWPX/HWP5 직렬화는 `VERTICAL` literal과 bit 8 매핑을 그대로 보존한다.
- 렌더링에서는 한컴 2024 기준에 맞춰 `CenterLine::Vertical`을 셀 중앙 가로선으로 그린다.
- `CenterLine::Horizontal`은 셀 중앙 세로선으로 그린다.
- `Cross`는 기존처럼 가로선과 세로선을 모두 그린다.

## 검증 기록

- `pdfinfo pdf/추진일정-2024.pdf`: Hwp 2024 13.0.0.3622 생성 PDF 확인
- `pdftoppm -png -r 96 -singlefile pdf/추진일정-2024.pdf output/poc/issue1599/stage2/pdf/추진일정-2024`: 기준 PDF PNG 생성
- `cargo test --lib center_line -- --nocapture`: 통과
- `cargo test --lib diagonal -- --nocapture`: 통과
- `cargo test --lib`: 통과, 1975 passed, 0 failed, 7 ignored
- `cargo fmt --check`: 통과
- `git diff --check`: 통과
- `cargo run --quiet --bin rhwp -- export-svg samples/추진일정.hwp -o output/poc/issue1599/stage2/rhwp-hwp-svg`: SVG 생성
- `cargo run --quiet --bin rhwp -- export-svg samples/추진일정.hwpx -o output/poc/issue1599/stage2/rhwp-hwpx-svg`: SVG 생성
- HWP/HWPX 출력 SVG SHA256 동일: `edd86a5744bd95d4597a43f9ceffd387701ece85f0d18f205db16c5e13d0a63a`
- 출력 SVG의 색상 중심선은 모두 가로선으로 생성됨
- 시각 판단용 PNG 생성: `output/poc/issue1599/stage2/rhwp-visual/추진일정-hwp.png`, `output/poc/issue1599/stage2/rhwp-visual/추진일정-hwpx.png`
- HWP/HWPX 출력 PNG SHA256 동일: `1d55686cf6ee6130e6b750e5ec4abd55ed11240b211f26d2f4bb04daf7a2379e`
- 출력 PNG 크기: 794x1123
- 기준 PDF PNG: `output/poc/issue1599/stage2/pdf/추진일정-2024.png`, 794x1122
- `wasm-pack build --target web --out-dir pkg`: 통과
- 작업지시자 시각 검증: 완료

## 검증 계획

- `cargo test --lib center_line -- --nocapture`
- `cargo test --lib diagonal -- --nocapture`
- `cargo fmt --check`
- `git diff --check`
- `wasm-pack build --target web --out-dir pkg`
- 기준 PDF PNG와 rhwp PNG를 나란히 확인하고 작업지시자 시각 판단을 기다린다.
