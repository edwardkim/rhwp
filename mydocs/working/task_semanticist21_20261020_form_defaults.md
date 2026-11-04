# #8096 메인터너 보정: 양식 속성과 빈 호스트 줄

## 수정 전 근거

원 head `67acb6d0562dc3b8df04e4a2ebbc6bf39ea9ba07`의 재현 생성 함수를 바꾸지 않고
HWP/HWPX를 만들었다. 각각 저장 제품에 맞는 한컴 2020 Print로 출력했다.
Native 033은 HWP 60.05486%, HWPX 74.37026%이며, 원본·PDF SHA와 actual Print 증거는
ignored `output/pr-review/semanticist21-20261019/mcp-pr8096-137/`에 있다.

- HWPX `drawFrame=1`을 지켰으나 `borderTypeIDRef=0`도 있어 실제 Print의 외곽 프레임은 없다.
  renderer는 BorderType을 소비하지 않았다. 체크/라디오의 내부 아이콘과 외곽선을 구분한다.
- 체크/라디오에 OPAQUE와 검은 배경이 선언됐다. Print는 전체 개체 배경을 채우고 renderer는
  작은 아이콘만 그렸다. 공통 FormDrawing에서 선언한 배경을 먼저 그린다. TRANSPARENT도 보존한다.
- HWP writer의 타입별 기본 BorderType은 버튼 4, 체크/라디오 0, 콤보/입력 5다. HWPX writer는
  모든 타입의 누락 기본을 0으로 써 저장 포맷에 따라 외형이 바뀐다. 같은 기본값 함수를 공유한다.
- HWP의 줄 높이 차이는 재현 생성 시 수식의 자리에 FormObject를 직접 바꾼 경로에서 저장된
  원본 LineSeg와 실제 control 높이·태그를 먼저 대조한다. 새 기본값만으로 해소됐다고 쓰지 않는다.

## 생산과 소비

`FormObject.properties → FormAppearance::resolve → form_drawing → Native/Canvas/SVG/JSON`에서
같은 배경·프레임 조건을 쓴다. HWP/HWPX 저장은 같은 타입별 BorderType 기본값을 소비한다.
문서 ID나 목표 화면 좌표로 분기하지 않는다. 기존 form-01 등 정상 입력도 같은 Print로 재검증한다.

## 검증 계획

기존 원 재현/Print를 보존하고 Native 새 제품으로 대조한다. 저장 기본값 변경은 새 저장본의
별도 Print로 확인한다. 모든 관련 페이지의 Native/fresh WASM 90% 이상·직접 판독 전에 새 실물
회귀/golden을 추가하거나 기대값을 바꾸지 않는다. 전체 통합 PR은 계속 보류 상태다.

## 저장 줄 원인 확인

`form-source-observations-140.json`에서 HWP의 줄 태그가 0x80060000(합성), 높이 975HU이며
각 Form의 원본 높이는 1,984/1,450HU인 것을 확인했다. 정상 form-01은 태그 0x60000이며
저장 줄 높이가 개체 높이와 일치한다. 단일 Form만 있는 합성 줄은 빈 호스트 줄과 같은
원본 높이·baseline 결과를 소비하게 보정하며 정상 저장 줄의 수용 조건은 바꾸지 않는다.

## Native 보정 결과

Native 034 빌드 exit 0, 212.81초, 빌드 잡 8개와 공용 target/pr-review를 사용했다.
`char-overlap-native-build-034-result.json`에 제품과 소스 해시를 보존했다.

| 입력 | 보정 전 Native | 보정 후 Native |
| --- | ---: | ---: |
| 원 HWP | 60.05486% | 99.99195% |
| 원 HWPX | 74.37026% | 99.99179% |
| 기존 form-01 HWP | 기존 #8064 100% | 100% |
| 기존 form-01 HWPX | 기존 #8064 100% | 100% |
| 기존 암호 입력 상자 | 정상 대조군 | 99.78036% |

새 기본값으로 다시 저장한 HWP/HWPX도 각각 독립 Print와 99.99195%다.
`form-controls-140-form-fixed-native/`, `form-defaults-140-form-fixed-native/`의 원문 TSV를
보존한다. 원 HWPX의 같은 배율 review/standalone overlay를 직접 판독했고 소실·줄 간격·배경·
프레임의 큰 차이가 해소됐다. `form-controls-panels-141/`에서 두 PNG를 함께 확인했다.
얇은 아이콘·글자 가장자리 잔차는 남으며 fresh WASM과 최종 회귀는 아직 진행 전이다.
이 중간 커밋으로 원 PR 수용이나 통합 검토 완료를 선언하지 않는다.
