# Task M100 #1189 Stage 2

## 목적

Stage1에서 확인한 `3-09월_교육_통합_2023.hwp` 19쪽 미주 흐름 밀림과 후속으로 확인된 `3-11월_실전_통합_2022.hwp` 10~12쪽/17쪽 미주 분배 어긋남을 개선한다.

## 시작 기준

- 이슈: [#1189](https://github.com/edwardkim/rhwp/issues/1189)
- 작업 브랜치: `local/task_m100_1189`
- Stage1 판단: 19쪽 우측 단에서 `문29)` 이하가 PDF보다 아래로 밀리고, `pi=952/953`이 페이지 하단 overflow.
- 우선 조사 대상: 미주 흐름의 VPOS 되감김, TAC 그림 주변 gap, partial paragraph 분할.

## 구현 가설

1. 미주 흐름은 일반 본문과 달리 같은 단 안에서도 LINE_SEG VPOS가 크게 되감긴다.
2. 현재 보정은 일부 하단 overflow를 막지만, `문29)` 앞의 TAC 그림/문단 조합에서 누적 높이를 과대 보존하는 구간이 남아 있다.
3. `pi=935`, `pi=951`, `pi=952`, `pi=953` 주변의 VPOS anchor 선택과 picture-only paragraph 처리 조건을 조정하면 19쪽 단1 하단 overflow를 줄일 수 있다.
4. `3-11월_실전_통합_2022.hwp` 16쪽 끝의 `pi=786`은 내부 LINE_SEG 되감기 직전 줄이 큰 수식 줄이다. 일반 텍스트 되감기보다 내부 split 신호를 우선해야 17쪽 첫머리 수식 이어짐이 한컴/PDF와 맞는다.
5. `3-11월_실전_통합_2022.hwp` 12쪽 하단 수식은 렌더 bbox 높이를 식 자체에 Y축 스케일로 적용해 글자가 세로로 찌그러지는 문제가 있다. bbox 높이는 줄 높이와 여백을 포함한 배치 영역으로 보고, 식 SVG/Canvas 렌더에는 X축 스케일만 적용한다.

## 진행 계획

1. `src/renderer/height_cursor.rs`의 compact endnote 보정 조건을 확인한다.
2. `dump-pages`와 debug SVG로 `pi=935`, `pi=951`, `pi=952`, `pi=953`의 y 흐름을 비교한다.
3. 최소 범위 수정 후 19쪽 SVG/PDF 비교와 기존 task 1139 회귀 테스트를 확인한다.
4. 개선 범위가 19쪽에만 국한되지 않는지 18~20쪽 dump와 기존 12/13/23쪽 회귀 케이스를 재확인한다.

## 현재 상태

- 2026-05-31: 작업지시자가 개선 시작을 승인했다.
- 2026-05-31: `3-09월_교육_통합_2023.hwp` 19쪽 보정 중, 작업지시자가 `3-11월_실전_통합_2022.hwp` 17쪽도 같은 후속 보정 범위로 확인 후 수정하도록 지시했다.
- 2026-05-31: 작업지시자가 중간 탐색 중 전체 `cargo test` 실행은 중단하고, 전체 테스트는 커밋 직전 검증 단계에서만 수행하도록 지시했다.
- 2026-05-31: 작업지시자가 수정 직후 검증에는 `tests/issue_1139_inline_picture_duplicate.rs` 단일 통합 테스트를 수행해야 한다고 지시했다.
- 2026-05-31: `3-11월_실전_통합_2022.hwp` 16/17쪽은 `rsvg-convert` 변환 PNG와 PDF 추출본으로 비교했다. 수정 전 `pi=786 lines=0..2`가 16쪽에 남아 overflow했으나, 수정 후 `16쪽 lines=0..1`, `17쪽 lines=1..5`로 분배되어 PDF 기준 첫 수식 이어짐이 맞아졌다.
- 2026-05-31: `3-09월_교육_통합_2023.hwp` 19쪽은 `pi=935 lines=0..2 / 2..3`, `pi=953 lines=0..1` 분배를 유지해 Stage2 1차 보정 상태로 회복되는 것을 확인했다.
- 2026-05-31: 코드 보정은 두 축이다. `height_cursor.rs`는 compact 미주 질문 제목의 큰 forward gap을 하단 영역에서 cap하고, `typeset.rs`는 내부 되감기 앞줄이 큰 수식 줄(`line_height >= 2000`)일 때만 internal rewind split을 fit split보다 우선한다.
- 2026-05-31: 회귀 테스트에 `issue_1189_2022_nov_page17_internal_rewind_keeps_formula_tail_on_next_page`를 추가하고, 2023 19쪽 테스트에는 `pi=935` 분배 유지 조건을 추가했다.
- 2026-05-31: 작업지시자가 `3-11월_실전_통합_2022.hwp` 10~12쪽을 모두 확인하고 12쪽 마지막 수식 깨짐을 수정하라고 지시했다.
- 2026-05-31: 10~12쪽 최신 산출물은 `output/task1189_stage2_3-11_pages10_12/post4/`에 생성했다. PDF 추출은 `pdftoppm`, SVG→PNG 변환은 `rsvg-convert`를 사용했다.
- 2026-05-31: 12쪽은 `pi=553 lines=8..11` 뒤 `pi=554` 그래프와 `pi=555` 문15가 이어지도록 보정했다. 누락되던 `h(x)=x(x+2)` 시작부가 12쪽 상단에 복구됐다.
- 2026-05-31: 수식 렌더러는 `svg.rs`, `web_canvas.rs`, `paragraph_layout.rs`에서 수식 배치 높이를 식 자체의 Y축 스케일/베이스라인 스케일에 쓰지 않도록 수정했다. 12쪽 하단 `a_5` 관련 수식은 세로 찌그러짐이 사라졌다.
- 2026-05-31: 10쪽은 `pi=475` 7.6px overflow 로그가 남고, 11쪽은 `pi=553 lines=0..8`에서 최대 55.7px overflow 로그가 남는다. 12쪽 누락/수식 깨짐은 개선됐지만, 11쪽 하단 overflow는 잔여 보정 후보로 기록한다.
- 2026-05-31: 작업지시자가 `3-11월_실전_통합_2022.hwp` 14쪽과 17쪽 문28을 추가 확인하라고 지시했다. 최신 비교 산출물은 `output/task1189_stage2_3-11_pages14_17/post2/`에 생성했다. SVG→PNG 변환은 `rsvg-convert`, PDF 추출본은 기존 `pdftoppm` 산출물을 기준으로 비교했다.
- 2026-05-31: 17쪽 문28은 HWP3식 미주 수식 문단(`줄바꿈/탭 + TAC 수식`)에 HWP5 누락 marker 합성 보정이 적용되어 `g(θ)=□CQRT-△CST`가 뒤쪽 줄로 밀리는 문제였다. `composer.rs`에서 본문 텍스트가 줄바꿈/탭/공백뿐이고 컨트롤이 TAC 개체뿐인 문단은 marker 합성을 건너뛰도록 좁혔다.
- 2026-05-31: 수정 후 17쪽 문28의 `g(θ)=□CQRT-△CST`가 같은 줄로 돌아왔고, 뒤쪽 긴 수식도 PDF처럼 여러 줄로 내려오는 것을 확인했다. 14쪽은 문22~문27 시작 흐름이 유지되는 것을 확인하고 회귀 테스트 조건에 추가했다.
- 2026-05-31: `cargo fmt --all --check`와 `git diff --check`는 통과했다.
- 2026-05-31: 14/17쪽 보정 후 `cargo fmt --all --check`, `git diff --check`, `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과(31 passed, 0 failed). 출력에 2023 19쪽 잔여 `LAYOUT_OVERFLOW` 로그 2건(`pi=935`, `pi=953`)이 남지만 회귀 테스트 조건은 통과했다. 전체 `cargo test`는 커밋 전 검증으로 남겨 둔다.
- 2026-05-31: 작업지시자가 12쪽 문19) 변화표 내부 수식이 한컴과 다르다고 추가 지적했다. 원인은 HWP 수식 토큰 `SEARROW`/`NEARROW`가 화살표 기호로 매핑되지 않아 문자열 그대로 좁게 렌더되는 문제였다.
- 2026-05-31: `src/renderer/equation/symbols.rs`에 HWP 대문자 대각 화살표 토큰(`NWARROW`, `NEARROW`, `SWARROW`, `SEARROW`)을 추가했다. 문19) 회귀 테스트에는 `SEARROW`/`NEARROW` 문자열이 SVG에 남지 않고 `↘`/`↗`가 렌더되는 조건을 추가했다.
- 2026-05-31: 추가 보정 검증은 `cargo test --lib test_arrows --quiet`와 `cargo test --test issue_1139_inline_picture_duplicate issue_1189_2022_nov_pages10_12_rewind_tail_and_equation_scale_match_pdf -- --nocapture` 통과. 최신 12쪽 산출물은 `output/task1189_stage2_3-11_pages10_12/post5/page12/`에 생성했고, SVG→PNG 변환은 `rsvg-convert`를 사용했다.
- 2026-05-31: 문19) 추가 보정 후 `cargo fmt --all --check`, `git diff --check`, `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과(31 passed, 0 failed). 브라우저 확인용 `wasm-pack build --target web --out-dir pkg`도 통과했다.
- 2026-05-31: 작업지시자가 `3-11월_실전_통합_2022.hwp` 11쪽 overflow가 아직 남아 있음을 시각 확인했고, 현 Stage2 변경분은 커밋한 뒤 11쪽 overflow를 새 Stage3에서 처리하라고 지시했다.
- 2026-05-31: 커밋 전 전체 검증으로 `cargo test --tests` 통과. Stage2는 12쪽/17쪽/19쪽 보정과 11쪽 overflow 잔여 기록을 포함해 커밋 대상으로 확정한다.
