# Task M100 #1189 Stage 5

## 목적

Stage4 커밋 이후 `3-11월_실전_통합_2022.hwp` 10쪽 하단 미주 영역에서 문단이 겹쳐 보이는 문제를 한컴오피스 기준으로 재분석하고 보정한다.

## 시작 기준

- 이슈: [#1189](https://github.com/edwardkim/rhwp/issues/1189)
- 작업 브랜치: `local/task_m100_1189`
- 선행 커밋: `71c02d4e` (`task 1189: 17쪽 미주 드래그 선택 보정`)
- 대상 문서: `samples/3-11월_실전_통합_2022.hwp`
- 대상 페이지: 10쪽
- 사용자 시각 판정: 10쪽 하단 왼쪽 단 `문6)` 부근에서 텍스트가 겹치거나 페이지 하단에 눌려 보인다.

## 초기 판단

1. `dump-pages -p 9` 기준 10쪽 왼쪽 단 끝은 `pi=475`이고, Stage2 잔여 기록의 10쪽 `pi=475` overflow와 같은 지점으로 보인다.
2. 같은 문서 11쪽/12쪽은 Stage2/Stage3에서 수식 줄 되감기와 미주 제목 간격을 보정했으므로, 이번 수정은 10쪽 하단 왼쪽 단의 남은 overflow만 좁게 다룬다.
3. 10쪽 미주 흐름은 page-base 미주 흐름의 첫 페이지에 가까워, lazy-base compact 미주 제목 보정과 충돌하지 않도록 확인한다.
4. 재확인 결과 `pi=475`는 빈 줄이 아니라 `text="" + controls=1`인 수식 전용 미주 문단이었다. 숨김 처리가 아니라 직전 새 문제 제목(`문6)`)의 하단 간격을 줄여 꼬리 수식이 본문 하단 안에 들어오게 해야 한다.

## 진행 계획

1. 10쪽/11쪽 `dump-pages`와 SVG 내보내기 로그로 실제 overflow 문단과 줄 범위를 확정한다.
2. `pi=475` 주변 LINE_SEG/수식/문단 높이를 확인해 겹침 원인이 줄 내부 높이 부족인지, 다음 단 시작 위치 계산 문제인지 분리한다.
3. 가장 좁은 조건으로 레이아웃 보정을 적용하고, 10~12쪽 기존 회귀 테스트가 유지되는지 확인한다.
4. 수정 후 `tests/issue_1139_inline_picture_duplicate.rs` 단일 테스트와 필요한 단일 케이스를 재검증한다.

## 현재 상태

- 2026-06-01: 작업지시자가 10쪽 하단 미주 오버랩을 보고했다. Stage5 문서를 만들고 분석을 시작한다.
- 2026-06-01: `dump-pages -p 9`와 `export-svg -p 9` 로그 기준 오버랩 지점은 왼쪽 단 마지막 `pi=475` 한 줄이다. 현재 `LAYOUT_OVERFLOW_DRAW: section=0 pi=475 line=0 ... overflow=7.6px`가 발생한다.
- 2026-06-01: `RHWP_VPOS_DEBUG=1` 기준 `pi=470..475`는 lazy base 역산이 음수가 되어 VPOS 보정이 건너뛰어지고 있다. 그 결과 paginator의 fit 판정은 통과하지만 실제 렌더 line box는 body 하단을 넘는다.
- 2026-06-01: 보정 방향은 `pi=475`를 왼쪽 단 하단에 억지로 남기는 것이 아니라, compact 미주 다단 흐름에서 하단 safety margin을 적용해 다음 단 시작으로 넘기는 쪽으로 판단했다.
- 2026-06-01: `pi=475`를 다음 단으로 넘기면 오른쪽 단 시작 흐름이 달라질 수 있어, 실제 구현은 `HeightCursor`의 compact 미주 하단 새 문제 제목 간격 보정으로 좁혔다. lazy base 역산이 음수인 경우에도 하단 85% 이후의 새 문제 제목은 직전 내용 하단 + 10px로 제한한다.
- 2026-06-01: 수정 후 `export-svg -p 9` 로그에서 `LAYOUT_OVERFLOW_DRAW`가 사라졌다. `rsvg-convert` 산출물은 `output/task1189_stage5_3-11_page10/fixed/rhwp_page10.png`.

## 검증 기록

- `cargo run --quiet --bin rhwp -- export-svg samples/3-11월_실전_통합_2022.hwp -p 9 -o output/task1189_stage5_3-11_page10/fixed --show-control-codes --debug-overlay` 통과. `LAYOUT_OVERFLOW_DRAW` 없음.
- `rsvg-convert output/task1189_stage5_3-11_page10/fixed/3-11월_실전_통합_2022_010.svg -o output/task1189_stage5_3-11_page10/fixed/rhwp_page10.png` 통과.
- `cargo test --test issue_1139_inline_picture_duplicate issue_1189_2022_nov_pages10_12_rewind_tail_and_equation_scale_match_pdf -- --nocapture` 통과.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과(32 passed).
- `cargo fmt --all --check` 통과.
- `git diff --check` 통과.
- `cargo test --tests` 통과.
- `wasm-pack build --target web --out-dir pkg` 통과.
