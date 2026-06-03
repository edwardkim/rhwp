# Task M100 #1209 Stage 4

## 목적

`3-09월_교육_통합_2022.hwp`의 10쪽 오른쪽 단 `문12)` 배치가 한컴오피스/PDF 기준과 다른 문제를 해결한다.
추가로 작업지시자가 13쪽 `문19)`의 그래프 주변 문단 겹침도 같은 파일의 후속 보정 범위로 지적했으므로 함께 확인한다.

## 시작 기준

- 이슈: [#1209](https://github.com/edwardkim/rhwp/issues/1209)
- 작업 브랜치: `local/task_m100_1209`
- 선행 커밋: `fb6b358b` (`task 1209: Stage3 2024 미주 모양 공통 처리`)
- 대상 HWP: `samples/3-09월_교육_통합_2022.hwp`
- 대상 PDF: `pdf/3-09월_교육_통합_2022.pdf`
- 대상 페이지: 10쪽, 13쪽

## 확인 질문

1. `문12)` 제목과 본문/수식의 실제 PDF 기준 y 위치가 현재 RHWP와 얼마나 다른가?
2. 차이가 미주 사이/구분선 아래 공통 보정의 회귀인지, 이전부터 있던 split/rewind 배치 문제인지 판단한다.
3. 문항별 예외 없이 기존 미주 pagination/lineSeg 공통 로직으로 보정 가능한지 확인한다.

## 진행 계획

1. 10쪽 RHWP SVG를 내보내고 `rsvg-convert`로 PNG를 생성한다.
2. 기준 PDF 10쪽을 `pdftoppm`으로 PNG 변환한다.
3. `dump-pages`와 render tree 좌표로 `문12)` 주변 paragraph 배치를 확인한다.
4. 원인을 좁힌 뒤 공통 로직으로 수정하고, `tests/issue_1139_inline_picture_duplicate.rs`에 회귀 테스트를 추가한다.
5. 수정 후 대상 테스트 파일만 검증한다.

## 현재 상태

- 2026-06-01: 작업지시자가 Stage3 커밋 후 `3-09월_교육_통합_2022.hwp` 10쪽 `문12)` 배치 문제 해결을 지시했다.
- 2026-06-01: 10쪽 `문12)` 비교 중 13쪽 `문19)` 그래프 주변 문단 겹침이 추가로 보고되었다.
- 2026-06-01: 13쪽 덤프 기준 `pi=728`은 비-TAC `Square` 그림이고 후속 `pi=729`, `pi=730`이 그림 옆 좁은 `LINE_SEG` 영역을 따라야 한다. 본문 흐름에는 `wrap_anchors` 등록 로직이 있으나, 미주 가상 문단 흐름에는 같은 등록이 없어 후속 문단이 전체 단 폭으로 조판되는 것으로 판단했다.
- 2026-06-01: 미주 가상 문단에도 비-TAC `Square` 그림 anchor를 등록하도록 공통 helper를 추가했다. `pi=729`, `pi=730`의 첫 줄 bbox가 그래프 bbox를 침범하지 않는 회귀 테스트를 추가했다.
- 2026-06-01: 작업지시자 재확인으로 13쪽 `문19)`에서 `f(2)=48-32-48=-32` 수식과 다음 `를 갖는다.` 문단이 여전히 세로로 겹치는 것을 확인했다.
- 2026-06-01: 원인은 `Square` 그림 host의 공백 줄을 0높이로 처리하는 공통 skip 조건이 같은 줄의 TAC 수식까지 빈 guide 줄로 오판한 것이다. `typeset`과 `paragraph_layout` 모두에서 해당 줄에 TAC 수식/개체가 있으면 정상 line advance를 보존하도록 조건을 좁혔다.
- 2026-06-01: 회귀 테스트를 `문19)` 그래프 옆 문단의 가로 wrap뿐 아니라 꼬리 수식 bbox 하단과 다음 문단의 세로 비겹침까지 검증하도록 보강했다.
- 2026-06-01: `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과(38 passed). `cargo fmt --all --check`, `git diff --check`, `wasm-pack build --target web --out-dir pkg` 통과.
- 2026-06-01: 10쪽 `문12)`은 RHWP/PDF 산출물 비교상 현재 큰 겹침은 보이지 않아 별도 문항 예외 없이 유지했다. 이번 코드 변경은 13쪽 `문19)` 그래프 주변 `Square` 둘러싸기 공통 처리에 한정한다.
- 2026-06-01: `rsvg-convert` 산출물 갱신:
  - `output/task1209_stage4_page10/rhwp_page10.png`
  - `output/task1209_stage4_page10/pdf_page10-10.png`
  - `output/task1209_stage4_page13/rhwp_page13.png`
  - `output/task1209_stage4_page13/pdf_page13-13.png`
