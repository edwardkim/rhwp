# #8153: Print JPEG의 공통 정수 변환과 표본 경계

## 2026-10-25: 최종 JPEG 재생과 원본 변환의 구분

현재 작업 범위는 행정업무 편람 HWP·HWPX다. 두 원본의 90쪽은 같은 최종 JPEG
바이트를 전달한다. Native Skia의 복호화 RGB는 독립 libjpeg와 일치하지만 실제 WASM
`image::load_from_memory`의 결과는 26,930픽셀, 채널 최댓값 4만큼 다르다.
실제 Canvas 원본을 관측해 Native의 `image` crate 결과와 바이트까지 같음을 확인했다.
브라우저 JPEG 디코더가 만든 차이라고 표현하지 않는다.

공개 libjpeg-turbo 3.1.2의
[`jidctint.c`](https://github.com/libjpeg-turbo/libjpeg-turbo/blob/3.1.2/src/jidctint.c),
[`jdsample.c`](https://github.com/libjpeg-turbo/libjpeg-turbo/blob/3.1.2/src/jdsample.c),
[`jdcolor.c`](https://github.com/libjpeg-turbo/libjpeg-turbo/blob/3.1.2/src/jdcolor.c)의
8비트 ISLOW IDCT·중심 보간·16비트 YCbCr 변환을 독립 진단으로 대조했다.
보간만 변경하면 차이가 1,387픽셀, IDCT까지 변경하면 5픽셀, 색 변환까지 적용하면
0픽셀이다. 기존 코덱 경계 85건과 편람의 실제 JPEG 4건, 총 89건이 독립 libjpeg RGB와
전부 일치했다. 회색·RGB JPEG에는 불가능한 YCbCr 출력을 요청하지 않고 원래 색 공간을
유지해야 함도 실제 첫 진단 실패와 보정 재실행으로 확인했다.

같은 optimized WASM·원본·좌표·PDF·실루엣 규칙에서 표본 함수가 읽는 RGB만 교체한
원인 분리 진단은 90쪽 raw 100%·미대응 0개다. 원래 관측과 같은 두 번의 source 읽기를
확인했다. 첫 후처리는 읽기 횟수를 1회라고 잘못 가정해 실패했으며, 실제 관측 횟수와
PNG 해시를 대조하는 감사만 다시 실행했다. 이는 제품 구현·검증 통과가 아니다.

다음 구현은 `최종 Print JPEG → 공통 bounded 재생 decoder → 실제 원본 Canvas →
기존 공통 장치 표본 → Canvas 출력`을 연결한다. 원본 사진을 Print용으로 펼치는 기존
Intel IDCT·RGB555·JPEG 압축은 별도 계약이므로 유지한다. 화면·기본 decoder·PDF import의
선택은 바꾸지 않으며 문서·쪽·그림 크기로 분기하지 않는다. 최종 재생 선택은 명시적인
Print 표본 소비 경로로 제한한다. 새 IJG 파생 구현은 기존 IJG 라이선스를 보존한 vendor에
두고, 재생 전에도 헤더·8192축·32M픽셀·할당 예산·fallible output guard를 적용한다.

구현 후 같은 89건과 자원 경계를 실제 제품 함수로 재검증하고, 두 원본 90쪽 및 JPEG가
있는 영향 쪽의 표준 optimized fresh WASM·Native 대조·확대 판독을 완료한다. 기존에
후퇴한 255·263쪽은 이 JPEG 진단 통과로 해결된 것으로 세지 않는다. 새 정식 회귀는
해당 시각 선행 조건을 갖춘 후 등록하며, 전쪽 1,532결과의 raw100 목표는 그대로 남는다.

진단 자료는 ignored `handbook-paper-frame-candidate/` 아래 `jpeg-p090-decoders/`,
`jpeg-final-replay-controls-v2/`, `canvas-standard-jpeg-p090-diagnostic/`에 보존한다.
비공개 모듈 분석·덤프는 Git에 포함하지 않는다.

### 최종 재생 구현과 호스트 선행 검증

최종 재생 선택도 기존 `print_jpeg.rs`의 같은 헤더·예산·fallible output 생산 함수를
소비하게 연결했다. 실제 source와 바이트가 같은 header 모듈 및 제품 decoder를 연결한
호스트 진단 89건은 독립 RGB와 모두 일치하고, 기존 원본 변환 85건도 독립 기대 RGB를
그대로 유지한다. 축 초과·면적 초과·잘린 JPEG 헤더 3건도 거절했다. 첫 진단 build의
module 경로 해석 오류는 header와 자식 module의 디렉터리 구조를 보존한 source snapshot으로
보정했으며 원본과 snapshot SHA-256의 동일성을 기록했다. 실제 제품 코드는 이 문제로
바꾸지 않았다. 자료는 `handbook-paper-frame-candidate/jpeg-final-product-controls/`다.

Canvas의 명시적인 Print 표본 경로가 최종 재생 RGB를 소비한다. 일반 source canvas와
cache를 구분하며 새 cache는 16MiB·64항목 이내로 유지하고, 큰 그림은 cache하지 않는다.
일반 화면·원본 전달·PDF import는 원래 선택을 유지한다. 전체 source crop은 기존
`draw_image` 위임을 거쳐 같은 소비 지점에 들어오며, 원점을 옮기거나 crop을 다시 추측하지 않는다.

사전 정책 검사에서 앞선 단계의 `src` 내부 산술 검사 7건 증설을 발견해
`tests/cases/issue_8153_print_device_arithmetic.rs`로 옮겼다. 독립 용지 원점 검사는
`PrintPaper::from_css`의 실제 결과를 사용하고, 참조 DC 계산 helper의 가시성은 부모 module
까지만 허용했다. 기대 수치와 12조건·표본 경계의 보호는 유지한다. 직접 실행과 suite 안의
동일한 module 배치 실행에서 각각 7건 PASS다. 정확한 경로의 `path_attr`·`root_mod`만
허용하는 정책을 더해 독립 integration target을 늘리지 않는다. 최초 suite 정책 검사 실패와
보정 재실행은 별도로 보존한다. source unit 검사는 base의 4101건·296 module로 통과했다.

위 결과는 호스트 선행 검증이다. 실제 source를 커밋한 뒤 Native 출력과 표준 최적화
fresh WASM에서 90쪽 및 영향·정상 대조 쪽을 검증한다. 최종 전쪽 raw100·정식 회귀와
전체 검증은 아직 완료되지 않았다.

## 원인과 독립 근거

### 최종 재생 후보 `a2e14ae3bb`의 실제 전쪽 검증

표준 optimized fresh WASM은 jobs 8·codegen units 16으로 wasm-opt까지 21분 32초,
Native release-test/native-skia는 같은 `target/pr-review`에서 7분 56초·exit 0이다.
WASM SHA-256은 `5124c84b7dbf536a3045d5b721e7c6ec25a57dc13c12c5902e70ae96c5e19362`,
Native CLI SHA-256은 `ce3d44a0428eb50ca3adbc2ae83aef36e3003056dfa35928561e2306b92f25ff`다.
같은 두 원본·독립 2024 Print·정확한 96dpi에서 전 1,532결과를 새로 생산했다.

| 입력 | backend | 최저 raw % | 미대응 0픽셀 쪽 | 미대응 픽셀 합 |
| --- | --- | ---: | ---: | ---: |
| 편람 HWPX | Native | 99.36039 | 246 / 383 | 14493 |
| 편람 HWP | Native | 98.93537 | 256 / 383 | 13414 |
| 편람 HWPX | fresh WASM | 99.24283 | 245 / 383 | 15123 |
| 편람 HWP | fresh WASM | 98.93468 | 254 / 383 | 13969 |

Native 전 766 PNG는 앞선 용지 프레임 후보와 바이트까지 같다. fresh WASM의 90쪽은
두 형식 모두 raw 100%·미대응 0픽셀이다. 다만 92쪽은 각각 11→12픽셀로 늘어 집중 비교의
보존 gate가 실제 실패했다. 새 한 픽셀의 위치와 25×25 RGB 주변은 Native와 정확히 같다.
기존 decoder 오류가 가리던 위치 잔여가 드러난 것이며, 이를 목표 달성이나 보존 gate 통과로
바꾸지 않는다. 별도 전쪽 특성 조사는 이 실패를 보존한 채 완료했다. 136쪽의 -0.00001%p는
미대응 44개가 같은 상태에서 내용 합집합 크기만 바뀐 결과다.

전쪽 raw 100%·미대응 0픽셀은 1,001 / 1,532결과다. 실제 최적화 WASM의 90쪽 review를
직접 판독했고, 92쪽의 새 차이도 Native와 같은 위치임을 확대 확인했다. raw 100%를 RGB 전체
동일성으로 표현하지 않는다. 남은 531결과·기존 실패 검사·최종 필수 검증은 계속 처리한다.

실제 실행 원장은 ignored `handbook-final-jpeg-replay-candidate/` 아래에 있다.
Native TSV는 `native/handbook-{hwp,hwpx}/whole-score/silhouette.tsv`, WASM TSV는
`wasm-whole/handbook-{hwp,hwpx}/whole-score/silhouette.tsv`다. 최초 집중 gate 실패,
후속 특성 조사와 766 PNG 보존 감사도 별도 원장으로 유지한다.

### 다음 단계: 글줄 그림의 원점 생산·소비 검증

기존 다음 장치 원점 계약은 PR #8191의 merge `60972534680`에서 반영됐으며,
Stage82·83·107의 C352/C354/C356 독립 Print·실제 소비·회귀 보호를 먼저 확인했다.
이전 전체 최근접 원점 진단은 이 대조군과 맞지 않으므로 그대로 제품에 적용하지 않는다.

글줄 안 그림은 저장 LineSeg의 상대 위치와 본문/단 원점이 서로 다른 좌표 소유자다.
독립 검증용 같은 문서에서 여백 위쪽만 5668→7200 HU로 변경하고, 한컴 2024에서 실제
HWP 저장 후 Print했다. 본문 원점과 그림의 상대 위치를 각각 최근접 장치 정수로 변환한
예측은 세 그림 모두 독립 PDF와 0.00042pt 이내다. 전역 ceil은 첫째·셋째 그림에서 약
0.12pt 어긋난다. 예측은 새 Print 전에 정했으며 원본 편람·LineSeg·PDF를 고치지 않았다.
비등방 0.9×0.8 그림을 쓰는 기존 본문 원점의 별도 저장·Print 대조군도 보존한다.

다음 진단은 `실제 Column 원점 → 소유 글줄의 상대 bbox → 각각 장치 정수 변환 →
PagePaintContext의 물리 종이 단위 → Image 및 image_frame_owner 테두리`를 대조한다.
문서·쪽·그림 ID나 점수로 분기하지 않는다. 종이 기준 floating·앞쪽 crop·별도 변환·셀/머리말의
다른 좌표 소유 계약을 본문 Column 계약으로 간주하지 않는다. 같은 좌표 소유 경로가 실제로
적용되는지 확인하고 대조군과 영향 쪽에서 방향을 검증한 뒤 공통 제품 로직을 보정한다.

새 alpha 진단에서는 원본 알파와 고정 16×16 Bayer 순서만으로 독립 Print의 1비트 mask를
721,536픽셀 전부 재현했고, mask 밖 RGB를 흰색으로 전달한 별도 색 평면의 JPEG RGB도
독립 PDF와 일치했다. 상수 alpha의 tiled stencil·mask 표본 소비 계약은 아직 입증하지
않았으므로 일반 투명 PNG에 이 결과를 적용하지 않는다. 진단 문서·PDF·비공개 분석은
`output/`에만 보존하며 제품 구현 완료로 세지 않는다.

#### 실제 원본 소비 진단과 공통 경로 연결

`a2e14ae3bb`의 고정 library로 독립 대조군 5개와 편람 28쪽씩을 실행했다.
원래 경로의 편람 56 PNG는 실제 CLI와 RGB 전체가 같았다. 실제 Column 원점과 상대
위치를 각각 변환하고 같은 그림의 외곽선 소유자도 함께 이동한 진단은 편람 14조건 개선,
42조건 동일, 하락 0건이다. 두 형식 모두 92·94·95·163·263·273쪽은 raw 100%·미대응
0픽셀이다. 255쪽은 두 형식에서 각각 657→265픽셀로 줄었으며 alpha 잔여는 유지한다.

독립 정규 저장본의 원래 phase와 종이 기준 그림은 각각 100%·99.98973%를 유지했다.
0.9×0.8 그림의 본문/종이/여백 변경 대조군은 각각 94.80134%·80.82043%·81.90630%로
기존과 같았다. 좌표 예측은 독립 PDF 연산자로 확인했지만 낮은 전쪽 점수를 좌표 보정
성공으로 덮거나 이 입력을 정식 회귀로 신규 등록하지 않는다. 이 진단은 제품 빌드가 아니다.

제품에서는 `LayerBuilder::build_children → PagePaintContext::for_children`이 실제
단 원점을 상속하고, floating·셀·글상자·머리말·각주·변환 그룹에서 해당 소유를 끊는다.
`image_frame`은 원본 bbox를 보존한 채 원점과 상대 위치의 장치 정수 합을 기존 물리
PrintPaper 단위로 발행한다. `ImagePaintFrame::border_frame`이 같은 결과와 원래 pen
확장을 소비하므로 외곽선에 별도 좌표 보정을 복제하지 않는다. 앞 crop·별도 그림 변환·
본문 단을 소유하지 않는 그림과 비2024 경로는 기존 계약을 유지한다.

진단 원장은 ignored `handbook-inline-anchor-diagnostic-a2e14/result.json`, 원점 예측은
`bitmap-inline-body-anchor-controls/source-predicted-origin-comparison.json`이다.
이 제품 연결 뒤 Native 집중/전쪽·기존 그림 대조군·표준 optimized fresh WASM으로 실제
출력을 다시 검증한다. 기존 JPEG 단계 전쪽 TSV를 새 좌표 후보의 결과로 바꾸지 않는다.

#### `4bb17e9f9c`의 실제 Native 전쪽 검증

고정 checkout·공유 target·jobs 8의 release-test/native-skia 빌드는 372.389초·exit 0이다.
CLI SHA-256은 `3e243605d981d4d03bbd15d1874d3a48e818ef9d4457f417914b6ebb6d0bdd6e`다.
실제 집중 56조건은 앞선 source 기반 진단과 RGB 전체가 같고, 14조건 개선·42조건 동일이다.
같은 후보의 두 원본 전 766쪽을 새로 출력해 383쪽씩의 대응과 PNG/입력/PDF 해시를 확인했다.

| 입력 | Native 최저 raw % | 미대응 0픽셀 쪽 | 미대응 픽셀 합 |
| --- | ---: | ---: | ---: |
| 편람 HWPX | 99.36039 | 255 / 383 | 12385 |
| 편람 HWP | 98.93537 | 265 / 383 | 11306 |

직전 `a2e14ae3bb` 대비 두 형식 각각 11쪽, 총 22쪽이 개선됐고 하락·미대응 픽셀 증가는
0건이다. 각각 91·92·94·95·163·262·263·265·273쪽이 미대응 0픽셀이 됐고, 93쪽은
5→2개, 255쪽은 657→265개다. 263쪽의 실제 Native review를 직접 확인했다. 두 그림,
외곽선과 뒤 문단의 소속·순서·내용을 유지하며 raw 100%를 RGB 전체 일치로 확대하지 않는다.

기존 phase/color/faint 3개를 2022·2024 모드에서 직전·새 CLI로 실제 실행한 6쌍도 RGB가
전부 같다. 이 결과는 독립 reference의 점수를 새로 판정한 결과와 구분한다. 실행 원장은
`handbook-inline-anchor-candidate/{native-focus-result,whole-native-exact-audit,
whole-native-comparison,actual-vs-source-derived-diagnostic}.json`과 `paper-controls/result.json`이다.
Native TSV는 `native/handbook-{hwp,hwpx}/whole-score/silhouette.tsv`에 있다.

표준 최적화 fresh WASM은 같은 `4bb17e9f9c`에서 빌드 중이다. Native 성공을 WASM 또는
전쪽 raw100 완료로 세지 않는다. 기존 실패 검사와 최종 필수 검증도 아직 완료되지 않았다.

후속 표준 fresh WASM 빌드는 wasm-opt까지 20분 29초·exit 0이다. 실제 브라우저의 편람
집중 56조건과 팔레트 대조 1조건을 통과한 뒤, 같은 패키지에서 전 766쪽을 새로 출력했다.
WASM SHA-256은 `5819b18df6575ec41569180db28dda4bab99220014bf7077fb4c14dc1f5d1d65`다.
HWPX/HWP의 최저 raw는 각각 99.24283% / 98.93468%, 미대응 0픽셀은 254 / 263쪽,
미대응 픽셀 합은 13015 / 11861개다. 직전 후보보다 22조건 개선·744조건 유지이며 하락과
미대응 픽셀 증가는 없다. 전체 1,532결과의 raw 100%·미대응 0픽셀은 1,037결과다.
새 TSV는 `wasm-whole/handbook-{hwp,hwpx}/whole-score/silhouette.tsv`에 있다.

94·163쪽은 두 형식·양쪽 backend 모두 raw 100%지만 실제 3배 확대에서 외곽선이
Print보다 옅었다. 기존 그림 외곽선 회귀 4건은 같은 고정 library에서 여전히 실패한다.
중심선 bbox와 그림 bbox의 동일성 대신 소유·중심·잉크 포함 관계로 바꾼 진단 초안은
4건 통과했으나, 보이는 선폭 차이를 먼저 보정해야 하므로 정식 검사에는 반영하지 않았다.
raw 100%를 직접 판독의 면제로 쓰지 않는다. 관련 원장은 `border-direct-review/`와
`logs/group-picture-border-{old,proposed}-test.log`이며 전쪽 100% 목표와 최종 검증은 계속 남는다.

#### alpha 일반화 전의 추가 반례

255쪽의 원본 색 평면·고정 Bayer16 mask와 공개 PDFium 16.16 표본/정수 합성으로 해당
그림 영역을 재생한 진단은 독립 Print와 RGB 전 픽셀이 같다. 그러나 원래 alpha 전 256값
대조군의 색 평면까지 같은 방식으로 일반화하면 두 JPEG에서 각각 16만 픽셀 이상이 다르다.
참조의 718열 창을 압축 전에 반영해도 해결되지 않는다. 단순 PARGB floor 왕복 가설도
개선되지 않아 제품에는 적용하지 않았다. mask의 721,536픽셀 일치와 색 전달의 미입증 범위를
구분한다.

16색·상수 alpha 1/17/203의 추가 독립 Print는 실제 원본 RGB와 최종 색 평면의 차이를
압축 오차 없이 관측하기 위한 진단이다. alpha 17은 실제 Skia decoder와 같지만 203은
다르다. 일반화할 색 변환의 근거를 더 확보해야 하며, 단일 편람 그림의 일치만으로 모든
투명 그림의 decoder·mask 계약을 바꾸지 않는다. 이 PDF·분석·실행 원장은 ignored
`alpha-p255-public-contract/`, `alpha-grid-color-plane-controls/`,
`alpha-lossless-color-controls/`, `alpha-constant-color-contracts/`에만 보존한다.

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
공급한다. 팔레트는 변환 전 원본의 보이는 창에서 먼저 판정하고, truecolor 전달로
선택한 창에만 Print IDCT·RGB555·Print JPEG 재압축 결과를 함께 소비하게 한다.
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

## 팔레트 판정 순서의 반례와 보정

`d5050daa77`의 Native 전쪽 검사에서 90쪽 사진은 두 형식 모두 raw 100%로 개선됐지만,
Q 아이콘을 쓰는 페이지가 후퇴했다. 원본 `BinData/image352.jpg`는 일반 bounded decoder에서
254색·예약색 포함 255색이다. Print IDCT를 먼저 적용하면 256색·예약색 포함 257색이 되어
팔레트 그림을 RGB555 truecolor로 잘못 분류한다. 독립 한컴 PDF 290쪽의 xref 2138은
RGB555 PNG가 아닌 JPEG 압축 전달이며, 이 후보의 대응 그림은 변환 PNG다.
독립 PDF의 JPEG도 재압축된 결과이므로 원본 파일의 바이트 보존을 기대값으로 삼지 않는다.

보정은 문서나 그림 번호로 분기하지 않는다. `원본 bounded decode → 보이는 창의 팔레트
판정 → truecolor일 때만 Print decode → 같은 창 → RGB555 → 압축` 순서를 공통화한다.
팔레트 경로의 원본 색과 기본 decoder를 유지하고, 이미 변환한 PNG에 색 전달을 중복 적용하지
않는다. 재디코딩 전에 앞 버퍼를 해제하며 기존 축·픽셀·할당 제한을 유지한다.

독립 PDF의 Q 그림 전달과 90쪽 사진을 함께 검사하고, 기존 crop·예약색·BMP/PNG 경계도
확인한다. 이후 두 형식의 전쪽 Native/fresh WASM을 다시 비교한다. 이전 766쪽 결과는
실제 후퇴 증거로 보존하며 새 후보의 검증으로 재사용하지 않는다.

## `2cb8223029`의 실제 Native 재검증

공유 target의 상대 경로 dep-info가 이전 devel의 JPEG 아티팩트를 재사용해 첫 빌드가
실패했다. 후보의 변경 소스 91개는 내용 해시를 유지한 채 수정 시각을 갱신했고,
JPEG decoder·encoder·PDF import 패키지의 재컴파일을 확인했다. 다시 실행한
`release-test` Native 빌드는 jobs 8에서 13분 47초·exit 0이다. 캐시 전체를 삭제하지 않았다.

동일한 원본·기존 2024 Print·96dpi의 Native 전 766쪽에서 입력/PNG/PDF 해시와
383쪽씩의 대응을 확인했다.

| 입력 | 전쪽 최저 raw % | 미대응 0픽셀 쪽 | 미대응 픽셀 합 |
| --- | ---: | ---: | ---: |
| 편람 HWPX | 98.59333 | 190 / 383 | 26294 |
| 편람 HWP | 98.59338 | 157 / 383 | 25415 |

이전 완료 후보 `fbe2ea2e54`와 비교하면 282–307쪽의 Q 아이콘 구간은 전부 복원됐다.
90·145쪽은 양쪽 형식 모두 raw 100%이며, 91·246쪽도 개선됐다. 다만 144쪽은 양쪽에서
미대응 픽셀이 438개에서 439개로 1개 늘었다. 이 차이도 전쪽 100% 목표의 잔여 항목으로
유지한다. 최신 기준 `d9fd85ed5b`와 비교한 0.1%p 이상 하락은 없다.

원본 두 형식의 Q 전달 검사는 수정 전 2건 실패·수정 후 2건 통과다. 독립 PDF 290쪽의
Q 그림은 **2개**다. 초기 진단에서 배경 3개와 혼동한 개수 assertion을 독립 PDF의 실제
인스턴스로 정정했다. 90쪽 최종 Print JPEG의 독립 RGB 대조 2건도 통과했다.
이 실행은 진단용 consumer 검사이며 정식 회귀 신규 등록을 대신하지 않는다.

## 기존 팔레트 검사 4건의 재검토

`verified_print_bitmap_visible_palette.rs`와 `verified_hancom2024_reserved_palette.rs`의
4건은 수정 전 `d5050daa77`에서도 동일하게 실패한다. 원본 3개·독립 Print의 Native 점수는
99.99862%·100%·100%이고 이번 팔레트 순서 수정 전후의 PNG 바이트도 같다.

새 Print 전달은 장치별 JPEG와 잘린 창을 게시하므로, 과거 `catmullRom` 마커·원본 RGBA
불변·모든 출력의 bin 키 재조회 전제를 그대로 적용할 수 있는지 확인해야 한다.
독립 PDF의 23개 그림 인스턴스와 최종 payload를 대조하면 22개는 크기와 RGB 전 픽셀이
같지만 위쪽 crop 1개는 실제 RGB 차이가 남는다. 이 차이를 먼저 보정하고 독립 기대값과
현재 Native/fresh WASM의 시각 근거로 기존 검사를 정리한다. 실패를 무시하거나 현재
구현의 값을 그대로 기대값으로 삼지 않는다.

## 기준과 후보의 전쪽 Native/fresh WASM 검증

새 devel `d9fd85ed5b`의 표준 release WASM은 jobs 8·codegen units 16으로 wasm-opt까지
32분 56초·exit 0이다. 실제 브라우저 전 766쪽과 출력 해시를 확인했다. HWPX/HWP의 최저
raw는 각각 92.51660% / 92.51694%이며 이전 기준 캐시와 페이지별 점수도 모두 같다.
devel의 Kitty·Rust 오류 consumer 검사 9건도 통과했다.

후보 `2cb8223029`의 표준 release fresh WASM도 jobs 8·codegen units 16으로 wasm-opt까지
29분 35초·exit 0이다. 빌드 뒤 갱신되는 Studio JavaScript를 소스 변경으로 오인한 진단
wrapper의 보존 단계를 정정했다. 변경 파일이 측정한 `pkg/rhwp.js`와 바이트까지 같음을
확인하고 패치를 보존한 뒤 검증 checkout의 추적 파일을 복원했다. 재빌드 없이 manifest의
SHA와 일치하는 패키지를 고정했으며, 검증 checkout은 clean이다.

같은 원본·독립 Print·96dpi의 후보 전 1,532쪽에서 입력/PNG/PDF 해시와 원본별 383쪽을
확인했다. 모든 TSV 원문과 정수 오차 픽셀 감사를 대조했다.

| 입력 | backend | 전쪽 최저 raw % | 미대응 0픽셀 쪽 | 미대응 픽셀 합 |
| --- | --- | ---: | ---: | ---: |
| 편람 HWPX | Native | 98.59333 | 190 / 383 | 26294 |
| 편람 HWP | Native | 98.59338 | 157 / 383 | 25415 |
| 편람 HWPX | fresh WASM | 98.60005 | 208 / 383 | 26473 |
| 편람 HWP | fresh WASM | 98.60022 | 186 / 383 | 24991 |

고정한 새 devel `d9fd85ed5b`보다 0.1%p 이상 하락한 쪽은 네 경로 모두 없다. 새 devel에서
실제로 다시 산출한 Native/fresh WASM TSV도 과거 기준 캐시와 전 1,532쪽의 점수가 같다.
후보의 raw 100%·미대응 0픽셀은 741 / 1,532쪽이며 이전 후보의 735쪽보다 6쪽 늘었다.
나머지 791쪽은 전쪽 100% 목표의 잔여 범위다.

기존 팔레트 대조군 3개도 같은 패키지로 fresh WASM을 실행하고 review PNG를 직접 확인했다.
최저 raw는 Native와 동일한 99.99862%·100%·100%다. 높은 점수가 위쪽 crop의 남은 차이를
면제하지 않는다. 기존 검사 4건의 보정, 전쪽 100%, 페이지별 중요 영역의 ±2px 보호와 최종
필수 검증은 아직 완료되지 않았다. 이 중간 검증으로 PR 준비 완료를 선언하지 않는다.

TSV·실행 로그·독립 payload 대조와 빌드 receipt는 모두
`output/pr-review/issue8153-handbook-raw100-20261022/` 아래에 보존한다. 이번 단계의
Native 원본 TSV는 `palette-order-2cb8223029/native/handbook-{hwp,hwpx}/whole-score/`,
후보 WASM 원본 TSV는 `palette-order-2cb8223029/wasm-whole/handbook-{hwp,hwpx}/whole-score/`,
새 기준 WASM 원본 TSV는 `rebase-d9fd85ed5b-baseline/wasm-whole/handbook-{hwp,hwpx}/whole-score/`다.
통합 검증 receipt는 `palette-order-2cb8223029/validation-summary.json`이며, 후보 WASM
실행 로그는 `logs/palette-order-fresh-wasm-whole.log`다.
