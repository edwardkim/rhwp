# #8153 묶음 빈칸의 원본 표기와 공통 전진폭

## 목적과 근거

두 행정업무운영 편람의 232쪽에서 앞쪽 묶음 빈칸과 뒤 표지의 위치를 독립 한컴 Print와
대조한다. HWP 5.0 설명본 표 7은 묶음 빈칸을 제어코드 30으로 정의하고, 글자 모양은
언어별 장평과 자간을 기록한다. 설명본에는 이 제어의 전진폭 계산식이 없으므로 다음 한컴
관측값을 사양에 적힌 공식으로 취급하지 않는다.

동일 입력과 글자 모양에서 한컴 2020·2024 Print의 제어 표기 54조건은 같은 전진을 보였다.
한컴 2024의 리터럴 U+00A0 54조건은 자간의 영향을 받았고 제어 표기는 받지 않았다.
`useFontSpace`에 대한 반응도 일반 ASCII 공백과 구별됐다. 제어 표기 2개와 리터럴 1개를
같은 문단에 넣은 54조건에서도 다음 글자의 위치 변화가 리터럴 3개 대조군의 약 1/3이었다.
동일 glyph의 PDF 외곽 x 좌표를 96dpi로 환산했으며, 1/3과의 최대 차이는 약 0.107px다.
장치 격자·반올림 오차가 있는 관측치를 정확한 배치 상수로 삼지 않는다.

입력 생성·한컴 정규화·Print의 출처와 해시는 ignored
`output/pr-review/issue8153-handbook-raw100-20261022/remaining-raw100-diagnostic/small-residuals/script-metrics-common/`의
`nbsp-encoding-version-comparison.json` 및
`nbspace-mixed-metric-controls/mixed-origin-comparison.json`에 있다. 기존 편람 원본과 독립
Print PDF는 재생성하지 않았다. 혼용 대조군 Native 54쪽의 최저 raw는 59.40594%이므로
해당 자료를 아직 정식 렌더링 회귀 fixture로 등록하지 않는다.

## 확인한 결함과 책임 경로

- HWP `PARA_TEXT`의 코드 30과 HWPX `hp:nbSpace`는 공통 모델에서 U+00A0으로 투영된다.
  리터럴 U+00A0도 같은 모델 문자다. 현재 문단의 `control_mask` 30번 비트는 제어 표기의
  존재만 나타내며 혼용 문단에서 개별 출처를 구분하지 못한다.
- 실제 혼용 HWPX의 제어 요소 108개와 리터럴 54개는 현재 HWPX 왕복 및 HWP 경유 왕복에서
  모두 제어 요소 162개로 바뀐다. 독립 대조 입력과 두 저장 경로로 재현한 수정 전 검사
  2건은 기대 `(108, 54)` 대비 실제 `(162, 0)`으로 실패한다. 원본 검사는
  `nbsp_mixed_proposed.rs`, 결과는 `nbspace-mixed-proposed-before-v2.log`에 있다.
  최초 진단은 XML 빈 요소의 공백 표기를 잘못 집계했으므로 버리고 수정된 결과를 사용한다.
- 생산 지점은 `src/parser/body_text.rs::parse_para_text`와
  `src/parser/hwpx/section.rs::read_text_content_with_tabs`다. 저장은 각각
  `src/serializer/body_text.rs`와 `src/serializer/hwpx/section.rs`가 문단 비트를 사용한다.
- 측정은 `ParagraphMetricScope`와 `char_width_decision`을 통과한다. 실제 배치는
  `ComposedTextRun::text_style`과 같은 문자 전진 결과를 소비한다. 저장 줄과 재조판,
  표 셀·일반 본문 및 각 출력 backend가 개별 원본 출처를 공유해야 한다.
- 문자 삽입·삭제·분할·병합뿐 아니라 붙여넣기, 기존 문단 직접 교체 및 undo의 소유 경로도
  확인해야 한다. 모델의 문자 좌표와 원본 UTF-16 좌표를 혼동하지 않는다.

## 진행 순서

1. HWP·HWPX 파서가 개별 제어 위치와 리터럴을 구분해 보존하는 공통 모델 계약을 만든다.
   기존 출처 정보가 없는 합성 입력의 동작도 명시한다.
2. 편집과 저장이 같은 출처를 유지하도록 보정하고, 제어만·리터럴만·혼용·삽입·삭제·
   분할·병합 및 저장 왕복을 독립 근거로 검증한다. 기존 #5174의 정확한 검사는 유지한다.
3. 저장 줄과 재조판이 같은 출처별 전진을 측정·배치에 전달하도록 공통 로직을 보정한다.
   글꼴의 실제 공급, 장평·자간·`useFontSpace`와 앞뒤 문자를 함께 대조하고 문서명이나
   특정 폰트명으로 분기하지 않는다.
4. 수정 전후 대조군과 실제 편람 두 파일의 Native·fresh WASM을 동일 기준 PDF로 확인한다.
   원본 문서의 모든 페이지에서 raw 100%·미대응 0픽셀을 목표로 하며, 관련 경계의 직접
   판독과 정식 회귀 및 최종 필수 검증을 완료한다. 미완료 상태에서 PR을 만들지 않는다.
