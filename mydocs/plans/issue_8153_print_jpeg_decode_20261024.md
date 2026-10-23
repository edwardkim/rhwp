# #8153: Print JPEG의 공통 정수 변환과 표본 경계

## 원인과 독립 근거

원본 JPEG를 펼치는 IDCT와 색차 보간의 정수 반올림은 PDF로 전달할 RGB555·JPEG보다
앞선 단계다. 현재 Print 전달은 일반 화면용 `image` decoder를 그대로 사용하므로,
동일한 원본·crop·RGB555·JPEG 양자화를 적용해도 독립 한컴 Print 이미지와 차이가 남는다.
사진의 좌표나 실루엣 계산을 움직여 이 차이를 상쇄하지 않는다.

공개 Intel MediaSDK `pjdecdctcn.c`의 정수 IDCT와 IJG의 중심 표본 3:1 보간 규칙을
독립 JPEG 성분 및 원본 전체 사진과 대조했다. 공개 알고리즘과 기존 zune의 14비트
BT.601 색 변환으로 원본 사진 1,721,995픽셀의 RGB가 독립 대조 결과와 전부 일치했다.
이는 decoder 단계의 근거이며, 최종 렌더링·전쪽 TSV의 완료를 뜻하지 않는다.

- IDCT 공개 원문: `Intel-Media-SDK/MediaSDK`의
  `7a72de33a15d6e7cdb842b12b901a003f7154f0a/contrib/ipp/src/pjdecdctcn.c`.
  공개 소스의 MIT 고지를 Rust 번역에도 보존한다.
- 표본 규칙: IJG `jdsample.c`의 h2v1·h1v2·h2v2 중심 표본 보간과 실제 성분 폭·높이의
  끝 표본 복제. MCU 패딩을 실제 그림의 이웃 표본으로 사용하지 않는다.
- 색 변환: 기존 zune scalar의 BT.601 14비트 계수와 반올림을 그대로 사용한다.

비공개 모듈 조사·덤프·실행 파일·진단 자료는 ignored `output/`에만 보존한다.
제품에는 공개 알고리즘과 출처·라이선스만 포함한다.

## 생산·소비와 적용 범위

기존 bounded JPEG decoder에 인스턴스별 변환 선택을 추가한다. 기본값은 기존 동작이며
PDF import와 화면 렌더링의 선택을 바꾸지 않는다. Print JPEG에만 같은 IDCT·보간 결과를
공급하고, 팔레트 판정·원본 창·RGB555·Print JPEG 재압축이 이 결과를 함께 소비한다.
Native와 WASM은 같은 안전한 Rust scalar 구현을 사용한다.

일반 bitmap의 8,192축·32M픽셀 제한, fallible allocation 및 요청별 메모리 예산을 유지한다.
CMYK 등 다른 색 계약은 기존 decoder 경로를 유지하며, 원본 bbox·crop·알파·배치에는 손대지 않는다.

## 실제 검증 순서

1. 공개 C와 Rust IDCT를 전체 성분 및 음수·포화 경계에서 대조한다.
2. JPEG의 홀수·짝수 폭/높이, 한 표본 폭, MCU 이어받기, baseline/progressive와
   4:4:4·4:2:2·4:2:0 표본 경계를 실제 실행한다. 기본 decoder의 결과·예산 계약도 보존한다.
3. 원본 사진·잘린 사진·BMP/PNG 정상 대조군의 최종 Print 전달을 독립 PDF와 비교한다.
4. 영향 페이지의 최신 Native/fresh WASM TSV·확대 PNG를 확인한 뒤 정식 회귀를 등록한다.
   전쪽 1,532결과 및 기존 corpus의 회귀 검증은 별도 필수 완료 조건으로 유지한다.

현재 문서는 구현 전 계획이다. 진단 성공을 제품 또는 PR 준비 완료로 표시하지 않는다.

## 새 devel 리베이스 후 보존할 패키지 계약

`d9fd85ed5b`의 PR #8181 source 사전 검사는 base에서 통과했지만, 리베이스한
`6fe79b138a`에서는 `vendor/rhwp-jpeg-encoder`가 후보 의존 그래프 밖이라는 오류를
실제로 재현했다. 인코더를 기존 source 선별·manifest include 계약에 추가하고 Print의
직접 decoder 의존성도 선언한다. 내부 패키지의 `publish = false`와 제품 버전은 유지한다.

IJG 보간 구현은 요청 예산과 실제 성분 경계를 소비하는 vendored decoder 모듈에 둔다.
Print resolver는 그 선택 함수를 호출한다. IJG 원문 README를 decoder 패키지에 포함하고
해당 패키지의 SPDX 고지에도 IJG를 반영한다. 본체의 MIT 코드를 IJG 파생 소스와 같은
패키지에 섞지 않는다. 알고리즘·샘플 값·기본 decoder의 선택은 바꾸지 않는다.

리베이스 전 후보는 공개 85개 이미지·781개 IDCT 블록·기본 출력 85개·자원/API 4개를
통과했고, 실제 원본 두 형식의 90쪽 Print JPEG RGB 대조 2건도 통과했다. Native 90쪽은
raw 100%다. 이는 새 base와 아래 모듈 이동 뒤의 검증을 대체하지 않는다. 새 후보에서
같은 진단과 Native/fresh WASM·패키지 사전 검사를 다시 수행한다.

### 재배치한 공통 decoder의 재검증

IJG 모듈 이동 후 실제 diagnostic build는 7초·exit 0이다. 공개 참조 이미지 85건은
모두 RGB가 같고, IDCT 781블록의 차이도 0개다. 기본 decoder 출력 85건이 이전 구현과
바이트까지 같으며, 변환 등록 시점·출력 guard·할당 예산 등 자원/API 계약 4건도 통과했다.
이 결과는 `output/pr-review/issue8153-handbook-raw100-20261022/print-jpeg-decoder-diagnosis/`
아래에 보존한다. 독립 소스의 이전 기록과 모듈 이동 후 결과를 구분했다.

새 devel Native baseline은 같은 `target/pr-review`, jobs 8에서 7분 51초·exit 0으로
빌드했다. 커밋된 source의 패키지 사전 검사와 새 후보 Native/fresh WASM 출력은 계속
검증해야 한다. 이번 단계 커밋은 PR 제출·전쪽 100% 완료 판정이 아니다.
