# Task #1129 최종 보고서 - 한컴오피스식 격자 보기 및 쪽 테두리 정합

- 이슈: [#1129](https://github.com/edwardkim/rhwp/issues/1129)
- 브랜치: `local/task_m100_1129`
- 기준 브랜치: `upstream/devel`
- 일자: 2026-05-26

## 1. 작업 결과

`rhwp-studio`에서 한컴오피스처럼 격자 보기를 켜고 끌 수 있게 했고, 격자 설정 대화상자에서 모양/위치/방식/간격/기준 위치/오프셋을 조정할 수 있게 했다.

격자 표시와 함께 쪽 테두리/배경 설정도 실제 문서 기준에 맞게 보존하고 표시하도록 보강했다. 특히 `hwp3-sample16-hwp5.hwp`, `종이기준.hwp`, `쪽기준.hwp` 비교를 통해 한컴오피스의 쪽 기준/종이 기준 차이를 반영했고, HWP3 원본에는 종이 기준 선택이 없으므로 쪽 기준으로 렌더링하도록 정리했다.

Stage 29에서는 `--show-grid` SVG 비교와 사용자 시각 판단 결과를 바탕으로 쪽 기준 외곽선 위치를 최종 보정했다.

## 2. 주요 변경

- 보기 메뉴와 도구막대 `격자 보기` 명령 활성화
- 페이지별 격자 오버레이와 격자 설정 대화상자 추가
- 격자 점/가로선/세로선/가로세로선 표시 모드 지원
- `쪽`/`종이` 기준 위치 전환과 기준별 기본 오프셋 처리
- 확대 배율별 격자 점 소실 보정
- 격자 점 크기와 SVG `--show-grid` 디버그 출력 정리
- HWP5 `SECTION_DEFINE`의 줄/글자 격자 정보 보존
- HWPX `hp:grid` 및 `snapToGrid` 보존
- 쪽 테두리/배경 대화상자 추가 및 문서 속성 표시
- HWP3/HWP5/HWPX의 쪽 테두리 기준 파싱 분리
- 본문 기준 쪽 테두리 외곽선 위치와 double-line 보정
- `hwp3-sample16-hwp5.hwp` 첫 페이지 로고와 외곽선 간격 정합 보정

## 3. 기준 위치 정책

- `쪽`: 본문 쪽 영역 기준. HWP3 원본은 이 기준으로 취급한다.
- `종이`: 종이 전체 기준. HWP5/HWPX에서 저장된 기준값을 보존한다.
- UI에 표시되는 기준값과 렌더러가 실제 외곽선을 배치하는 기준값을 분리했다.
- 쪽 기준 외곽선은 본문 영역과 머리말/꼬리말 영역을 포함한 한컴오피스식 기준 위치에 맞춘다.

## 4. 검증

- `cargo fmt --all -- --check` 통과
- `cargo build --verbose` 통과
- `cargo test canvas_layer_tree_matches_legacy --lib` 통과
- `cargo check --target wasm32-unknown-unknown --lib` 통과
- `cargo test --features native-skia skia --lib` 통과
- `cargo test --verbose` 통과
- `cargo clippy -- -D warnings` 통과
- `wasm-pack build --target web --out-dir pkg` 통과
- `./target/debug/rhwp export-svg --show-grid ...` 기반 `hwp3-sample16-hwp5.hwp` 정밀 비교 수행
- 작업지시자 시각 판단 완료

## 5. 참고

- `rhwp-studio`는 Vite dev 서버에서 TypeScript/CSS가 실시간 반영된다.
- Rust/WASM 변경은 시각 판단 요청 전에 `wasm-pack build --target web --out-dir pkg`를 수행해야 한다.
- PR은 일반 Open PR로 생성하며, 머지 작업은 수행하지 않는다.
- 이슈 close는 작업지시자 승인 전까지 수행하지 않는다.
