# #8153: 사각형 채움과 테두리 프레임 분리

## 독립 근거와 위반 규칙

HWP 5.0 revision 1.3 표 86·88의 선 굵기와 Outline style, HWPX
`lineShape/@width`·`outlineStyle`은 별도 원본 속성이다. `NONE`은 선의 표시를
생략하며 저장된 굵기까지 없애는 근거가 아니다. 161쪽의 검은 제목 상자 원본은
선 종류 NONE·굵기 425 HU·NORMAL을 보존한다. 일반 한컴 저장 왕복 뒤에도
이 속성과 도형 점은 그대로였다. 독립 Print에서 채움 폭·높이가 현재 실제
Native보다 약 4.5px 넓지만, 본문·테두리의 원래 프레임을 넓힐 문제가 아니다.

별도 1쪽 대조군의 NONE/SOLID × 0/60/425/1600 HU 8개 사각형을 한컴 2024로
Print했다. 실제 출력의 채움 경계는 선의 표시 여부와 독립적으로 같은 폭만큼
확장됐다. 추가로 설치된 한컴 2024의 실제 그리기 호출을 600dpi Print 장치에서
실행해 8개 사각형의 출력 polygon을 확보했다. 정상 테두리의 장치 폭 0/5/35/133에서
채움 확장은 각 변 0/2/14/53 장치 픽셀이었다. 이는 장치 선폭의 40%를 정수로
버리는 연산이며, 관측 비율에 맞춘 임의 보정값으로 정한 것이 아니다.
라이브러리 분석·직접 호출 코드와 원시 출력은 모두 ignored `output/`에만 둔다.

공식 사양은 위 원본 속성을 정의하지만 이 Print 채움 중첩의 장치 연산을 명시하지
않는다. 이 규칙을 공식 사양의 식으로 인용하거나 공식 원문을 변경하지 않는다.
2024 Print 호환 계약을 독립 Print·실제 실행으로 구분해 구현·검증한다.

## 공통 생산과 소비 지점

일반 Rectangle 생산자가 원본의 NORMAL 선폭을 별도 메타데이터로 보존한다.
Print의 공통 paint builder가 원래 프레임·테두리·자식 배치를 유지하고 채움만
독립된 Rectangle paint op로 만든다. 공통 장치 폭 변환 뒤 확장한 채움 프레임은
Native·Canvas2D·CanvasKit·SVG가 기존 Rectangle 소비자로 재생한다.
backend별 확장식이나 문서 ID·검은색·페이지 조건은 추가하지 않는다.

처음 입증한 대상은 NORMAL의 각진 단색·불투명·무그림자 사각형이다. 그림·그라데이션·
패턴 채움, OUTER/INNER, 둥근 모서리·별도 아핀 좌표계는 이번 입증의 대조군이
아니므로 기존 계약을 유지한다. 독립 근거가 없는 경로까지 같은 결과로 간주하지 않는다.
원본 폭 0·합성 노드와 Screen은 기존 출력으로 남긴다. 도형·글상자 bbox와 줄 소속,
원본 stroke 및 clip은 확장하지 않는다.

## 검증 계획

실제 Native layer에서 채움과 테두리·글상자 소유를 분리해 확인한다. 독립 Print
대조군 8개, 원본 161쪽 제목 상자의 채움 경계, 기존 화살촉 9곳과 다른 paint op,
영향 36쪽의 출력·raw를 대조한다. 그 뒤 새 source의 Native/fresh WASM과 전쪽
검증을 수행한다. 아직 실제 구현·재검증 전이며, 이 계획으로 완료를 판정하지 않는다.

작업 루트는 `output/pr-review/issue8153-handbook-raw100-20261022/`이며,
`black-rect-no-stroke-probe/observations.json`·`stroke-and-fill-bounds.json`에
Print 수치를, `so-analysis-20261024-table-width/`에 비공개 분석·호출 결과를 둔다.

## 구현과 장치 단위 확인

`f087833d6c`에서 원본 NORMAL 폭을 일반 Rectangle의 내부 메타데이터로
보존하고, 공통 paint builder가 채움과 원래 테두리의 op를 분리하도록 구현했다.
이미지 채움과 별도 그림 테두리는 이 경로에서 제외했다. metadata는 렌더 트리
직렬화에 추가하지 않으며, 기존 paint op와 backend를 사용한다.

추가 검토에서 장치 선폭은 PNG 출력 dpi와 독립적인 600dpi/HWPUNIT 변환으로
계산해야 함을 확인했다. 원본 폭을 먼저 96dpi px로 바꾼 뒤 실제 dpi의 장치
픽셀로 나누면 192dpi 등에서 양자화 폭을 잘못 선택한다. 따라서 600/7200으로
장치 폭을 정하고 최종 중첩에만 현재 dpi의 장치 픽셀 크기를 곱하도록 보정했다.
96dpi 결과는 동일하며, 비기본 dpi도 별도 검증한다.

비공개 호출 자료의 EMF 상태·선택 brush/pen·실제 polygon 점과 독립 Print를
대조한 진단은 8 PASS / 0 FAIL이었다. 폭 0의 무확장과 NONE/SOLID의 동일한
채움 확장을 실제 출력에서 확인했다. 이 결과는 rhwp의 실제 Native/WASM
소비자 검증을 대신하지 않으며, `f087833d6c`의 96dpi 빌드와 영향 36쪽 검증은
계속 실행 중이다. 새 장치 단위 보정의 최종 source도 이후 재검증한다.

## 현재 source의 실제 검증 결과

장치 단위를 보정한 `934aae310d` Native 빌드는 5분 50초에 exit 0으로 끝났다.
동일 원본 대조군을 실제 라이브러리로 72/96/144/192dpi에서 다시 조판한 4개
검증은 모두 통과했다. 각 dpi의 채움 확장을 독립 한컴 호출의 2/14/53 장치
픽셀과 대조했으며, layer JSON 반올림을 포함한 최대 오차는 0.00278 장치 픽셀이다.
원본 161쪽의 다각형 화살촉 5개와 직선 방향 4개도 모두 유지됐다. 기존 source
대비 161쪽의 사각형 이외 출력 요소 140개는 정확히 동일하다.

동일 source에서 8개 사각형의 Print·Screen PNG를 새로 만들었다. Screen은
수정 전과 전체 RGBA가 동일했다. 독립 Print에 노출된 검은 채움 7곳 중 기존
최대 경계 오차 9px였던 사례는 0px로 줄었고, 3px였던 사례는 1px로 줄었다.
나머지 보이는 채움은 0~1px 이내를 유지했다. 가장 굵은 SOLID의 1곳은 독립
Print와 양쪽 Native 모두 테두리가 검은 채움을 가려, 보이는 경계 검사로
통과를 주장하지 않는다. 해당 사례의 채움 기하는 별도 실제 호출 자료로 확인했다.

원본 HWP/HWPX 영향 18쪽씩 총 36쪽은 최신 source에서 다시 출력·측정해
exit 0이었다. 161쪽은 HWPX 99.32745% → 99.78293%, HWP
99.32746% → 99.78294%로 각각 0.45548%p 개선됐으며 나머지 34쪽의
raw는 동일했다. 이전 96dpi 보정 source의 36개 PNG와 현재 실제 PNG의
전체 RGB도 동일했다. 직접 판독한 채움 보정과 현재 캡처를 구분해 대조했다.

동일 입력·독립 Print의 전쪽 Native 766쪽 산출과 표준 release fresh WASM
빌드를 시작했다. 후자는 공유 Cargo 검증이 종료되고 실제 dpi/화살표 소비자와
36쪽 검증이 완료된 뒤 실행했으며 빌드 잡 8개·codegen-units 16을 사용한다.
두 작업의 최종 exit·전쪽 점수·WASM 재생은 아직 완료되지 않았다.

작업 루트의 `rectangle-overlap-934aae310d/native-focus-result.json`,
`dpi-consumers/dpi-consumer-result.json`, `p161-nonrectangle-preservation.json`,
`rectangle-overlap-controls/rectangle-overlap-934aae310d/boundary-result.json`에
증거를 연결한다. `logs/rectangle-overlap-native-whole.log`와
`logs/rectangle-overlap-standard-wasm-controller.log`에서 후속 실행을 확인한다.

161쪽에는 여전히 실제 미대응 픽셀 329개가 남는다. 가장 큰 잔여 영역은
`instid=568445309` 글상자의 「표 사이 간격 1㎜」 부근이었다. 원본은 두 문단에
4개 저장 LineSeg를 갖고 독립 Print도 4줄인데 현재 실제 Native는 3줄이다.
원 PR #1681 → 체리픽 `c279f7d93c` → 통합 PR #1704 및 당시 검토 기록을
찾았고, 재조판 경로의 고정 3px 허용과 높이 초과 판정은 추가 원인 검증 대상이다.
줄 수 차이와 구현 위치만으로 원인을 확정하거나 기존 기대값을 바꾸지 않는다.
원 PR의 정상 대조군과 저장 정보 유효성을 함께 확인한 뒤 공통 로직 보정을 계속한다.
100% 목표·정식 회귀·최종 제출은 완료되지 않았다.
