# Task #1510 Stage 3 — HWPX 페이지네이션 원인 분리

## 1. 출발점

Stage 2 커밋: `6b61d683 task 1510: visible float 표 시각 정합 보정`

Stage 2는 HWP 샘플의 visible text host 문단에 함께 앵커된 `TopAndBottom`
floating 표를 보정했다.

- 음수 `vertical_offset` 표는 visible host 경로에서만 선언 위치를 반영한다.
- 양수 `vertical_offset` 표는 렌더된 표 y 구간에 후속 본문이 닿을 때만 회피한다.
- HWP 기준 PDF/PNG는 한컴 2024 1페이지 기준에 근접했다.

## 2. 남은 문제

HWPX 샘플은 한컴 2024 PDF가 2페이지인데, rhwp는 1페이지로 렌더한다.

Stage 2 조사에서 확인한 차이:

- HWP 샘플: B 표 `vertical_offset = -2000 HU`로 파싱된다.
- HWPX 샘플: B 표 `vertical_offset = 0`으로 파싱된다.
- HWPX는 Stage 2 HWP 전용 active exclusion을 적용하지 않도록 gate 처리했다.

## 3. 조사 방향

1. HWPX 원본 XML의 표 속성 값을 확인한다.
   - B 표의 음수 offset이 XML에 없는지, parser가 누락하는지 분리한다.
   - A 표의 양수 offset과 `textWrap`, `vertRelTo` 매핑을 확인한다.
2. 한컴 2024 HWPX PDF가 2페이지가 되는 원인을 분리한다.
   - HWPX semantics 상 visible host `TopAndBottom` 양수 offset 표가 본문 flow를 더 강하게
     밀어야 하는지 확인한다.
   - 또는 HWPX 생성 샘플 자체가 HWP 샘플과 동등하지 않은지 확인한다.
3. HWP 경로 Stage 2 보정과 #986/#712 회귀를 유지한다.

## 4. 검증 계획

- `target/debug/rhwp dump samples/issue1510_coanchored_float_tables.hwpx -s 0 -p 0`
- HWPX zip XML 직접 확인
- `target/debug/rhwp export-pdf samples/issue1510_coanchored_float_tables.hwpx`
- `pdftoppm` PNG 추출 후 `pdf/issue1510_coanchored_float_tables-hwpx-2024.pdf`와 비교
- `cargo test --test issue_1510 -- --nocapture`
- 필요 시 HWPX 전용 회귀 테스트 추가

## 5. 원인 확인

HWPX 원본 XML의 B 표 `vertOffset`은 `4294965296`이다. 이는 unsigned 표현의
`-2000 HU`인데, HWPX parser가 `parse_i32()`로 읽으면서 0으로 포화되어 B 표가
음수 offset이 아닌 0 offset으로 들어왔다.

또한 HWPX `lineseg`의 순수 본문 줄은 다음 값을 가진다.

- `vertsize=1100`
- `textheight=1000`
- `spacing=600`

한컴 2024 PDF의 filler 본문 줄 피치는 약 `15.98pt`로, `textheight + spacing`
(`1600 HU`)에 가깝다. rhwp는 `vertsize + spacing` (`1700 HU`)으로 진행해
페이지 하단으로 갈수록 본문이 크게 내려갔다.

마지막으로 visible text host 문단 안의 B/C 같은 non-positive float siblings는
layout에서 세로로 쌓이지만, typeset reserve는 각각 `para_start_height` 기준으로
계산해 C 표 높이만큼 page break 판단이 느슨했다.

## 6. 구현 내용

- HWPX table `<hp:pos>`의 `vertOffset`/`horzOffset`를 wrapping signed 값으로 파싱.
- HWPX source의 순수 텍스트 줄은 렌더/조판용 line height를 `textheight`로 낮춰
  `textheight + spacing` 피치를 사용.
- visible text host의 양수 offset float는 후속 paragraph가 표 구간과 겹치면 회피.
- HWPX visible text host의 non-positive float는 layout과 typeset 양쪽에서
  현재 흐름 높이를 기준으로 세로 stacking.
- `tests/issue_1510.rs`에 HWPX 2페이지 baseline, B/C stacking, filler 29/30 분할
  회귀 테스트 추가.

## 7. 검증 결과

- `cargo build --bin rhwp`
- `cargo test --test issue_1510 -- --nocapture`
- `cargo test --lib test_parse_hwpx_table_materializes_hwp_common_attrs -- --nocapture`
- `cargo test --test issue_986 -- --nocapture`
- `cargo test --test issue_712 -- --nocapture`

`issue_712`는 통과했으나 기존 page 26의 작은 `LAYOUT_OVERFLOW` 로그
(`overflow=2.8px`)가 출력된다.

시각 비교:

- HWPX page 1: `diff>30 = 2.2805%`, `diff>80 = 1.4841%`, `rms = 21.614`
- HWPX page 2: `diff>30 = 0.1450%`, `diff>80 = 0.0807%`, `rms = 4.388`
- 주요 bbox:
  - Hancom `filler paragraph 29`: y=730.3pt
  - rhwp `filler paragraph 29`: y=731.3pt
  - Hancom/rhwp 모두 `filler paragraph 30`은 2쪽 시작

비교 산출물:

- `output/pdf/issue1510_stage3_compare/hwpx_p1_side_by_side_new.png`
- `output/pdf/issue1510_stage3_compare/hwpx_p2_side_by_side_new.png`
- `output/pdf/issue1510_stage3_compare/hwpx_p1_diff_new.png`
- `output/pdf/issue1510_stage3_compare/hwpx_p2_diff_new.png`
