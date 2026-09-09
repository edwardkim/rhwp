# 구현계획서 — Task M100 #1599

## 설계 근거

기존 #1038 정정 이후 셀 대각선은 두 축으로 분리되어 있다.

- `BorderFill.attr`: slash/backSlash 방향 비트와 중심선 비트
- `BorderFill.diagonal`: 선 종류, 굵기, 색

이번 결함은 중심선 비트가 있어도 `render_cell_diagonal`이 slash/backSlash만 보고 조기 반환하는 데서
발생한다. 중심선도 대각선 탭의 공통 선 종류/굵기/색을 사용하므로 기존 `DiagonalLine` 스타일을
재사용한다.

샘플 `추진일정.hwp`에서 중심선 `BorderFill`은 `attr=0x2100`으로 관찰되었다. 이는 HWP5
bit 13 중심선 유무와 bit 8 slash `Crooked=1` 조합이며, HWPX의
`centerLine="VERTICAL"` / `<hh:slash Crooked="1">`와 대응한다. 따라서 모델에는
`NONE`/`VERTICAL`/`HORIZONTAL`/`CROSS` 방향을 별도 값으로 보존하고, HWP5/HWPX
직렬화 때 보조 비트까지 함께 복원한다.

Stage 2에서 한컴 2024 및 기준 PDF를 대조한 결과, HWPX literal `VERTICAL`은 화면상
셀 중앙 세로선이 아니라 셀 중앙 가로 진행 막대로 표시된다. 반대로 `HORIZONTAL`은
셀 중앙 세로선으로 렌더링해야 한다. 모델 값 이름은 HWPX literal 보존용으로 유지하고,
렌더링 단계에서 한컴 2024 기준 방향으로 해석한다.

## 구현 단계

### Stage 1 — 파서/모델 확인

- HWP5 `BORDER_FILL` raw attr bit 13과 bit 8/10 보조 비트에서 중심선 방향을 해석한다.
- HWPX `<hh:borderFill centerLine="...">`와 `slash/backSlash`의 `Crooked`/`isCounter`를 파싱한다.
- `centerLine != NONE`이면 HWP5 attr bit 13과 방향 보조 비트를 직렬화에서 보존한다.

### Stage 2 — 렌더링

- `src/renderer/layout/border_rendering.rs`의 `render_cell_diagonal`에서 중심선 방향을 감지한다.
- 중심선 단독 설정에서도 `diagonal.diagonal_type != 0`이면 선을 그린다.
- 중심선은 셀 중앙 가로/세로 선 후보를 기존 선 스타일로 렌더링한다.

### Stage 3 — 테스트

- 중심선 bit만 있는 `BorderFill`이 선 노드를 생성하는 단위 테스트 추가
- HWPX `centerLine` 속성 파싱 단위 테스트 추가
- HWPX 저장 시 `centerLine`과 `Crooked` 보조 비트가 유지되는 단위 테스트 추가
- 기존 slash/backSlash 대각선 테스트 회귀 확인

### Stage 4 — 샘플 검증

- `samples/추진일정.hwp` / `samples/추진일정.hwpx`에서 중심선이 SVG에 출력되는지 확인
- `wasm-pack build --target web --out-dir pkg`를 최종 확인으로 수행한다.
- 결과는 단계 보고서에 기록하고, 최종 시각 판단은 작업지시자 확인을 기다린다.
