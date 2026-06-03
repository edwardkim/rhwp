# Task M100 #1209 Stage 6

## 목적

Stage5 커밋 이후 남은 시각 정합 문제를 별도 단계로 분리한다.
`3-09월_교육_통합_2022.hwp` 10쪽에서 한컴/PDF 기준과 RHWP 렌더링이 다르게 보이는 문단/수식 배치 문제를 분석하고 보정한다.

## 시작 기준

- 이슈: [#1209](https://github.com/edwardkim/rhwp/issues/1209)
- 작업 브랜치: `local/task_m100_1209`
- 선행 커밋: `e862c8c0` (`task 1209: Stage5 어울림 그림 줄 위치 보정`)
- 대상 HWP: `samples/3-09월_교육_통합_2022.hwp`
- 대상 PDF: `pdf/3-09월_교육_통합_2022.pdf`
- 대상 페이지: 10쪽
- 주요 관찰 지점: `문9)` 하단부터 `문13)`까지의 양단 문단/수식 배치

## 이전 단계에서 이관한 항목

- Stage5에서는 8쪽 `문29)`의 `Square/어울림` 그림을 `LINE_SEG`의 첫 좁은 줄 위치에 맞추는 문제를 해결했다.
- Stage5에서 확인한 공통 원칙은 유지한다. HWP5 CommonObjAttr의 `Square/어울림`과 `TopAndBottom/자리차지`는 뒤집지 않고, 본문 회피/세로 흐름은 저장된 `LINE_SEG` 정보를 우선 기준으로 삼는다.
- Stage6에서는 같은 원칙이 수식이 포함된 일반 문단 흐름에서도 어긋나는지 확인한다.

## 신규 관찰

- 2026-06-01: 작업지시자가 `3-09월_교육_통합_2022.hwp` 10쪽 캡처를 제공했다. RHWP 화면과 한컴/PDF 기준에서 페이지 내 문단 위치 및 `문12)`, `문13)` 주변 수식 배치가 다르게 보인다.
- 첨부 캡처 기준으로 오른쪽 단의 `문12)` 하단 수식 묶음과 `문13)` 시작 위치, 왼쪽 단의 `문9)` 그래프 이후 `문10)` 위치를 우선 비교한다.

## 확인 질문

1. 10쪽 차이가 페이지/단 전체 높이 누적 오차인지, 특정 수식 line box 높이 산정 문제인지 구분한다.
2. HWP `LINE_SEG`의 `vertical_pos`, `height`, `spacing`이 저장한 위치와 RHWP paragraph layout 결과가 어디서 벌어지는지 확인한다.
3. Stage5의 `LINE_SEG` 기반 보정이 일반 문단/수식 배치에 영향을 주었는지 확인한다.
4. 수정이 필요하면 개별 문항 예외가 아니라 수식/문단 높이 공통 로직으로 처리한다.
5. 현재 `HeightCursor`/미주 흐름에 누적된 이슈별 가드가 구조적 공통 정책으로 치환 가능한지 검토한다.

## 공통화 방향

현재 문제는 문항별 예외가 아니라 `LINE_SEG`와 실제 렌더 높이 사이의 조율 정책 문제로 본다.
공통 로직은 다음 입력만 보고 판단하도록 정리한다.

- 저장 흐름: 직전/현재 `LINE_SEG.vertical_pos`, `line_height`, `line_spacing`, rewind/reset 여부
- 렌더 흐름: 현재 sequential y, 직전 visible content bottom, 단 하단까지 남은 공간
- 구조 신호: 미주 compact flow, 새 미주 제목, tall inline 수식, TAC 개체, `Square/TopAndBottom` wrap
- 안전 조건: 목표 y가 직전 visible content를 침범하지 않는가, 단 하단 overflow를 줄이는가, 저장된 미주 사이 간격을 훼손하지 않는가

따라서 Stage6에서는 `문12)`만 맞추는 추가 if를 넣지 않고, `HeightCursor`의 backtrack/forward suppression을 “저장 VPOS 목표와 visible content guard” 중심의 공통 함수로 정리할 수 있는지 먼저 확인한다.

## 진행 계획

1. 10쪽을 SVG로 내보내고 `rsvg-convert`로 PNG를 생성한다.
2. `pdf/3-09월_교육_통합_2022.pdf` 10쪽을 PNG로 변환해 동일 배율로 비교한다.
3. `문12)` 및 `문13)` 주변 paragraph index와 `LINE_SEG`를 덤프한다.
4. 수식 bbox, line height, paragraph advance 중 실제 차이를 만드는 지점을 좁힌다.
5. 수정 후 `tests/issue_1139_inline_picture_duplicate.rs`에 대상 페이지 회귀 조건을 추가한다.

## 현재 상태

- 2026-06-01: Stage5 변경분을 커밋한 뒤 새 스테이지 문서를 생성했다.
- 2026-06-01: 10쪽 SVG/PDF 비교와 `RHWP_VPOS_DEBUG` 덤프를 확인했다.
- 2026-06-01: 오른쪽 단 `문12)` 제목(`pi=567`)이 저장 `LINE_SEG`의 안전한 되감김 위치보다 아래에 남아 수식 블록과 `문13)` 시작 위치가 PDF보다 늦어지는 것을 확인했다.
- 2026-06-01: 단, 같은 유형의 큰 `line_spacing`은 3-10월 11쪽 `문20)`, 3-11월 14쪽 `문23)`처럼 단 하단 미주 사이 간격을 보존해야 하는 곳에도 존재한다. 따라서 문항별 예외가 아니라 단 중간부에서만 저장 VPOS 되감김을 허용하는 공통 가드로 제한했다.

## 구현 결과

- `HeightCursor::vpos_adjust`에 `compact_endnote_safe_vpos_backtrack` 공통 정책을 추가했다.
- 조건은 다음과 같다.
  - compact 미주 흐름(`suppress_large_forward_jump`)일 것
  - 현재 VPOS rewind 자체가 아닐 것
  - 저장 VPOS 목표가 현재 sequential y보다 위에 있되, 직전 visible content bottom을 침범하지 않을 것
  - 단 하단부가 아니라 단 중간부(`col_area_y + col_area_height * 0.75` 이하)일 것
- 이로써 `문12)`처럼 저장된 mid-column VPOS를 따라야 하는 경우만 되감기고, 단 하단의 미주 사이 간격은 기존 Stage1/Stage2 공통 gap 정책이 계속 담당한다.

## 검증

- `cargo fmt --all --check` 통과.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과: 41 passed, 0 failed.
- `wasm-pack build --target web --out-dir pkg` 통과.
- 시각 비교 산출물:
  - RHWP: `output/task1209_stage6_page10_final/rhwp_page10.png`
  - PDF: `output/task1209_stage6_page10_final/pdf_page10.png`
- `rsvg-convert`를 사용해 SVG를 PNG로 변환했다.
