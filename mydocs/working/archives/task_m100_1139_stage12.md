# Task M100 #1139 Stage 12

## 목적

`hwp3-sample16-hwp5.hwp`에서 한컴오피스와 rhwp-studio의 하단 쪽 테두리 및 쪽번호 위치가 다른 문제를 #1139 후속 시각 정합 범위에 추가한다.

작업지시자의 추가 판정:

- 격자 설정은 한컴오피스와 rhwp-studio 모두 `가로 3mm`, `세로 3mm`, 종이 기준 `가로 15mm`, `세로 20mm`로 맞췄다.
- 격자 설정을 맞춘 상태에서도 `hwp3-sample16-hwp5.hwp`의 표시 내용이 하단 경계선/쪽번호 기준으로 한컴오피스와 다르다.
- 비교 기준은 작업지시자 스크린샷 기준으로 세 번째가 rhwp-studio, 네 번째가 한컴오피스다.

## 초기 관찰

- 대상 화면은 문서 전체 3쪽이며, 표시 쪽번호는 `-1-`로 시작하는 구간이다.
- `PAGE_DEF`는 A4, 좌우 15mm, 상하 10mm, 머리말/꼬리말 10mm다.
- `PAGE_BORDER_FILL` 첫 레코드는 `attr=0x00000001`, offset 1420HU(약 5.01mm), border fill id 2다.
- `PageNumPos`는 `attr=0x00000500`으로 가운데 아래 위치다.
- 현재 rhwp SVG probe 기준:
  - 하단 이중 테두리 y 좌표는 약 1072.26px, 1074.36px
  - 쪽번호 baseline y 좌표는 약 1079.12px

## 분석 계획

1. `hwp3-sample16-hwp5.hwp` 3쪽을 `export-svg --show-grid=3mm`로 계속 재생성해 한컴오피스 스크린샷과 비교한다.
2. `PAGE_BORDER_FILL.attr=1` 변환본에서 렌더 기준이 `BodyBased`인지, HWP3 변환본 특례가 필요한지 확인한다.
3. 쪽번호 `PageNumPos`의 아래 위치가 footer 영역 중앙인지, 테두리 기준 보정이 필요한지 확인한다.
4. 변경이 필요하면 HWP5/HWPX 일반 문서와 HWP3 변환본의 page border 계약을 분리해 회귀를 좁힌다.

## 검증 예정

- `target/debug/rhwp export-svg samples/hwp3-sample16-hwp5.hwp -o output/task1139_stage12_svg -p 2 --show-grid=3mm`
- `target/debug/rhwp dump-pages samples/hwp3-sample16-hwp5.hwp -p 2`
- `cargo test` 중 page border/page number 관련 회귀 테스트
- 필요 시 `wasm-pack build --target web --out-dir pkg`

## 승인

작업지시자가 Stage 12 소스 수정을 승인했다.

## 구현

- `BodyBased` 쪽 테두리에서 하단 선 묶음 폭을 footer/쪽번호 영역 쪽으로 추가 확장하지 않도록 보정했다.
- 상단/좌우는 기존 Stage 29의 로고/외곽 정합을 유지하기 위해 선 묶음 visual outset을 그대로 둔다.
- Studio page info 쪽 테두리 계산도 같은 하단 기준을 쓰도록 맞췄다.
- `export-svg --show-grid=3mm` 비교 정확도를 높이기 위해 한컴오피스의 `격자 기준 위치`를 받을 수 있는 `--grid-origin=가로,세로` 옵션을 추가했다.
  - 예: `--show-grid=3mm --grid-origin=15mm,20mm`
  - 동일 의미의 별칭으로 `--grid-paper-origin=15mm,20mm`도 허용한다.
- `src/wasm_api/tests.rs`의 오래된 직접 `PageItem` match를 `PageItem::para_index()` 호출로 바꿔 `EndnoteSeparator` variant 추가 후에도 lib test 컴파일이 되도록 했다.
- `hwp3-sample16-hwp5.hwp` 3쪽 최종 SVG 기준 회귀 테스트를 추가했다.

## 자동 검증

- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `cargo fmt --all -- --check`
- `cargo test -q page_border_fill_sample_basis_matches_hancom_ui --lib`
- `cargo test --test issue_1116 -- --nocapture`
- `target/debug/rhwp export-svg samples/hwp3-sample16-hwp5.hwp -o output/task1139_stage12_svg -p 2 --show-grid=3mm`
- `target/debug/rhwp export-svg samples/hwp3-sample16-hwp5.hwp -o output/task1139_stage12_svg_grid_origin -p 2 --show-grid=3mm --grid-origin=15mm,20mm`
- `target/debug/rhwp export-svg samples/hwp3-sample16-hwp5.hwp -o output/task1139_stage12_svg_p0 -p 0 --show-grid=3mm`
- `cargo build`
- `wasm-pack build --target web --out-dir pkg`

검증 결과:

- 3쪽 SVG 하단 이중 테두리 y 좌표: 약 `1064.76px`, `1066.86px`
- 3쪽 SVG 가운데 아래 쪽번호 baseline y 좌표: 약 `1079.12px`
- 하단 테두리와 쪽번호 사이 간격: 약 `12.26px`

## 시각 검증

작업지시자가 시각 검증 통과를 알렸고, Stage 12 변경분 커밋을 승인했다.
