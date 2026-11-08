# Native 굵기 기능 보존 분석

## 관측한 차이

같은 `pr8642/report-838-original.hwp`와 기존 독립 한컴 Print PDF의 24쪽을 비교했다.
원 PR의 각주·표·캡션 소유 검토 중 제목과 캡션의 굵기가 이전 Native 출력보다 가늘어진 것을
확인했다. raw 99.94%만으로 직접 판독을 대신하지 않는다. 글꼴 프로그램을 배포물에 추가하거나
기준 PDF를 다시 인쇄해 이 차이를 가리지 않는다.

- 이전 Native `0261687265f3a10a64e51809d006b2305a36d950`과 현재 Native
  `65195e9730f35bdbd7ea0c9a393e07e66125202e`에 같은 글꼴 환경·경로·compat 2022·96 DPI를 공급했다.
- 이전/현재 글꼴 환경을 교차 적용해도 굵기 차이는 실행 파일을 따라간다. 새 base
  `7f76cee60e38fceb63feacaff706cf984b19c427`에서도 같은 Native 제목 차이를 재현했다.
  따라서 글꼴 환경 JSON 교체만으로 해결됐다고 판정하지 않는다.
- 원본의 제목·표 캡션·표 머리말은 공개 레이어 getter에서 모두 명시 `bold=true`다.
  family 이름만으로 합성 굵기 여부를 추측하지 않는다.
- 독립 PDF의 같은 제목과 캡션에는 fill 및 stroke 텍스트가 함께 있다. 원본 문서의 bold와
  실제 프로그램의 weight, OS가 덧붙인 합성 weight, 최종 paint의 획을 구분해 추적한다.
- 이전/현재/Print 확대 영상의 검은 잉크 합계는 굵기 차이를 찾는 진단값이다. 정식 일치율이나
  새로운 통과 임계값으로 사용하지 않는다.

## 보정 범위와 검증

PR #8754의 실제 프로그램·cmap·collection face 선택과 20조건/562쪽 보호는 유지한다.
그 변경 전체를 되돌리지 않으며, Native의 실제 선택 face에서 최종 굵기 paint까지 공통 경로를
확인한 뒤 필요한 부분을 보정한다. WASM의 완료된 제품 source와 실제 바이트가 영향을 받지 않으면
그 증적을 재사용하고, Native는 새 실행 파일로 영향 영역과 필수 게이트를 다시 검증한다.

## 확인한 원인과 보정

호스트의 두 Fontconfig pattern 규칙은 `한컴바탕` 요청에 HCR 대체를 넣고 실제
Haansoft face를 다시 우선한다. OS의 한 번의 face 매칭과 스타일 목록 열거는 같은 결과를
보장하지 않는다. 실제 `family_order=0` 후보에 HCR Batang과 Haansoft Batang이 함께
들어왔으며, 새 TTF 우선 처리에서 다른 계열의 Bold TTF가 원 계열을 대신했다.
물리 프로그램 재개방 자체나 명시 `bold`가 사라진 문제가 아니다.

- `matching_system_programs`는 같은 요청 이름의 스타일 목록을 실제 family 소유로
  간주했다. 목록의 각 프로그램이 SFNT family를 선언했는지 확인하도록 보정한다.
- `match_system_family_style`의 같은 스타일 TTF 우선 선택도 실제 family 소유를
  확인한다. 같은 계열 안의 TTF → TTC → OTF 정책과 물리 프로그램의 cmap·paint 일치는 유지한다.
- 공개 GUST CFF와 OFL TTF 두 개만 격리 공급하고 같은 pattern 규칙을 공개 이름으로
  구성한 대조군을 만들었다. 원본 글꼴의 name 테이블이 독립 기대값이다.
  수정 전 요청 `Latin Modern Math` 대신 `Rhwp Family Contract`가 선택되어 exit 101로 실패했다.
- 새 정식 검사는 `native_system_font_family_ownership.rs`이며 새 프로세스의 전용
  Fontconfig에서만 두 프로그램을 노출한다. 보통/굵게, 원 CFF 계열/대조 TTF 계열의
  양방향 소유를 검사한다. 설치된 한컴·OTF나 다른 검사에서 초기화한 FontMgr에 의존하지 않는다.
  합성 이름 선택 계약이며 실문서의 px·쪽수·이미지 기대값을 만들지 않는다.

[수정 전 정식 검사 실패](native-family-ownership-r46g/before-test.txt) ·
[실패 실행 출처](native-family-ownership-r46g/before-provenance.json) ·
[사전 진단](native-family-ownership-r46g/before-probe.txt) ·
[실제 후보 묶음](native-family-ownership-r46g/actual-chain-before.json).

구현 보정 후 재검증은 아직 완료 전이다. 현재 기록만으로 통과나 PR 제출을 선언하지 않는다.
