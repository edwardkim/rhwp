# 저장 그림의 음수 프레임과 실제 반전

## 발견한 회귀

고정 base `5e2b9bfbf0`와 같은 런타임의 #8064 producer를 Native035와 같은 입력·Print·글꼴·
DPI로 대조했다. `samples/exam_kor.hwp`와 `samples/hwpx/exam_kor.hwpx`의 13/18/19쪽에서
같은 하락이 확인됐다. 13쪽은 99.52816%에서 95.10311%로 내려갔다. 90% 이상이어도 후퇴이므로
수용하지 않는다. 전쪽 결과와 해당 쪽의 base/candidate review·overlay를 보존했다.

## 원인 계층과 독립 근거

- 문제 영역은 표 안 텍스트 배치가 아니라 BMP를 변환한 PNG 그림이다. base/후보의 논리 bbox는
  같다. WMF crop 기준과는 다른 경로다.
- 13쪽 원본 `image12`는 `curSz.height=4294956467`(signed -10829), scale Y=-0.16043,
  명시 flip=false다. 18쪽 `image18`은 signed -3370, 19쪽 `image22`는 signed -3326이다.
  두 그림도 음수 scale Y와 flip=false를 함께 보존한다.
- 독립 한컴 Print와 base는 정상 방향이다. 후보의 공통 `picture_transform`은 저장 행렬의
  음수 기저를 바로 반전으로 판정해 세 그림에 `vertFlip=true`를 발행한다.
- 현재 음수 크기의 프레임은 양수 물리 bbox로 이미 정규화돼 있다. 행렬을 이 양수 프레임에
  옮길 때도 원래 프레임 축의 부호를 함께 해석해야 한다. 문서명·문단 번호로 예외를 두지 않는다.

## 보정과 검증 계획

`저장 current size의 signed 축 → 회전 분해 후 기저 방향 → ShapeTransform → 공통 이미지
paint/clip → Native·Canvas2D·SVG·Paint JSON/CanvasKit`을 추적한다. 회전 전 크기를 계산하는
소비 지점도 부호가 아닌 길이를 사용하도록 확인한다. 일반 양수 프레임의 실제 flip, 회전·R*F,
반전 플래그와 저장 행렬의 구분, source viewport의 기존 대조군을 유지한다.

Native037/fresh WASM014는 이 보정 전 제품으로 보존한다. 현재 WASM014 빌드가 종료돼 소스 지문을
고정한 후 구현한다. 원본 두 형식의 전쪽과 기존 정상 대조군을 다시 출력해 raw 90% 이상·쪽수
일치·base 대비 하락 해소·확대 판독을 확인하고 새 이름의 관계 회귀를 추가한다. 미해결을 별도
이슈로 넘기거나 baseline을 낮춰 완료 처리하지 않는다.

## 증적

- `output/pr-review/semanticist21-20261019/pr8064-native-143/results.json`: 전체 동일 조건 대조.
- `pr8064-failures-155/{base,candidate}/p{013,018,019}`: 같은 페이지의 review·overlay.
- `exam-image-layer-tree-155.json`: 동결 WASM013의 실제 image paint 속성.
- 원본 HWPX의 section1/section2 `pic`와 독립 Print PDF는 원문 그대로 유지했다.


## 전체 검증 종료와 캡션의 별도 회귀

Native035의 #8064 179조건·2,082쪽은 모두 산출됐으며 176조건 통과·3조건 후퇴다. 국어 시험지
두 형식 외에 `samples/hwp-3.0-HWPML.hwp` 15/31쪽에서 2.26949/0.74111%p 하락이 있다.
이 두 쪽에는 그림이 없고 캡션 중복이 원인이다. 원 해결 #8094의 commit89a1c01247과 메인터너
보정 a938ba888c, 고정 base·후보의 실제 Caption TextRun과 독립 Print를 대조했다.

`auto_number_projection::project_placeholder`가 원문을 번호 앞·번호·뒤로 나누면서 앞뒤 조각에
전체 `display_text`를 복제했다. PUA 옛한글이 포함된 캡션의 앞·뒤 조각이 각각 같은 캡션 전체를
표시해 중복과 가운데 정렬 후퇴를 만들었다. 이미 사용하는 공통 `display_for_model_range`로
원문 조각과 표시 조각을 함께 나눈다. 번호는 한 원문 scalar에 투영하고 원문 주소를 유지한다.
원본·PDF·전후 TextRun과 대표 review/overlay는 `hwpml-failures-159`,
`hwpml-image-layer-tree-158.json`과 원 179조건 결과에 연결했다. 진단용 HWPX 내보내기는
원본 메타데이터 확인용이며 독립 기준이나 원본의 대체물로 세지 않는다.

## 파선의 원본 구간 위상

올바른 정규체만 공급해도 위 밑줄의 파선에 미세 차이가 남았다. 같은 원본 글자 모양이 언어별
표시 런으로 나뉠 때 펜 위상을 매번 0으로 다시 시작한 것이 원인이다. 한컴 Print의 원본
`H 첫째 줄`은 전체를 한 펜으로 그린다. 추가 API 정상 저장본4개를 실제 Print로 출력해,
글꼴·크기·색이 바뀐 원본 글자 모양 구간과 비장식 간격에서는 새 위상을 시작하는 반례도 확인했다.

같은 TextLine의 원본 char_shape_id·기준선·연속 전진에서 공통 위상을 만들고 실제 표시 방향의
구간 원점을 보존한다. Native·SVG·Canvas2D·Paint JSON/CanvasKit이 같은 위상을 소비한다.
부동 개체·다른 줄·다른 원본 구간을 글꼴 이름만으로 합치지 않는다.

## Native039 검증과 단계 고정

공유 `target/pr-review`, 빌드 잡8개로 Native039를 빌드해 exit0을 확인했다. 입력·Print·글꼴을
고정한 전쪽 재출력에서 국어 HWP/HWPX 각각20쪽과 HWPML122쪽의 쪽수가 일치했다.
고정 base 대비0.1%p 이상 하락은 모두0건이다. 국어13쪽은95.10311%에서99.50942%,
HWPML15쪽은97.58396%에서99.93051%,31쪽은99.11081%에서99.98561%로 회복됐다.

같은 제품의 위 밑줄·위상31조건은 모두90% 이상이며 최저99.17494%다. 원본 글자 모양의
실제 변경을 포함하는 추가 위상4조건은 모두100%다. 별도 강조점 대조군은71.17595%로
미달이며 이 단계에서 통과로 바꾸거나 새 회귀를 등록하지 않는다.

정규체 공급은 원본 HCR Batang Regular만 포함하는 디렉터리로 고정했다. 이전 진단의
255MiB 묶음은 Native 공급 예산64MiB를 넘겨 정규체를 빠뜨릴 수 있었다. 두 글꼴이 필요한
구간 반례에는 HCR Batang/HCR Dotum Regular 약48MiB를 공급했다. 해당 환경 오류를 제품
출력의 독립 기대값으로 사용하지 않는다.

`signed-phase-caption-162-combined-fixed-native/results.json`과
`signed-phase-caption-native039-preservation-163.json`에 명령·TSV·원본 SHA·전체 쪽 비교를
보존했다. `fixed-native039-panels-165`의 국어13쪽과 HWPML15/31쪽, 파선 대조군의 review와
standalone overlay를 새로 생성했다. 직접 판독에서 반전된 그림 방향, 캡션의 중복 제거와
번호 앞뒤 순서, 파선 연결을 확인했다. 기존 글리프 농도·표선 가장자리 잔차는 구분한다.

Native039는 이 단계의 미커밋 제품 바이트로 만든 동결 산출물이다. 단계 commit 이후에는
receipt의 파일별 지문과 commit 바이트의 동일성을 별도 확인한다. fresh WASM014는 이 보정
전 제품이므로 Native039의 결과를 WASM 검증으로 승격하지 않는다. 다음 단계에서 같은 제품의
fresh WASM과 필요한 정식 회귀를 검증한다. 이 단계 commit은 전체 통합 검증 완료가 아니다.
