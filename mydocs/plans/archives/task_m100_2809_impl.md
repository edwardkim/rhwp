# 구현계획서 — #2809 표 셀 나눔정렬 마지막 줄·속성 차이 정정

- 이슈: [#2809](https://github.com/edwardkim/rhwp/issues/2809)
- 수행계획서: [`task_m100_2809.md`](task_m100_2809.md)
- 브랜치: `task/2809-distribute-align` (`upstream/devel` @ `58991a768`)
- 작성일: 2026-07-22

## 1. 구현 계약

첨부 HWP의 문제 문단은 `Alignment::Split`(HWPX `DISTRIBUTE_SPACE`)이다. `Split`은
문단의 마지막 줄에서도 내부 공백에 남은 폭을 분배한다. 반면 일반 `Justify`는
기존처럼 마지막 줄과 강제 줄바꿈 줄의 분배를 억제한다. `Split`의 강제 줄바꿈
억제도 기존 동작을 유지한다.

셀 padding, 문단 여백, 저장 `LINE_SEG`, 자간과 공백 분배 산식은 변경하지 않는다.
특히 위쪽 문단의 자간 `-50%`/`6972HU`와 아래쪽 문단의 자간 `0%`/`6872HU` 차이를
상쇄하지 않고 PDF와 같이 서로 다른 분배 폭으로 보존한다.

## 2. 변경 지점

### `src/renderer/layout/paragraph_layout.rs`

1. 정렬 종류와 마지막 줄/머리말·꼬리말/강제 줄바꿈 상태를 받아 공백 분배 여부를
   반환하는 `needs_word_distribution` 헬퍼를 둔다.
2. `Alignment::Split`은 강제 줄바꿈이 아닌 마지막 줄에서도 `true`,
   `Alignment::Justify`는 기존 조건을 유지하도록 분리한다.
3. 음수 자간 `Split`은 마지막 glyph의 실제 잉크 폭을 분배 계산에 예약해 셀 우측
   clip 안에 온전히 배치한다.
4. 일반 `Justify`, 저장 LineSeg와 다른 정렬 경로는 그대로 둔다.
5. Canvas 2D의 폰트 실측 폭 보정은 음수 자간일 때 glyph 자체를 advance 폭으로
   축소하지 않도록 제한한다. 음수 자간은 다음 글자의 시작 위치에만 반영한다.

## 3. 테스트 계획

- `paragraph_layout.rs` 단위 테스트:
  - 단일 마지막 줄에서 `Split=true`, `Justify=false`.
  - `다 같 이`의 두 내부 공백에 양의 slack이 균등하게 배분됨.
- 시각 회귀:
  - 이슈 첨부 HWP 2쪽의 위·아래 `다/같/이` 문자 x 좌표와 span을 비교.
  - WASM SVG와 페이지 레이어 트리에서 위·아래 최종 분배 폭이 PDF처럼 정합함.
  - 위·아래 첫 glyph 잉크 폭이 같아 음수 자간 행만 가로로 눌리지 않음.
  - 위쪽 마지막 glyph 잉크가 셀 우측에서 잘리지 않음.
- 명령:
  - `cargo test --lib issue_2809_split_alignment_tests`
  - `cargo test --lib`
  - `cargo test --test svg_snapshot`
  - `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`
  - `wasm-pack build --target web --out-dir pkg`
  - `cd rhwp-studio && npm run e2e:issue-2809`

## 4. 시각 증적 계획

- 입력: 이슈 #2809 첨부 `jubo_20260104.hwp`.
- 기준: HWP 2020 변환 PDF 6쪽, SHA-256
  `a73d50620bf8fe96beaff72ba0e40cd34f396ec75de9798ac1fd0402e28f8e2b`.
- 비교: 문제 2쪽을 144dpi로 맞춰 라벨의 첫/가운데/마지막 글자와 셀 좌우 경계를
  대조하고, WASM Canvas 2배율에서 동일 페이지를 다시 확인한다.
- 테스트에 사용한 HWP, 기준 PDF, 최종 review PNG와 WASM E2E 보고서를 저장소에
  포함한다. 원본 ZIP 묶음은 후속 요청에 따라 추적 대상에서 제외한다.

## 5. 위험과 완화

| 위험 | 완화 |
|---|---|
| `Justify` 마지막 줄까지 함께 벌어짐 | `Split`과 `Justify` 판정을 헬퍼에서 명시적으로 분리하고 음성 테스트를 둔다. |
| 강제 줄바꿈 의미가 달라짐 | `Justify`와 `Split` 모두 기존 강제 줄바꿈 억제를 보존하는 음성 테스트를 둔다. |
| 증적 재현성 부족 | 원본 HWP/PDF, 최종 review PNG와 E2E HTML의 SHA-256을 증적 README에 고정한다. |
| 폰트 fallback에 따른 외형 차이 | 폰트 외형 대신 원문 charPr/LineSeg와 문자 좌표 span을 검증하고 HWP 2020 PDF를 기준으로 둔다. |
