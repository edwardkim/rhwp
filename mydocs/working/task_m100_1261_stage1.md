# Task #1261 Stage1 보고

## 대상

- 이슈: [#1261](https://github.com/edwardkim/rhwp/issues/1261)
- 브랜치: `local/task_m100_1261`
- 문서: `samples/3-10월_교육_통합_2022.hwp`
- 위치: 5쪽 `문28)` 조건 박스와 선택지

## 진단

`dump-pages -p 4` 기준 5쪽 `문28)`은 문단 `pi=306`에 조건 글상자와 선택지 텍스트가 함께 들어 있다.
조건 박스 컨트롤 `ci=0`은 글자처럼 취급되는 Shape이며, `common.height`는 약 16px 수준이지만 `shape_attr.current_height`는 약 95px로 실제 렌더링 높이가 더 크다.

기존 조판은 다음 두 조건 때문에 선택지 `①②③` 줄을 조건 박스 내부 y로 올렸다.

- TAC Shape 높이를 `common.height`만 사용해 실제 박스 높이를 줄 진행량에 충분히 반영하지 못했다.
- 문단 전체에 TAC Shape가 있다는 사실만 보고 글상자 안내용 공백 줄까지 폰트 높이로 축소했다.

그 결과 조건 글상자 하단을 지나야 할 선택지 줄이 박스 내부 문장과 겹쳤다.

## 수정

- TAC Picture/Shape 높이를 계산할 때 Shape는 `common.height`와 `shape_attr.current_height` 중 큰 값을 사용하도록 보정했다.
- 줄별 TAC Shape 판정을 문단 전체 기준이 아니라 현재 줄 기준으로 바꿨다.
- 현재 줄이 공백뿐이면 TAC Shape 폰트 높이 축소 보정을 적용하지 않도록 제한했다.
- 문28 조건 박스 하단보다 선택지 첫 줄이 아래에서 시작하는 회귀 테스트를 추가했다.

## 검증

- `cargo fmt -- --check`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1261_2022_oct_page5_question28_choices_stay_below_condition_box -- --nocapture`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1245_2022_page7_square_pictures_use_relative_line_vpos -- --nocapture`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `cargo test --test issue_1219_equation_line_hangul_advance -- --nocapture`
- `cargo test --lib tac -- --nocapture`
- `cargo test --lib textbox -- --nocapture`
- `wasm-pack build --target web --out-dir pkg`

## 시각 확인

- SVG 산출물: `output/task1261_stage1_page5_svg_v2/3-10월_교육_통합_2022_005.svg`
- PNG 산출물: `output/task1261_stage1_page5_svg_v2/3-10월_교육_통합_2022_005.png`
- 조건 박스 하단은 약 `y=988.17`, 선택지 첫 줄은 약 `y=1004.53`에서 시작한다.
- 2026-06-03 작업지시자가 `localhost:7700` 화면에서 시각 검증을 확인했다.

## 남은 일

- 문28 수정은 커밋 대상으로 정리한다.
- 작업지시자가 제공한 `3-09월_교육_통합_2024-미주사이20-2024.pdf` 기준 문8 겹침 원인은 다음 스테이지에서 별도로 조사한다.
