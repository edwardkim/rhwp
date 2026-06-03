# Task #1261 Stage3 계획 - 문8 미주 간격 공통 로직 재검증

## 배경

Stage2에서는 `3-09월_교육_통합_2024-미주사이20.hwp` 10쪽 `문8）` 제목이 직전 문7 마지막 수식과 겹치는 문제를 렌더 단계에서 보정했다.

작업지시자는 미주 사이와 구분선 아래가 공통 로직으로 처리되어야 하는데, 왜 `문8）`만 공통 로직으로 처리되지 않는지 검증 후 수정하라고 지시했다.

## 목표

1. `문8）`이 기존 미주 사이 공통 로직을 타지 않거나, 타더라도 실제 수식 하단을 기준으로 삼지 못하는 이유를 확인한다.
2. Stage2의 렌더 후단 보정이 문8 전용처럼 보이는지 검토하고, 가능하면 공통 미주 간격 산정 경로로 수정 위치를 옮긴다.
3. `미주 사이 20mm`, `구분선 아래 20mm`, 기본 `미주 사이 7mm` 기준 테스트를 함께 검증한다.

## 조사 절차

- `HeightCursor::vpos_adjust()`의 compact endnote 새 미주 제목 분기와 `endnote_between_notes_hu` min-gap 분기를 추적한다.
- `typeset.rs`의 `endnote_between_notes_margin()` 적용과 `dump-pages` 페이지 분배가 문8 경계에서 어떻게 계산되는지 확인한다.
- `layout.rs`의 Stage2 보정이 공통 간격 정책을 우회하는지 확인한다.
- 필요하면 공통 로직이 사용할 수 있는 직전 미주 실제 콘텐츠 하단 또는 display 수식 하단 추정값을 공통 헬퍼로 정리한다.

## 주의

- 소스 수정은 이 문서 생성 후 진행한다.
- Stage2 커밋 `6aa638c6`은 커밋된 기준으로 유지하고, Stage3 변경은 새 커밋으로 분리한다.
- `pkg/` 산출물은 작업지시자가 수동 WASM 검증에 사용하므로 Codex 커밋 범위에 포함하지 않는다.

## 조사 결과

- `typeset.rs`는 `FOOTNOTE_SHAPE.raw_unknown`을 한컴 UI의 `미주 사이` 값으로 읽고, 미주 경계마다 같은 `endnote_between_notes_hu` 값을 `HeightCursor`에 전달한다.
- `구분선 아래` 값은 `note_spacing`으로 읽혀 미주 구분선 아래 간격에 적용된다. 따라서 `문8）`만 미주 사이/구분선 아래 공통 값 주입에서 제외된 것은 아니었다.
- 실제 원인은 `HeightCursor::vpos_adjust()`의 compact 미주 새 문항 제목 분기였다.
  - `문8）` 직전 문7 마지막 풀이 문단 `pi=522`는 display 수식으로 끝나며 실제 렌더 하단은 `437.4px`였다.
  - `문8）` 진입 로그는 `prev_ls=5669`, `end_y=480.29`, `compact_new_note=true`였다.
  - 기존 공통 분기는 display 수식 뒤 제목을 당길 때 실제 콘텐츠 하단 대신 `y_offset - prev_ls`를 사용해 약 `371.8px`를 기준으로 삼았다.
  - Stage2의 `layout.rs` 후단 보정이 이를 다시 `447.4px`로 내렸기 때문에 결과는 맞았지만, 공통 미주 간격 판단 바깥에서 보정되는 구조였다.
- 첫 Stage3 보정은 실제 콘텐츠 하단 + `10px` 겹침 방지였으나, 작업지시자 시각 검증에서 한컴과 다른 위치로 확인됐다. 한컴 PDF bbox 기준 `문8）`은 `384.678pt = 513.0px`에 있고, 직전 왼쪽 단 콘텐츠 하단은 약 `328.55pt = 438.1px`라서 실제 gap은 `20mm`에 해당한다.

## 수정

- `HeightCursor`에 렌더러가 기록한 직전 항목 실제 콘텐츠 하단(`prev_item_content_bottom_y`)을 입력으로 추가했다.
- `layout.rs`는 각 항목 진입 전 `last_item_content_bottom`을 `HeightCursor`에 전달만 한다.
- Stage2의 `layout.rs` 후단 겹침 방지 클램프와 보조 판정은 제거했다.
- compact 미주 display 수식 뒤 새 문항 제목 분기는 렌더 하단이 있으면 그 값에 `endnote_between_notes_hu`를 더해 공통 `미주 사이` 간격을 적용한다. `endnote_between_notes_hu`가 없으면 기존 `10px` 폴백을 유지한다.
- 공통 분기 내부에서 결과 y가 저장 vpos 위치보다 아래로 내려가는 경우에도 활성 vpos base를 함께 이동해 후속 미주 문단이 같은 기준을 따른다.

## 검증

- `cargo test --lib height_cursor -- --nocapture` 통과: 33개 테스트.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과: 45개 테스트.
- `cargo fmt -- --check` 통과.
- `git diff --check` 통과.
- 디버그 로그 확인:
  - `pi=522 y_out=437.4`
  - `pi=523 compact_new_note=true`
  - `pi=523 y_in=513.0`
  - `pi=524 base=103950`
  - `pi=532 y_in=792.9`
- 한컴 PDF bbox 비교:
  - `문8）`: `384.678pt * 96/72 = 512.9px`
  - `문9）`: `594.678pt * 96/72 = 792.9px`
- `미주사이20` p10 SVG/PNG 생성:
  - `output/task1261_stage3_page10_gap20/3-09월_교육_통합_2024-미주사이20_010.svg`
  - `output/task1261_stage3_page10_gap20_compare/rhwp_page10.png`
- `구분선아래20` p10 SVG 생성:
  - `output/task1261_stage3_below20_page10_gap_common/3-09월_교육_통합_2024-구분선아래20_010.svg`

## 시각 검증 대기

- Codex 자동 렌더 PNG와 한컴 PDF 96dpi 이미지 비교에서 `미주사이20` p10 `문8）`/`문9）` 위치가 한컴 bbox 좌표와 정합한다.
- WASM 빌드와 Studio 시각 검증은 작업지시자가 수동으로 진행한다.
