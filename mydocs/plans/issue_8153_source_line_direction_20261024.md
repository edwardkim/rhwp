# #8153: 저장 직선의 방향 보정 플래그와 실제 화살표

## 위반된 규칙과 독립 근거

공식 HWP 5.0 revision 1.3 표 92의 직선 레코드는 시작·끝 좌표 뒤의 UINT16을,
처음 생성한 수직·수평선의 방향을 바로잡는 플래그로 정의한다. 공식 PDF와 프로젝트
설명본의 해당 문장을 대조했으며 사양 설명의 오류가 아니다. 파서는 HWP의 해당 값을
`LineShape.started_right_or_bottom`으로 읽고, HWPX의 `isReverseHV`도 같은 값으로
보존하지만 최종 선의 끝점 생산자는 이를 사용하지 않았다. 원본을 다시 저장해도
플래그는 보존되므로 serializer나 기대 좌표를 바꿔 해결할 문제가 아니다.

원본 HWP·HWPX를 각각 실제 파싱한 결과는 동일하다. 일반 선 71개 중 플래그가 있는
선 9개, 연결선 20개 중 플래그가 있는 선은 0개다. 161쪽의 파란 화살표 4개가 해당
플래그를 갖는다. 독립 Print의 파란 채움 polygon에서 직접 얻은 화살촉과 현재 Native
layer의 실제 화살촉을 대조하면 4개 모두 반대 끝에 있으며, 최대 축 오차는 약 52.84px다.
지표가 99% 이상이라는 이유로 실제 방향 결함을 면제하지 않는다.

## 생산·소비 경로

- 원본 시작·끝 좌표와 방향 플래그 → 저장 아핀/그룹의 실제 투영 → 공통 방향 보정 →
  LineNode의 논리 시작·끝 → 같은 선 스타일과 화살촉을 각 backend가 소비한다.
- 일반 직선과 묶음의 아핀 직선이 같은 공통 결과를 사용한다. 플래그가 있는 선의
  끝점 순서만 바로잡으며 저장 좌표·폭·그룹 행렬·head/tail 스타일을 바꾸지 않는다.
  음의 배율과 방향 플래그는 서로 다른 원본 속성이다. 좌표 부호로 플래그를 추측하지 않는다.
- 제어점 없는 연결선도 같은 끝점 생산자를 소비한다. 현재 두 원본의 연결선 20개는
  모두 플래그 0이다. 제어점 경로와 LinkLineType은 이 원본 결함의 적용 대상이 아니며,
  저장 경로를 임의로 뒤집어 현재 범위를 확장하지 않는다.
- 문서 ID·좌표·색·그룹 깊이로 동작을 선택하지 않는다. 비인쇄·Screen 여부에 따른
  방향 예외를 만들지 않는다. 편집 시 보이는 선과 인쇄의 논리 방향도 같은 계약이다.

## 검증 순서와 현재 상태

독립 Print polygon의 원래 좌표와 source instid 4개를 작업 자료에 고정했다.
수정 전 실제 Native 화살촉은 0 PASS / 4 FAIL이다. 새 Native 제품의 layer와 같은
Print를 다시 비교하고 161쪽 원본 PNG·확대 판독·raw와 실제 미대응 픽셀을 확인한다.
원본의 다른 플래그 선·일반 선·연결선과 이전 bitmap/clip 개선을 함께 보호한다.
관계·좌표 회귀의 정식 등록은 최종 Native/fresh WASM 선행 조건과 직접 판독 후에 한다.

자료는 ignored 작업 경로의 `line-direction-original-models.txt`,
`rectangle-clip-e8ec8c9845-line-direction-oracle.json`, `remaining-layer-e8ec8c9845/`,
`image-coverage-d3fdfa0c51/p161-source-line-matrices.json` 등에 보존했다.
새 head의 전쪽·fresh WASM·최종 회귀와 전쪽 100%는 아직 미완료다.

## `ff722d79ef`의 실제 결과

Native 빌드는 jobs 8·고정 target에서 6분 4초 후 종료 코드 0이다. 같은 제품 library로
원본 161쪽 layer를 다시 생성했고, 같은 독립 Print polygon에 대한 실제 화살촉 대조는
0 PASS / 4 FAIL → 4 PASS / 0 FAIL이다. source·library·진단 실행 파일·글꼴 환경·layer
SHA를 영수증에 보존했다. 기존 후보와 leaf paint op를 순서대로 비교해 변경은 정확히
해당 선 4개, 다른 선 10개와 선 이외 op 156개는 바이트 단위로 동일함을 확인했다.

원본 영향 36쪽 비교도 종료 코드 0이며, 직전 clip 후보 대비 최대 하락은 0.00006%p다.
161쪽 raw는 HWPX 99.28019 → 99.30697, HWP 99.28020 → 99.30698이다.
288·307쪽의 양쪽 포맷 raw 100%도 유지했다. 실제 Print/수정 전/수정 후 확대 PNG에서
위·아래 및 왼쪽 화살표의 방향을 직접 확인했다. 작은 점수 변화로 실제 방향의 보정을
무의미하다고 판단하지 않으며, 이 결과를 쪽 전체 100%나 fresh WASM 통과로 바꾸어 쓰지 않는다.

같은 확대에서 L자 개체의 화살촉은 아직 없었다. 원본을 추적하니 해당 개체는 연결선
컨트롤이 아니라 끝 화살표 속성을 가진 열린 다각형이다. 현재 다각형 생산자는 이 속성을
공통 화살촉 소비자에게 전달하지 않는다. 저장 직선 방향과 별개인 이 전달 누락을 이어서
보정한다. 기존 공통 화살촉 형상·크기 계약은 유지한다.

증거: `source-line-ff722d79ef-line-direction-oracle.json`,
`source-line-ff722d79ef/p161-preserved-consumers.json`, `p161-arrow-direction-review.png`,
`remaining-layer-ff722d79ef/receipt.json`, `logs/source-line-native-*.log`.
