# Task M100 #1189 Stage 3

## 목적

Stage2 커밋 이후 남은 `3-11월_실전_통합_2022.hwp` 11쪽 overflow를 한컴 기준으로 재분석하고 보정한다.

## 시작 기준

- 이슈: [#1189](https://github.com/edwardkim/rhwp/issues/1189)
- 작업 브랜치: `local/task_m100_1189`
- 선행 커밋: `6cb2fd55` (`task 1189: 미주 수식 흐름 후속 보정`)
- 대상 문서: `samples/3-11월_실전_통합_2022.hwp`
- 대상 페이지: 11쪽
- 사용자 시각 판정: 11쪽 하단에서 미주 문단/수식이 한컴보다 아래로 overflow 된다.

## 초기 판단

1. Stage2에서는 12쪽 시작 누락, 12쪽 문19) 변화표 화살표, 17쪽 문28 수식 줄 밀림을 보정했다.
2. Stage2 검증 로그에도 `pi=553 lines=0..8` 주변 11쪽 overflow가 남아 있었고, 작업지시자가 실제 화면으로 이를 확인했다.
3. Stage3 재분석 결과 실제 overflow 지점은 오른쪽 단 `문14)` 꼬리 흐름(`pi=553`)이며, 새 미주 제목(`문13`/`문14`)이 직전 큰 수식 줄의 trailing 간격 뒤로 과도하게 밀린 것이었다.
4. Stage3에서는 11쪽 하단 `문14)` 흐름을 PDF/한컴 기준과 다시 비교하고, Stage2에서 도입한 internal rewind split/수식 높이 보정과 충돌하지 않는 좁은 조건을 찾는다.

## 진행 계획

1. 11쪽 PDF 기준 이미지와 현재 rhwp SVG/PNG를 같은 배율로 다시 생성한다.
2. `dump-pages -p 10` 기준 `pi=553` 분할 범위와 하단 overflow 지점을 확인한다.
3. 10~12쪽 전체 흐름이 유지되는 최소 보정 조건을 찾는다.
4. 수정 후 `tests/issue_1139_inline_picture_duplicate.rs` 단일 테스트와 10~12쪽 시각 산출물을 재검증한다.

## 현재 상태

- 2026-05-31: 작업지시자가 Stage2 커밋 후 11쪽 overflow를 새 스테이지에서 처리하라고 지시했다.
- 2026-05-31: `output/task1189_stage3_3-11_page11/current/`에 11쪽 비교 산출물을 생성했다. 수정 전 로그는 `pi=553` line 5~7에서 최대 55.7px 하단 overflow.
- 2026-05-31: PDF bbox 기준 현재 `문13`/`문14` 제목 baseline이 한컴보다 약 57px 낮음을 확인했다. lazy-base compact 미주 흐름에서 큰 display 수식 줄 뒤 새 문제 제목만 직전 내용 하단 + 10px로 붙도록 보정했다. page-base 흐름은 기존 7mm 미주 간격을 유지하도록 제외했다.
- 2026-05-31: `output/task1189_stage3_3-11_page11/fixed/`에 수정 후 11쪽 SVG/PNG/dump를 생성했다. `export-svg` 로그에서 `LAYOUT_OVERFLOW_DRAW`가 사라졌고, `문13` baseline 412.77px / `문14` baseline 628.56px로 PDF 환산 좌표와 정합.
- 2026-06-01: 작업지시자가 Stage3 결과를 승인했다. 커밋 전 전체 테스트를 진행한다.

## 검증 기록

- `cargo test --test issue_1139_inline_picture_duplicate issue_1139_page12_question15_keeps_hancom_endnote_gap -- --nocapture` 통과.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과(31 passed).
- `cargo test --lib compact_endnote_question_title_after_tall_line_uses_content_bottom_gap` 통과.
- `wasm-pack build --target web --out-dir pkg` 통과.
- `cargo test --tests` 통과.
- `rsvg-convert` 기준 수정 후 11쪽 PNG: `output/task1189_stage3_3-11_page11/fixed/page11/rhwp_page11.png`.
