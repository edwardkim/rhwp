# 작업기록 — Task M100 #1599 Stage 1

## 목표

- 표 셀 중심선이 SVG/렌더링 결과에서 누락되는 문제를 수정한다.
- 샘플 `추진일정.hwp`와 `추진일정.hwpx`의 중심선 스타일을 같은 출력으로 보존한다.

## 확인 내용

- 이슈: edwardkim/rhwp#1599
- 샘플: `samples/추진일정.hwp`, `samples/추진일정.hwpx`, `pdf/추진일정-2024.pdf`
- HWP5 probe 결과 중심선 `BorderFill`은 `attr=0x2100`으로 확인되었다.
- `0x2100`은 bit 13 중심선 유무와 bit 8 slash `Crooked=1` 조합이며, HWPX `centerLine="VERTICAL"`에 대응한다.

## 구현 요약

- `BorderFill`에 `CenterLine` 방향 값을 추가했다.
- HWP5/HWPX 파서와 serializer가 `NONE`/`VERTICAL`/`HORIZONTAL`/`CROSS` 중심선을 보존하도록 보강했다.
- 셀 대각선 렌더링 경로에서 중심선 단독 설정도 기존 `DiagonalLine` 색/굵기/종류로 그리도록 수정했다.
- 기존 `BorderFill` 생성 지점과 테스트 fixture는 기본 `CenterLine::None`을 채우도록 정리했다.

## 검증 기록

- `cargo test --lib center_line -- --nocapture`: 통과
- `cargo test --lib diagonal -- --nocapture`: 통과
- `cargo test --lib`: 통과, 1974 passed, 0 failed, 7 ignored
- `cargo fmt --check`: 통과
- `git diff --check`: 통과
- `cargo run --quiet --bin rhwp -- hwp5-borderfill-diagonal-probe samples/추진일정.hwp samples/추진일정.hwp --out-dir output/poc/issue1599/hwp5-borderfill-probe`: `attr=0x2100` 중심선 확인
- `cargo run --quiet --bin rhwp -- export-svg samples/추진일정.hwp -o output/poc/issue1599/hwp-svg`: SVG 생성
- `cargo run --quiet --bin rhwp -- export-svg samples/추진일정.hwpx -o output/poc/issue1599/hwpx-svg`: SVG 생성
- HWP/HWPX 출력 SVG SHA256 동일: `392775b970509757d4856c9df62bd3b485b979c04e4853a9ac8d87a554e302d9`
- 출력 SVG의 중심선 색 stroke 확인: `#41c7f4` 9개, `#ff0000` 4개
- 시각 판단용 PNG 생성: `output/poc/issue1599/visual/추진일정-hwp.png`, `output/poc/issue1599/visual/추진일정-hwpx.png`
- HWP/HWPX 출력 PNG SHA256 동일: `0b2b8ccec805d856e6ba51ecd331992a993f022da8b97a9ae777c5ec68186faf`
- 출력 PNG 크기: 794x1123
- `wasm-pack build --target web --out-dir pkg`: 통과

## 남은 판단

- 최종 시각 품질 판단은 작업지시자 확인을 기다린다.
