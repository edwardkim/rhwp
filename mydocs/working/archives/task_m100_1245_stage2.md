# Task M100 #1245 Stage2 — 10쪽 문12 우측 수식 배치 보정

## 작업 지시

- 2026-06-02: 작업지시자가 Stage1 시각 판단을 확인하고 커밋을 지시했다.
- Stage1 커밋: `ba8080a6` (`task 1245: Stage1 어울림 그림 상대 vpos 보정`)
- 이후 `3-09월_교육_통합_2022.hwp` 10쪽 `문12)` 우측 단 수식 배치가 한컴/PDF 기준과 다르게 보이는 문제를 새 스테이지에서 해결하라고 지시했다.

## 대상

- 문서: `samples/3-09월_교육_통합_2022.hwp`
- 페이지: 10쪽 / 23쪽
- 위치: 우측 단 `문12)` 본문과 수식 블록
- 증상: 우측 단 하단의 긴 수식/분수식이 단 경계 또는 페이지 외곽선 기준과 다르게 배치된다.

## 초기 확인 계획

1. `dump-pages`로 10쪽 우측 단의 `문12)` 관련 page item과 문단 번호를 확인한다.
2. `dump`로 `문12)` 관련 문단의 `LINE_SEG`, 수식 컨트롤 크기, paragraph style을 확인한다.
3. `export-svg` + `rsvg-convert`로 10쪽 수정 전 비교 PNG를 생성한다.
4. `pdf/3-09월_교육_통합_2022.pdf` 10쪽과 RHWP 결과를 비교한다.
5. 원인이 수식 bbox 폭/정렬, line segment 폭, column clipping, 또는 수식 line-height 중 어디인지 분리한다.

## 승인 상태

작업지시자가 Stage2 진행을 지시했으므로 분석 후 필요한 최소 소스 수정과 회귀 가드를 진행한다.

## 분석 결과

- `dump-pages samples/3-09월_교육_통합_2022.hwp -p 9` 기준 10쪽 우측 단 `문12)`는 `pi=567..574`, `문13)`은 `pi=575`로 배치된다.
- 문제 수식 블록은 `pi=574`의 텍스트가 없는 TAC 수식-only 문단이다.
- 수정 전 SVG에서 `따라서` 텍스트는 `x=402.52`에서 시작하지만, 바로 뒤 첫 `lim` 수식은 `x=442.88`에서 시작했다.
- 원인은 `paragraph_layout.rs`의 빈 runs + TAC 수식 fallback이 본문/미주 수식-only 문단에도 `effective_margin_left`와 alignment offset을 다시 적용한 것이다.
- Task #490에서 필요한 셀 내부 수식 alignment 보정은 유지해야 하므로, 셀 문단과 일반 본문/미주 문단을 분리해 처리한다.
- 작업지시자가 x 보정 뒤에도 `따라서`와 수식 사이 간격이 한컴보다 크다고 지적했다.
- `RHWP_DEBUG_PARA_TAC=1` 확인 결과 `pi=574`는 선행 `LINE_SEG`가 `char_start=0, char_end=0`, TAC 없음인 퇴화 안내 줄이고, 실제 첫 수식은 다음 줄 `char_start=0..1`에 매핑되어 있었다. 이 안내 줄 높이 때문에 첫 수식이 `y=610.09`로 밀렸다.
- 한컴은 이 수식-only 미주 문단의 선행 안내 줄을 첫 수식 앞 세로 예약으로 쓰지 않으므로, 다음 줄과 `char_start`가 같고 현재 줄에는 TAC가 없으며 다음 줄에 수식 TAC가 있는 빈 run 줄은 안내 줄로 취급한다.

## 수정 내용

- `src/renderer/layout/paragraph_layout.rs`
  - 셀 내부 수식-only 문단은 기존처럼 paragraph alignment와 margin을 적용한다.
  - 본문/미주 수식-only 문단은 저장된 LINE_SEG 흐름을 따르도록 `effective_col_x` 기준으로 배치한다.
  - 수식-only 문단의 선행 퇴화 안내 `LINE_SEG`를 기준 vpos와 y advance에서 제외해 첫 수식이 한컴처럼 `따라서` 바로 뒤에 붙도록 했다.
- `tests/issue_1139_inline_picture_duplicate.rs`
  - `issue_1209_2022_sep_page10_question12_uses_safe_vpos_backtrack`에 `문12)` 첫 수식 x 위치 회귀 검증을 추가했다.
  - 같은 테스트에 `따라서`와 첫 수식 사이 y 간격이 20px 이하로 유지되는 회귀 검증을 추가했다.
  - 주석 문단의 render node `para_index`가 원 문단 번호와 그대로 대응하지 않는 경우가 있어, 수식 SVG 내용과 y 범위로 대상 수식을 찾도록 했다.

## 검증

- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 42개 통과
- `cargo fmt --all -- --check`
  - 통과
- SVG/PNG 확인
  - `output/task1245_stage2_after/3-09월_교육_통합_2022_010.svg`
  - `output/task1245_stage2_after/3-09월_교육_통합_2022_010.png`
  - 첫 `lim` 수식 시작 x가 `442.88`에서 `402.52`로 이동해 우측 단 왼쪽 흐름과 정렬됨.
  - `따라서` y=`564.88`, 첫 `lim` 수식 y=`572.71`로 조정되어 한컴/PDF처럼 간격이 촘촘해짐.
