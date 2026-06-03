# Task M100 #1139 Stage 13

## 목적

파일별로 한컴오피스 격자 설정의 `격자 기준 위치` 값이 다르게 표시되는 원인을 확인한다.

작업지시자 관찰:

- `hwp3-sample16-hwp5.hwp`는 한컴오피스 격자 설정에서 종이 기준 `가로 15.00mm`, `세로 20.00mm`로 표시된다.
- `3-09월_교육_통합_2022.hwp`는 한컴오피스 격자 설정에서 종이 기준 `가로 9.00mm`, `세로 24.00mm`로 표시된다.
- 따라서 격자 기준 위치가 파일 어딘가에 저장되어 있거나, 저장된 편집 용지 값에서 유도될 가능성이 있다.

## 조사 계획

1. HWP5 스펙과 기존 PR #1137 문서를 확인해 이미 파싱 중인 격자 관련 필드를 분리한다.
2. 샘플별 `SectionDef.line_grid`, `SectionDef.char_grid`, `PageDef.margin_*` 값을 비교한다.
3. 한컴 UI의 `격자 기준 위치` 값이 별도 필드인지, `PageDef.marginLeft` 및 `PageDef.marginTop + marginHeader` 조합인지 검증한다.
4. 소스 변경이 필요하면 구현 전 작업지시자 승인을 받는다.

## 현재 단서

- HWP5 `SECTION_DEF`에는 `line_grid`, `char_grid`가 있고 rhwp는 이미 파싱/직렬화한다.
- HWPX `<hp:grid lineGrid="..." charGrid="...">`도 rhwp 모델의 `SectionDef`로 보존된다.
- rhwp-studio의 현재 격자 설정 기본값은 `PageDef.marginLeft`, `PageDef.marginTop + PageDef.marginHeader`에서 종이 기준 위치를 계산한다.

## 분석 결과

- 두 대상 샘플의 `SECTION_DEF` grid 값은 모두 0이다.
  - HWP5 `CTRL_HEADER(secd)` payload 기준 `line_grid=0`, `char_grid=0`
  - HWPX 변환본도 `<hp:grid lineGrid="0" charGrid="0" wonggojiFormat="0"/>`
- 한컴오피스의 `격자 기준 위치: 종이` 값은 별도 grid record가 아니라 `PageDef`에서 유도되는 것으로 판단한다.
  - `hwp3-sample16-hwp5.hwp`: `marginLeft=4252HU = 15.00mm`, `marginTop=2836HU + marginHeader=2836HU = 20.01mm`
  - `3-09월_교육_통합_2022.hwp`: `marginLeft=2551HU = 9.00mm`, `marginTop=1984HU + marginHeader=4819HU = 24.00mm`
- 따라서 시각 비교용 CLI는 사람이 파일별 값을 직접 넣는 대신 `PageDef` 기반 자동 계산을 제공하는 것이 맞다.

## 구현

- `export-svg`의 `--grid-origin` 옵션에 `auto` 값을 추가했다.
- `--grid-origin=auto`는 페이지의 `sectionIndex`를 확인한 뒤 해당 구역 `PageDef`에서 종이 기준 격자 원점을 계산한다.
  - `x = marginLeft`
  - `y = marginTop + marginHeader`
- 기존 `--show-grid=3mm` 기본 원점은 그대로 `0,0`으로 유지한다.
- 기존 수동 비교도 계속 가능하다.
  - `--grid-origin=15mm,20mm`
  - `--grid-paper-origin=15mm,20mm`

## 검증

- `cargo fmt --all -- --check`
- `cargo build`
- `target/debug/rhwp export-svg samples/hwp3-sample16-hwp5.hwp -o output/task1139_stage13_grid_auto_sample16 -p 2 --show-grid=3mm --grid-origin=auto`
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -o output/task1139_stage13_grid_auto_exam2022 -p 8 --show-grid=3mm --grid-origin=auto`

검증 결과:

- `hwp3-sample16-hwp5_003.svg`: grid pattern `x=56.6933px`, `y=75.6267px` = 약 `15.00mm`, `20.01mm`
- `3-09월_교육_통합_2022_009.svg`: grid pattern `x=34.0133px`, `y=90.7067px` = 약 `9.00mm`, `24.00mm`

## 결론

파일별 한컴 격자 기준 위치 차이는 `PageDef` 값에서 유도된다. 현재 확인 범위에서는 별도 grid origin 저장 필드는 발견되지 않았다.
