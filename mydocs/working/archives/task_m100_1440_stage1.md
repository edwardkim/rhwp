# Task 1440 Stage 1 - 온새미로 35쪽 어울림 LineSeg 진단

## 목적

- `samples/[2027] 온새미로 1 본교재.hwp` 35쪽에서 한컴 PDF와 달리 본문이 그림 영역을 침범하는 원인을 찾는다.
- HWP5 `LineSeg.column_start`/`segment_width`가 파싱되어도 `typeset`/`paragraph_layout`에서 누락되는지 확인한다.

## 승인

- 작업지시자 승인 완료.
- 사용자 제공 `samples/[2027] 온새미로 1 본교재.hwpx`, `pdf/[2027] 온새미로 1 본교재-2024.pdf`는 이번 PR 포함 대상으로 유지한다.

## 진행 계획

1. 35쪽 render tree에서 그림 bbox와 같은 y 대역 TextLine bbox를 수집한다.
2. 해당 TextLine의 `para_index`, `line_idx`, `para_vpos`와 원문 `line_segs`를 비교한다.
3. `wrap_anchors` 등록 여부와 `paragraph_layout`의 cs/sw 적용 조건을 확인한다.
4. 최소 수정 후 회귀 테스트와 시각 자료를 갱신한다.

## 진단 결과

- 대상 그림 호스트는 `section=3, para=7`이며 `wrap=Square`, `treat_as_char=false`이다.
- 바로 다음 본문 문단 `section=3, para=8`은 첫 7줄에 `cs=850`, `sw=20999` LineSeg를 가지고 있다.
- 기존 렌더는 이 LineSeg를 적용하지 않아 `pi=8` TextLine bbox가 그림 bbox와 수평/수직으로 겹쳤다.
- 원인은 `paragraph_layout`의 `effective_col_x/effective_col_w` 적용이 같은 문단 그림 또는 TAC 표에만 묶여 있어, 후속 body 문단에 저장된 precomputed wrap zone이 무시되는 것이었다.

## 수정

- `src/renderer/layout/paragraph_layout.rs`
  - 셀 내부가 아닌 body 문단에서 현재 줄의 `LineSeg`가 단 폭보다 확연히 좁은 wrap zone을 보존하면 `segment_width`/`column_start`를 적용하도록 했다.
  - 정상 들여쓰기 회귀를 막기 위해 `column_start + segment_width ~= col_width`인 줄은 새 경로에서 제외했다.
- `tests/issue_1440_onsamiro_picture_wrap.rs`
  - HWP/HWPX 양쪽 35쪽에서 대상 그림 bbox와 `pi=8` 본문 줄 bbox가 겹치지 않음을 검증한다.
  - HWP/HWPX 양쪽 원본 문단 `s3/p8`의 첫 7줄이 `cs=850`, `sw=20999` wrap LineSeg를 보존하는지 확인한다.

## 검증

- `cargo test --test issue_1440_onsamiro_picture_wrap`
  - 결과: 성공, 2 passed.

## 시각 자료

- 한컴 PDF: `mydocs/report/assets/task_m100_1440/hancom_pdf/page-35.png`
- rhwp HWP 렌더: `mydocs/report/assets/task_m100_1440/rhwp_hwp_p35.png`
- rhwp HWPX 렌더: `mydocs/report/assets/task_m100_1440/rhwp_hwpx_p35.png`
- SVG 원본:
  - `mydocs/report/assets/task_m100_1440/rhwp_svg/hwp/[2027] 온새미로 1 본교재_035.svg`
  - `mydocs/report/assets/task_m100_1440/rhwp_svg/hwpx/[2027] 온새미로 1 본교재_035.svg`
