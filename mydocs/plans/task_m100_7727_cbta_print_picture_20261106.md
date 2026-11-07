# #7727 — CBTA 원본 전쪽 피델리티와 잘린 그림의 Print 전달

상태는 원인 진단이다. `task/7375-rectangle-corners`에서 #7700 증적 커밋 `9624efa2b6`
다음 순서로 처리한다. 구현·회귀 검사·기대값은 아직 변경하지 않았다.
#7445 부모를 먼저 닫거나 원본 32쪽의 미달을 예외로 통과시키지 않는다.

## 선행 해결과 현재 기준

- #6764 `d87b3037e5`, 후속 #6797 `29194a603f`, 메인터너 보정 `c547a52b2a`는
  앞 문단의 자리차지 밴드와 후속 블록 표의 소속을 보정했다.
- #6855 `26812ee08d`와 형식 보정 `cabcd9a07c`는 자리차지 표가 이미 채운 쪽을
  되감김 관문이 빈 쪽으로 읽는 문제를 보정했다. 모두 고정 base `170b886f`의 조상이다.
- 원본은 `samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp`다.
  SHA-256 `8ef9de3f35690bf9d7994527f77cb02d4a4fcff447c219a78fbc2855d64be6e7`,
  12,851,712바이트이며 기존 #7445 보존 사본과 같다. 한컴 2018 저장본이므로 새 Print는
  HWP2024 MCP client의 `engine 2020`을 사용한다.
- 독립 기준은 `pdf/1613000-202200037-air-traffic-controller-cbta-2020.pdf` 전 204쪽이다.
  SHA-256 `71145684dae73b459f088a8e10acc0335a7585c5345d7207401a3d988696fe6b`다.
  과거 201·202쪽 관측을 현재 결과로 재인용하지 않는다.
- 새 단계 시작 때 원격 devel이 여전히 `170b886f167b705b6e630c5b34bb638d92fae0a9`
  임을 확인했다. 반복 리베이스나 새 Cargo 빌드는 하지 않았다.

## 현재의 전쪽 사전 조사

같은 고정 base와 code head `fc378076f23aa60838a2871a7532d09830384216`의 frozen
Native·fresh WASM을 각각 전 204쪽 출력했다. 후보의 현재 Git head는 증적만 추가한
`9624efa2b6`이며 Rust·검사·스크립트는 같은 code head다.
정본 96DPI·compat2022·PDFium·2px raw·캡처 총 8을 유지했다.

Native는 양쪽 모두 최저 89.06875%, WASM은 양쪽 모두 89.10083%다.
미달은 각각 32쪽 하나이며 나머지 203쪽은 90% 이상이다.
Native의 base/후보 전 204쪽 실제 PNG는 byte 동일하다.
WASM도 같은 방식으로 실제 파일 hash를 대조해 전 204쪽 PNG가 byte 동일함을 확인했다.
수집 완료와 품질 통과를 구분한다.

32쪽 review/overlay에서는 본문·제목·그림의 대략적인 위치가 보존된다.
canonical raw 미일치 12,763픽셀 중 12,648개가 그림 9의 영역에 있다.
기존 색 경계 보조값은 99.90065%지만 제출 기준인 raw 89.06875%를 대신하지 않는다.
문턱·비교 영역·원본·미달 쪽을 바꾸지 않고 그림 전달 원인을 확인한다.

## 원본 그림과 독립 PDF의 교차 근거

독립 CFB/원시 HWP 레코드 읽기로 그림 9의 SHAPE_PICTURE `bin_data_id=10`을 찾았다.
DocInfo 10번의 `storage_id=10`, 확장자 BMP이며 원본 `BinData/BIN000A.bmp`와 연결된다.
원본은 2201×1105, 7,297,474바이트이고 SHA-256은
`ad8e5e2aa6b707e115657b37aaecb513e3df2c492dd09763dc610b57cb4999eb`다.
밝기·대비·효과는 0이고 crop은 `[16458, 7645, 160111, 82860]`다.
raw 문단 332와 Native IR 문단 333의 차이는 caption 정규화 경로도 교차 확인한다.

기존 PDF의 해당 그림은 object 347, 1915×1003 JPEG와 1비트 mask다.
JPEG는 4:2:0이며 DQT의 첫 값은 luma 23, chroma 24다. 원본 BMP와 PDF의 변환을
동일 원본의 새 한컴 Print로 재확인한다. 기존 PDF는 덮어쓰지 않는다.

현재 공통 Print의 `print_jpeg_payload`는 잘린 창이 전체 원본과 다르면 반환하지 않는다.
2024 전용 crop/장치 창 경로도 별도 세대 조건이 있다. 이 출처 가드를 무조건 풀거나
문서명·쪽수 예외를 넣지 않고, 독립 출력의 창·팔레트·압축 계약을 먼저 확인한다.

## 보정과 완료 조건

1. 독립 한컴 2020 Print와 원본 crop·정수 창·JPEG 압축을 대조해 조판 결함과 기준 오류를 구분한다.
2. 구현이 필요하면 source IR·BinData·그림 bbox를 보존하고 공통 Print payload에서 처리한다.
   원본 전체 창·바깥 crop·투명 자원·작은 팔레트·Screen과 2024의 기존 계약을 대조군으로 유지한다.
3. 원본 Native/fresh WASM 전 204쪽이 모두 90% 이상이고 새 raw 하락이 0.1%p 이내인지 확인한다.
   내용 소속·누락·중복과 기존 float/되감김 문제의 독립 쪽 소속도 확인한다.
4. 새 렌더링 회귀와 과거 제거 검사의 복원은 위 조건과 직접 판독 뒤 진행한다.
   98% 이상인 중요한 쪽에는 독립 Print ±2px 보조 핀을 쓰고 원문·소유 관계도 유지한다.
   잘못된 예전 절대 쪽수·좌표를 그대로 되살리지 않는다. 기존 정상 검사를 지우지 않는다.
5. 해당 소스의 focused·Native Skia 3종·전체 nextest·lint/build/manifest/unit tier와 실제 소비를
   완료하고 이슈별 로컬 커밋으로 정리한다. 원격 PR·통합·이슈 종료는 별도 상태다.

임시 스크립트·원본 자원 추출·로그는 ignored `output/open-issues-oldest-20261105/` 아래에
보존한다. 최종 증적은 구현과 검증이 끝난 뒤 정식 자산으로 옮기며, 임시 산출물을 PR에 넣지 않는다.


## 실제 한컴 2020 Print로 교차 검증한 결과

기존 `-2020.pdf`의 실제 Creator는 `Hwp 2022 0.0.0.0`이었다.
기존 PDF를 삭제·덮어쓰지 않고 같은 원본을 HWP2024 MCP client의 engine 2020으로
새 Print했다. job `a4dfe6a7-10fd-4155-925e-c1baba91e1a2`는 61초에 성공했고,
Hwp 2020 `11.0.0.9136`, `pdf_print_method=0`, `hancom2020_pdf_driver_one_up`을
실행 결과에서 확인했다. 새 PDF는 204쪽, 4,970,756바이트, SHA-256
`0db1eb55eda0f1a4985eeaf187656c23499616b7dccddcbb7c874c0937e1b2e0`다.

구현 변경 없이 Native 32쪽 raw는 새 Print에서 99.50563%, WASM은 99.49742%다.
새 PDF를 정본 PDFium으로 전쪽 다시 래스터하고 source/build/글꼴/실제 PNG hash를
검증한 기존 Native·WASM raster와 canonical `--png-pair`로 전 204쪽 TSV를 새로 산출했다.
고정 devel과 후보는 Native 모두 최저 92.05232%, WASM 모두 92.16419%(11쪽)이며,
각각 미달 0·쪽 누락 0이다. 값과 실제 PNG가 같아 후보의 새 후퇴도 없다.
인쇄 세대가 다른 기존 PDF의 89%를 보조 색 경계값이나 예외로 통과시킨 것이 아니다.

11·32·185·186·187쪽을 일반 Native/fresh WASM 경로에서 다시 출력해 review/overlay를
만들었다. Native 11쪽에서 본문 내용과 줄 소속을, 186쪽에서 앞 표의 마지막 행·
`과목 2: 인적 요소`·새 46×3 표의 올바른 이어받기를 직접 확인했다.
185–187쪽은 양쪽 모두 99.9%대로 새 px 보조 검사의 사전 조건을 충족한다.
182쪽은 96.23%대이므로 새 절대 좌표 핀 대상으로 삼지 않는다.

잘린 BMP의 JPEG 전달이 현재 source 결함이라는 초기 가설은 이 결과로 확정하지 않는다.
기존 2024 출처 가드를 풀거나 새 RGB 보정을 구현하면 올바른 2020 출력을 바꿀 수 있으므로
현재 renderer는 수정하지 않았다. 다음 단계는 원본의 실제 float/되감김 소속과 기존
회귀의 181쪽·1122.5px 상수가 현재 다른 쪽/용지를 가리키는지 검토하고, 기존 결함의
검출 의미를 보존해 검사를 보강·복원하는 것이다. 원본·두 PDF·원래 미달 수치는 유지한다.


## 최신 테스트 head의 Native Skia 필수 검사

로컬 커밋 `98e5afbb7a`에서 최소 글꼴·jobs8·threads8·Cargo 순차 조건으로
지침의 Native Skia 3종을 모두 별도 실행했다. 라이브러리 4,039PASS/0FAIL/12ignore,
PNG2PASS, 직접 PDF22PASS다. Native feature 전체 target Clippy와 fmt도 통과했다.
정식 영수증은 `mydocs/pr/assets/issue7727/verification-20261106/native-required.json`이다.
기본 nextest 결과로 Native 검사를 대체하지 않았다. 새 검사 포함 전체 회귀와 나머지
최종 게이트·기존 corpus 원장 재활성화 검토는 완료로 표시하지 않는다.


## CBTA 코퍼스 재활성화

기존 원장과 같은 계산식·Mac 최소 글꼴에서 고정 devel과 후보 전 204쪽의 본문 하단 넘침,
용지 밖, 글자 겹침, 셀 줄 넘침은 각각 0이다. 기준을 올리지 않고 원본 두 경로의 공통
6개 축 제외와 body/text 개별 제외를 해제했다. 해당 세 원장에 0을 명시하고 다른 입력은 유지했다.
제외 재도입을 차단하는 원문 소유 검사는 Native 복원 전 3PASS/1FAIL → 복원 후 4PASS/0FAIL이다.
이 단계를 커밋한 뒤 새 테스트 head의 전체 nextest와 범위별 게이트를 순차 실행한다.


## 코퍼스 복원 후 전체 검증 완료

커밋 `9b4c850535`에서 Mac 최소 글꼴 전체 nextest12,657PASS/0FAIL/45skip,
411.620초를 확인했다. 복원한 네 검사와 코퍼스 수집 경로도 포함된다.
Native Skia4,039+2+22PASS 및 fmt·Native/기본/WASM/all-target Clippy·workspace build·
고정base manifest/unit-tier 모두 exit0이다. `final-gates.json`에 실제 명령·로그 hash를 보존했다.
이슈#7727의 로컬 검증을 마치고 다음 오래된 #7728의 별도 원인 보정으로 진행한다.
원격통합·이슈종료·최종 제출 편람 gate 및 전체 오래된 이슈 목표의 완료로 확대하지 않는다.
