# #7751 편람375쪽 서식 괘선 원점 — 5단계

2026-10-16, 시작 생산 HEAD `8effe3e8e1ea24b4b40ee32cf95c0618ff9a0f04`.
사용자가375쪽 overlay의 외곽선·내부 칸 어긋남을 지적했다. Native92.41998%는
시각 완료의 근거가 아니며, 해당 배치 결함을 먼저 보정한다.

## 독립 근거와 조사 범위

동일 편람 HWP(SHA `40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f`)와
383쪽 KoPub 한컴2024 Print(SHA `3cdf329004cffad0ebacd122586e5e1702aff36b3015cbcb401d038857019593`)를
유지한다.96dpi에서 PDF의 실제 path 좌표를 읽었다. 오른쪽 서식 바깥 표의
왼끝은526.509px, 내부 첫 칸은534.508px, 제목 상자는584.583px다.
현재 render tree는 각각523.8/531.8/582.0px다. 단순 raster 색상 차이가 아니라
상위 표 원점에서 자손까지 이어진 가로 이동이다. 왼쪽 서식도 별도로 대조한다.

오른쪽은 구역12·문단43의 가운데 정렬 TAC 표, 왼쪽은 문단18의 문단 기준
글앞 표다. 저장 LineSeg·개체 바깥여백·호스트 줄 정렬의 생산/소비와 최종
원점 덮어쓰기를 추적한다. 문서 ID나 관측된 오차를 상수로 더하지 않는다.

## 검증 계획

1. 실제 원본의 줄·컨트롤 순서와 원점을 추적하고, 독립 Print path에 따라
   위반된 일반 배치 계약을 판정한다. 합성 입력을 실제 저장본 근거로 쓰지 않는다.
2. 원인 계층을 보정한 뒤375쪽 Native PNG를 먼저 재산출해 같은 선들을 대조한다.
   같은 변경의 정렬·여백 반례와 기존 정상 경계를 검증한다.
3. 최종 fresh WASM375쪽 및 전383쪽 양쪽 TSV·직접 판독을 갱신한다.
   98% 미만 쪽의 px 핀은 추가하지 않으며 소유·보존·포함 관계를 보호한다.
   소스 변경 뒤 필수 회귀·lint 결과는 새 생산 소스에 맞게 갱신한다.

원시 증적은 ignored `output/pr-review/pr7494-20261016/` 아래의
`reference-mark-native-worst375`와 `source-p375-dump.log`에 보존한다.

## 호출 경로에서 확인한 원인

- 오른쪽 문단의 `control_text_positions=[1]`, 텍스트는 공백1개다. 공백은
  표 **앞**에 있다. FullParagraph가 없는 TAC의 `layout_table_control_block`
  폴백은 선행 폭6px를 구하지만, `center.max(base_x)`로 표만 가운데 정렬한다.
  기존 `stored_inline_control_pen`의 줄 전체 정렬→앞 텍스트 전진 결과를
  재사용하면 공백·표·좌우 바깥여백을 같은 줄에서 소비할 수 있다. 최종
  `flow_placement` override의 저장 pen 존재 여부도 함께 확인한다.
- 왼쪽은 문단 기준 LEFT/TOP 글앞 표이며 사방 바깥여백141HU다.
  `compute_table_x_position`의 Para 분기는 Square/T&B만 여백을 싣고,
  `compute_table_y_position`은 Paper/재설정 저장 앵커만 위여백을 싣는다.
  글앞/글뒤는 표시 층위이며 바깥 상자 기준 원점의 여백을 지울 이유가 없다.
  상위 표의 원점에서 여백을 한번 소비하고 자손은 같은 이동을 상속한다.
  비흐름 overlay의 본문 전진·쪽 분할은 바꾸지 않는다.

## 추가 지시: raw 지표로 판정

사용자가130쪽의 Native99.10104% 대비 raw79.94718% 차이와 내부 선 어긋남을
지적했다.90% 최소 gate와98% px 허용 조건을 모두 `silhouette_raw_match_percent`로
판정하도록 명시적으로 승인했다. 색상 경계 보정값은 진단용으로 보존한다.
지침과 실제 gate·TSV의 below_90·검토 이미지 표기를 같은 지표로 맞추고,
raw 미달/보정값 통과·정확히90·raw 누락·쪽수 불일치 반례를 검사한다.
기존 raw TSV는 방법을 바꾸지 않았으므로 재분류 근거로 쓰되, 원래 실행 당시의
gate 결과와 새 기준 판정을 구분한다. 최종 출력·원점 보정의 검증은 계속 갱신한다.

## 원점 보정의 중간 검증과 남은 조건

`layout_table_control_block`이 호스트 본문을 그리지 않는 TAC에도 기존
`stored_inline_control_pen`을 소비하게 했다. 이 helper의 줄 들여쓰기는
`paragraph_line_indent_for_source`와 공유하고, 유효한 저장 줄 영역에 한 번만
반영한다. 음수 문단 여백을0으로 자르던 가정도 제거했다. 첫 후보에서 하락한
60·67·116·243쪽은 이 공통 계산을 바로잡은 두 번째 후보에서 해소됐다.

글앞/글뒤 표의 Para/Column 왼쪽 앵커는 바깥여백 상자에서 괘선으로 한 번
전진한다. Para 세로 앵커와 기존 본문 상단 제한도 같은 바깥 상자 좌표계를
소비한다. 표시 층위로 본문 흐름을 전진시키거나 셀 크기를 바꾸지 않는다.

생산 버전은 `30e91bf8e`와 ignored `form-origin-v2-source.patch` 및
별도 보존한 `form-origin-v2-overlay_outer_frame.rs`의 조합이다.
`form-origin-v2-{provenance,wasm-provenance}.json`에 변경 소스별 SHA와
실행 파일·WASM SHA를 기록했다. 이후 `cargo fmt --all`만 적용했으며
아직 이 커밋의 최종 전체 회귀·lint 완료를 주장하지 않는다.

| 확인 범위 | Native raw | fresh WASM raw | 상태 |
| --- | ---: | ---: | --- |
| 375쪽 | 99.37858% | 99.42850% | 두 서식 외곽·제목·도장을 같은 배율로 직접 판독 |
| 전383쪽 | 최저79.94718%(130쪽) | 전체 실행 대기 | 현재 PR gate 보류 |

Native 전쪽 실행은257.99초, exit1이다. 산출 누락이 아니라130쪽 raw 미달을
정상 검출한 종료다. 같은 기준 PDF의 baseline과 비교하면130쪽
95.02336→79.94718%,224쪽99.14080→99.12204%,226쪽99.16137→99.14314%로,
이 세 쪽의 원인 확인이 남았다. 평균값이나 색상 경계 보정값으로 상쇄하지 않는다.

ignored `pending-form-origin-regressions.rs`의 진단5건은 두 번째 후보에
수동 링크해5 PASS였다. 같은 파일의 서식 바깥여백 검사는 수정 전 라이브러리에서
의도한 여백 assertion으로 FAIL(exit101)했다. 전체 raw 선행 조건 미충족이므로
새 검사를 아직 정식 `tests/cases/`에 추가하지 않았다. 이 실행은 전체 Rust
suite의 통과 증거가 아니다.

증적은 `output/pr-review/pr7494-20261016/` 아래
`form-origin-v2-native-all/handbook/silhouette.tsv`,
`form-origin-v2-native-comparison.json`, `form-origin-v2-{native,wasm}-review/`,
`logs/form-origin-pending-{red,tests}.log`에 보존한다.
기존 독립383쪽 PDF를 유지하고, 다음 단계에서130쪽 색조 소비 경로를 조사한다.
