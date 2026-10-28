# 윗줄 장식의 글줄 기준과 공통 기하

## 구현 전 근거

- #8038의 원 변경과 현재 소비 경로를 대조했다. 기존 위 밑줄은 Native/SVG/Canvas2D/CanvasKit에서
  `baseline - fontSize + 1px`로 따로 계산하고, 다중 선도 고정 px였다.
- 공개 서식 API로 생성한 정상 저장본을 한컴 2020 PrintMethod 0으로 출력했다. 12/24/36/48pt,
  선 모양 0~10, HWP/HWPX 다중 선의 독립 PDF와 원시 벡터를 대조했다.
- 추가 반례 `mixed-top-small-only.hwpx`는 가장 큰 48pt 글자에 윗줄이 없어도 12/24pt 글자의
  윗줄 높이와 굵기가 같은 줄의 48pt를 따른다. 강조점은 각 글자의 크기를 따르므로 구분한다.
- 실선 중심은 기준선 위 약 0.87em, 굵기는 0.04em이다. 다중 선은 아래 잉크 경계 약 0.85em에서
  굵기와 간격을 쌓는다. 파선 길이와 간격도 글꼴 크기에 비례한다. PDF 장치 반올림 차이는
  문서별 px 보정으로 흡수하지 않는다.
- 입력·PDF SHA, Print job과 벡터 관측은 ignored
  `output/pr-review/semanticist21-20261019/top-underline-independent-print-contract.json`,
  `decoration-boundary-print-paths-146.json`, `mcp-decoration-boundary-144/*-complete.json`에 보존했다.

## 생산과 소비

1. 실제 글줄 구성의 `RunEmitVars.max_fs`를 위 밑줄의 참조 크기로 전달한다. 표 높이·bbox 높이를
   글꼴 크기로 추정하지 않으며, 원본 글자 크기·저장 스타일은 바꾸지 않는다.
2. `text_decoration`에서 선별 오프셋·굵기·파선·원형 점의 기하를 한 번 정한다.
3. Native/SVG/Canvas2D, 잉크 clip·paint bounds 및 Paint JSON/CanvasKit이 그 결과를 소비한다.
   helper 뒤에서 기존 고정 px 앵커로 덮지 않는지 확인한다.
4. 원래 아래 밑줄·취소선·강조점의 계약은 각각 검증하며, 윗줄 수치로 대체하지 않는다.

## 검증 계획

현재 Native035/fresh WASM013을 수정 전 동결 제품으로 보존했다. 새 구현에서 단일·다중 선,
파선 4크기, 큰 비장식 글자가 있는 혼합 줄을 같은 독립 Print와 비교한다. 전쪽 Native/fresh
WASM raw 90% 이상과 확대 판독을 확인한 후 정식 관계 회귀를 추가한다. #8064의 정상 원본도
보존 검증하며, 이번 단계 결과를 전체 통합 PR 완료로 보고하지 않는다.

## 실행 결과와 미완료 범위

- Native035는 수정 전 제품이며, 런타임 소스 바이트가 커밋 `f5b6bf654c`와 같음을 확인했다.
  동일 28조건 중 3통과·25미달이다. 별도 강조점 1조건은 전후 모두 미해결이다.
- Native036에서 위 밑줄 27조건은 모두 raw 90% 이상이다. 최저 95.61305%, 혼합 크기·단일 실선·
  다중 선은 100%다. `top-underline-148-top-fixed-native/results.json`과 수정 전
  `top-underline-150-before-top-native/results.json`에 명령·원문 SHA·PDF SHA·TSV를 기록했다.
- 같은 줄의 비장식 큰 글자가 있는 반례의 Native/Print review와 standalone overlay를 같은 영역에서
  확대 판독했다. 선 높이·굵기·문자 소속은 맞으며 글자 잉크 굵기와 가장자리 차이는 남는다.
  4배 확대 진단 점수를 전쪽 raw 100%와 구분한다. glyph 잔차의 실제 공급 face를 추가 추적한다.
- 코드 검토에서 빈 폭 장식의 backend 일관성과 큰 원형 점의 paint bounds를 보완했다.
  최종 Native037 빌드는 exit0, 263초이며 같은 28조건을 다시 대조 중이다.
- TypeScript 검사와 Rust 포맷은 통과했다. fresh WASM014와 정식 회귀 추가·전 범위 검증은 미완료다.
  Native035/fresh WASM013의 기존 90조건은 양쪽 각86통과·4미달이다. 4미달은 clip과 그림이
  면적 없이 닿는 수동 진단 입력이며, 이를 통과나 정식 fixture 자격으로 바꾸지 않았다.

## 강조점 분석의 별도 증거

한컴 Print는 강조점1~4에 U+0307/U+030A/U+030C/U+0303, 5~6에 U+302E/U+302F를 사용한다.
다른 본문 글꼴에서도 강조점의 실제 글꼴은 HCRBatang이다. 위 밑줄의 줄 최대 크기와 달리 각
문자 크기를 따른다. 출력 기하를 보정하지 않은 현재 혼합 강조점은 76.12050%이며 완료가 아니다.

글꼴별 대조군의 첫 시도는 setter가 수용하지 않는 `fontFamily` 키를 사용해 모두 같은 본문 글꼴로
저장됐다. 이를 글꼴별 증거로 세지 않는다. `findOrCreateFontIdForLang`과 `fontIds`로 수정하고
실제 글자 속성을 재조회한 새 입력·새 Print job은 `emphasis-font-controls-149`와
`mcp-emphasis-font-149`에 원본을 보존했다. 이전 입력과 receipt는 덮어쓰지 않았다.

최종 Native037도 위 밑줄 27/27조건을 통과했다(최저95.61305%). 혼합 강조점 1조건은 76.12050%로
별도 미달이며 전쪽 Native 결과와 구분했다. 최신 결과는 `top-underline-151-top-final-native/results.json`이다.
동일 소스의 최적화 fresh WASM을 다음 단계에서 검증한다. 정식 fixture·회귀는 양쪽 자격을 확인한 뒤 추가한다.
