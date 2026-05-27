# Task M100 #1139 Stage 15

## 목적

`3-09월_교육_통합_2022.hwp` 9쪽에서 격자 설정을 한컴오피스와 동일하게 맞춘 뒤에도 페이지 내용이 미세하게 다른 문제를 분석하고 보정한다.

## 작업지시자 기준

- 격자 설정은 한컴오피스와 rhwp-studio 모두 동일하다.
  - 격자 간격: 가로 `3.00mm`, 세로 `3.00mm`
  - 격자 기준 위치: `종이`
  - 가로 `9.00mm`, 세로 `24.00mm`
- 비교 화면에서 세 번째 스크린샷이 한컴오피스 정답이다.
- 격자 자체가 아니라 9쪽 본문/미주 내용의 미세한 배치 차이가 남아 있다.

## 분석 계획

1. `export-svg --show-grid=3mm --grid-origin=9mm,24mm`와 `--grid-origin=auto`로 9쪽을 재생성한다.
2. 9쪽 좌/우 단의 주요 앵커를 추출한다.
   - 상단 표
   - `문1)`, `문4)`, `문5)`
   - 우측 `문6)`, `문7)`, `[다른 풀이]`
   - 하단 쪽 테두리
3. `dump-pages -p 8`의 page item 순서와 y 위치를 다시 확인한다.
4. 차이가 font/scale인지, 단 내부 `vpos`/미주 separator/미주 사이 간격 문제인지 분리한다.

## 상태

작업지시자 승인 후 한컴 도움말과 HWP5 실제 레코드 기준으로 미주 모양 해석을 재정리했다.
자동 검증 완료, 시각 검증 대기.

## 중간 확인

`target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -o output/task1139_stage15_grid_paper -p 8 --show-grid=3mm --grid-origin=9mm,24mm`와 `--grid-origin=auto`를 비교했다.

- 수동 `9mm,24mm`와 자동 원점의 차이는 SVG pattern 기준 약 `0.002px` 수준이다.
- 따라서 현재 남은 차이는 격자 원점 오차가 아니라 9쪽 내부 내용 배치/렌더링 차이다.

`--debug-overlay` 기준 주요 paragraph anchor는 다음과 같다.

| 항목 | pi | y(px) | 3mm 격자 행 |
|---|---:|---:|---:|
| 제목 문단 | 465 | 90.70 | 0.00 |
| 상단 답안표 | 466 | 110.60 | 1.75 |
| 문1) | 468 | 361.40 | 23.87 |
| 문4) | 480 | 715.10 | 55.07 |
| 문5) | 491 | 973.40 | 77.85 |
| 문6) | 499 | 213.10 | 10.79 |
| 문7) | 511 | 479.70 | 34.31 |
| `[다른 풀이]` Shape | 518 | 803.50 | 62.86 |
| 문7 후반 | 522 | 881.60 | 69.75 |

`RHWP_VPOS_DEBUG=1`로 확인한 결과, 9쪽 미주 왼쪽 단은 `LINE_SEG` vpos 순서와 렌더 y 누적이 같은 흐름을 따른다. 오른쪽 단도 column base `pi=493 vpos=65429` 기준으로 vpos 보정이 적용된다.

현재 추정:

- 페이지 수, 9쪽/10쪽 미주 분기, 격자 원점 문제는 Stage 14까지의 수정으로 맞아 있다.
- 남은 미세 차이는 page item 단위의 시작 y보다는 문단 내부 수식/텍스트 렌더 메트릭, 특히 canvas/SVG 수식 baseline 및 저장 bbox 대비 layout box 스케일 차이일 가능성이 있다.

## 수정 내용

1차로 `src/renderer/layout/paragraph_layout.rs`에서 인라인 수식 배치 시 baseline 계산 뒤 줄 상단 `y`로 클램프하던 처리를 제거했으나, 작업지시자 시각 검증에서 한컴 정답지와 맞지 않는 것으로 확인되어 해당 실험은 되돌렸다.

한컴 도움말을 재확인해 `미주 사이`의 의미가 "앞 번호 미주 내용과 다음 번호 미주 내용 사이의 간격"임을 반영했다.

- `src/renderer/typeset.rs`
  - 기존에 직접 `shape.raw_unknown`을 읽던 특수 미주 단 전환 보정 경로를 `endnote_between_notes_margin()` 헬퍼로 명시화했다.
  - 전역적으로 `미주 사이 7mm`를 최소 간격으로 강제하는 실험은 전체 페이지 수가 24쪽으로 늘어 실패했다. 따라서 2022 원본의 `미주 사이 7mm`는 원본 `LINE_SEG`에 이미 들어 있는 기준 흐름으로 보고 중복 가산하지 않는다.
  - 작업지시자가 추가로 저장한 `3-09월_교육_통합_2024-구분선아래20.hwp`, `3-09월_교육_통합_2024-미주사이20.hwp`를 기준으로 재확인했다.
    - `구분선아래20`: `note_spacing=5669HU`(약 20mm), `raw_unknown=1984HU`(약 7mm), 한컴 23쪽.
    - `미주사이20`: `note_spacing=576HU`(약 2mm), `raw_unknown=5669HU`(약 20mm), 한컴 24쪽.
  - 이에 따라 `미주 사이`가 기본 7mm를 넘는 경우 초과분만 다음 미주 묶음의 `vpos_offset`에 반영한다. `current_height`에 별도로 즉시 더하면 다음 미주 문단의 vpos span에서 한 번 더 잡혀 `미주사이20`이 25쪽으로 밀리므로, pagination 판단은 보정된 vpos 흐름에 맡긴다.
- `mydocs/tech/한글문서파일형식_5.0_revision1.3.md`
  - 표 133 `HWPTAG_FOOTNOTE_SHAPE` 아래에 실제 HWP5 28바이트 레코드와 미문서화 2바이트 메모를 추가했다.
  - 2024 기준 파일 2개로 `note_spacing=구분선 아래`, `raw_unknown=미주 사이` 매핑을 검증했다고 기록했다.

9쪽 비교 SVG:

- `output/task1139_stage15_endnote_spacing_original/3-09월_교육_통합_2022_009.svg`
- `output/task1139_stage15_endnote_spacing_below20/3-09월_교육_통합_2024-구분선아래20_009.svg`
- `output/task1139_stage15_endnote_spacing_between20/3-09월_교육_통합_2024-미주사이20_009.svg`

문단 anchor, 페이지 수, `문8)` 10쪽 시작 조건은 유지된다.

## 검증

- `cargo build`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 6 passed
- `cargo test -q test_empty_document_info --lib`
  - 1 passed
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -o output/task1139_stage15_hancom_endnote_help -p 8 --show-grid=3mm --grid-origin=9mm,24mm`
  - 23페이지 로드, 9쪽 SVG 생성
- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2024-구분선아래20.hwp`
  - 23페이지 로드
- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2024-미주사이20.hwp`
  - 24페이지 로드
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2024-구분선아래20.hwp -o output/task1139_stage15_endnote_spacing_below20 -p 8 --show-grid=3mm --grid-origin=9mm,24mm`
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2024-미주사이20.hwp -o output/task1139_stage15_endnote_spacing_between20 -p 8 --show-grid=3mm --grid-origin=9mm,24mm`
- `wasm-pack build --target web --out-dir pkg`

## 대기

시각 검증 후 커밋한다. 작업지시자가 명시한 대로 아직 커밋하지 않는다.

## 한글 문서 형식 문서 재확인

작업지시자 재보고: 1차 수식 baseline 클램프 제거 후에도 한컴 정답지와 맞지 않다. `mydocs/tech/한글문서파일형식_5.0_revision1.3.md`에서 다음 항목이 직접 관련된다.

한컴 도움말 재확인:

- 미주는 현재 구역의 끝 또는 문서의 끝에 놓인다.
- `구분선 위`: 본문과 미주 구분선 사이 간격
- `구분선 아래`: 미주 구분선과 미주 내용 사이 간격
- `미주 사이`: 앞 번호 미주 내용과 다음 번호 미주 내용 사이 간격
- `미주 위치`: 문서의 끝 또는 구역의 끝

대상 HWP5 실제 레코드:

- 두 번째 `FOOTNOTE_SHAPE` record size는 28바이트다.
- 표 133의 `주석 사이 여백` 위치에는 한컴 UI `구분선 아래 2mm`가 들어 있다.
- 그 다음 미문서화 2바이트에는 한컴 UI `미주 사이 7mm`가 들어 있다.
- 따라서 스펙 문서의 26바이트 표만 그대로 따르면 `구분선 종류/굵기/색상`이 2바이트 밀린다.

- 표 44 `문단 모양 속성1`
  - `bit 20~21`: 문단 세로 정렬 (`0=글꼴기준`, `1=위쪽`, `2=가운데`, `3=아래`)
  - `bit 22`: 글꼴에 어울리는 줄 높이 여부
  - 현재 `getStyleDetail`에는 표시되지만 `ResolvedParaStyle`/레이아웃 계산에는 직접 반영되지 않는다.
- 표 62 `문단의 레이아웃`
  - `줄의 세로 위치`, `줄의 높이`, `텍스트 부분의 높이`, `줄의 세로 위치에서 베이스라인까지 거리`, `줄간격`이 모두 별도 저장된다.
  - 현재 9쪽 흐름은 `PARA_LINE_SEG.vertical_pos` 기준으로 맞지만, 줄 내부 텍스트/수식 세로 배치에는 `text_height`와 저장 baseline을 더 정밀하게 써야 할 가능성이 있다.
- 표 70 `개체 공통 속성`
  - `bit 0`: 글자처럼 취급 여부
  - `bit 2`: 줄 간격에 영향을 줄지 여부
  - 현재 `parse_common_obj_attr`는 bit 0은 `treat_as_char`로 구조화하지만 bit 2는 별도 필드로 구조화하지 않는다.
- 표 105 `수식 개체 속성`
  - `INT16 base line` 필드가 있다.
  - 현재 parser는 `Equation.baseline`으로 읽지만, `paragraph_layout.rs`의 인라인 수식 y 계산은 이 저장 baseline을 사용하지 않고 equation layout 결과의 `layout_box.baseline`만 사용한다.

추가로 `hwp5-inventory` 원본 record를 확인하면 대상 파일의 수식 `EQEDIT` payload에는 `font_size=0x0384(900)` 뒤에 `baseline=0x0056(86)` 같은 값이 반복된다. 즉, 파일은 수식별 기준선 정보를 실제로 들고 있고, 현재 렌더러가 이를 시각 배치에 반영하지 않는 것이 남은 차이의 더 유력한 원인이다.

다음 보정 후보:

1. `Equation.baseline`을 인라인 수식 y 계산에 반영한다.
2. `CommonObjAttr.attr bit2`를 구조화해 `줄 간격에 영향을 줄지 여부`가 line height/advance에 미치는 영향을 확인한다.
3. `ParaShape.attr1 bit20~22`를 `ResolvedParaStyle`로 전달하고, `LINE_SEG.text_height`/`baseline_distance`와 결합해 글꼴기준/위쪽/가운데/아래 정렬 차이를 재현한다.
