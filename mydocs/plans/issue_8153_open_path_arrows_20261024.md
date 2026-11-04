# #8153: 열린 다각형·곡선의 원본 화살표 속성 전달

## 사양·기존 해결과 현재 결함

HWP 5.0 revision 1.3 표 86·87의 그리기 공통 테두리 속성에는 시작·끝 화살표와
크기·채움이 있다. HWPX도 다각형의 `lineShape`에 같은 속성을 저장한다.
#7688의 PR #7697 통합 `21ed64071f`와 후속 `496e5a0755` 및
`mydocs/working/task_m100_7688_stage5.md`를 확인했다. 공통 `renderer/arrow.rs`는
실제 선/경로와 접선으로 화살촉을 만들며 Native·SVG·Canvas2D·CanvasKit이 이미
이를 소비한다. 그 형상·크기 계약을 다시 구현할 문제가 아니다.

원본 161쪽의 L자 개체를 조사했다. 이들은 연결선 컨트롤이 아니라 열린 다각형이다.
빨간 다각형 2개와 파란 다각형 3개에 시작 또는 끝의 오목 화살표가 명시되어 있다.
파서는 그 속성을 보존하지만 일반/아핀 다각형의 PathNode 생산자가 `line_style`과
경로 끝점 정보를 전달하지 않는다. 독립 Print polygon에서 직접 구한 화살촉 5곳에
대한 현재 실제 Native layer는 0 PASS / 5 FAIL이며, 화살촉 payload 자체가 없다.

## 공통 결과와 소비 경로

원본 drawing 테두리 → 기존 LineStyle 해소 → 실제 완성 경로의 논리 시작·끝과
첫/마지막 선·Bezier 접선 → 기존 공통 화살촉 → 실제 backend/JSON 순서로 연결한다.
일반/아핀 다각형·곡선이 같은 전달 helper를 사용한다. 원본 점과 기존 경로·스타일·
소유·그룹 변환은 보존하며 backend별 화살촉이나 문서/색/좌표 분기를 추가하지 않는다.

명시적으로 닫힌 경로와 여러 subpath, 현재 접선 계약에 포함되지 않은 ArcTo는 이
single-open-path 전달의 대상이 아니다. 빈 경로·stroke 없음·원본 화살표 없음은
기존 결과를 유지한다. 같은 점에서 시작·끝나는 열린 Bezier를 좌표만으로 닫힌
경로라고 추측하지 않는다. 기존 연결선의 별도 LinkLineType 계약도 다시 해석하지 않는다.

## 검증과 현재 상태

같은 원본·독립 Print·실제 layer로 5개 화살촉의 생성·소유·방향·끝점과 확대 PNG를
먼저 확인한다. 기존 직선 방향 4곳, 원본 경로 점·무화살표 곡선과 다른 paint op를
보호한다. Source line/bitmap/clip 개선과 영향 36쪽을 새 source에서 다시 확인한다.
새 Native/fresh WASM·전쪽 100%·미대응 픽셀 0·정식 회귀 및 최종 제출 조건은 미완료다.

작업 자료는 ignored `source-line-ff722d79ef-open-path-arrow-oracle.json`과
`remaining-layer-ff722d79ef/p161-layer.json`, 실제 Print/이전/이후 PNG에 연결한다.

## 실제 Native 소비자 검증

`9906f3425e`를 별도 검증 worktree에서 고정 `target/pr-review`와 빌드 잡 8개로
빌드했다. `release-test` Native CLI 빌드는 5분 56초에 exit 0으로 끝났다.
동일 라이브러리 SHA를 검증한 진단 실행에서 원본 HWPX의 실제 Print layer를
다시 만들었다. 다각형 화살촉은 수정 전 0 PASS / 5 FAIL에서 수정 후
5 PASS / 0 FAIL로 바뀌었다. 독립 PDF polygon의 끝점을 기준으로 대조했으며,
앞서 보정한 직선 방향 4곳도 4 PASS / 0 FAIL로 유지됐다.

83쪽 139개·149쪽 245개 출력 요소는 이전과 같았다. 161쪽 170개 중 원본
화살표가 있는 다각형 5개에만 `lineStyle`·`connectorEndpoints`·`arrowHeads`가
추가됐다. 원래 경로 점·bbox·스타일과 나머지 165개 출력 요소는 동일하다.
독립 Print/이전/이후 확대 PNG에서도 누락됐던 L자 화살촉의 표시·방향을 확인했다.

HWP/HWPX 영향 18쪽씩 총 36쪽의 Native 검증은 exit 0으로 끝났다.
161쪽 raw는 HWPX 99.30697% → 99.32745%, HWP 99.30698% → 99.32746%로
각각 0.02048%p 개선됐다. 나머지 34쪽은 raw 변화가 없으며, 288·307쪽의
양쪽 형식 100%도 유지됐다. 새 source의 fresh WASM은 아직 실행하지 않았다.

ignored 작업 루트의 `open-path-9906f3425e/producer/receipt.json`,
`open-path-9906f3425e-open-path-arrow-oracle.json`,
`open-path-9906f3425e-line-direction-oracle.json`,
`open-path-9906f3425e/preserved-path-consumers.json`,
`open-path-9906f3425e/native-focus-result.json`과
`open-path-9906f3425e/p161-open-path-arrow-review.png`에 결과를 보존했다.
제목 상자 채움 경계와 얇은 선 차이가 남아 있어 이 개선을 161쪽 또는 목표의
100% 완료로 판정하지 않는다. 전쪽 Native/fresh WASM과 정식 회귀·제출 게이트는 계속한다.
