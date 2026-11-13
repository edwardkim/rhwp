# #8957 Print 위치·윤곽 차이 분석과 보정

Issue: #8957. 상위 #7774. 작업 기준: `4c876654d86d071d342fc78706c51cfff6ddbde4`.

이 문서는 1단계의 역사 기록이다. 최종 상태는 [2단계 기록](task_m100_8957_stage2.md)을 따른다.

1단계 종료 상태: 딸기 HFT 굵기 전진 누락을 보정했고 Native/fresh WASM 독립 대조와 집중 회귀 57개,
형식 검사 및 집중 Clippy를 통과했다. 제공 표지의 연결 그림 표시 차이로 전쪽 게이트는 미달이며
#8957 종료·PR 제출은 보류한다. 상세 결과는 아래 마지막 절과 정식 tracking 증거를 따른다.

## 입력과 독립 근거

사용자가 2026-11-11 기준 PDF의 생성 환경을 한글 2024와 제공한 `HGSTRBRY.otf`라고 확인했다.
원본 HWPX·OTF·Microsoft Print PDF의 SHA는 #8957 본문과 이전 측정 원장에 고정했다.
현재 작업은 최신 devel의 `task/8957-print-frame-outline`에서 진행하며 산출물은 T7의 routed output에 둔다.

## 선행 수정과 보존 조건

- #8730 → #8734 → 통합 #8923 / `e118de8aba`: 그림 여백의 저장 줄 이중 적용 제거. 원본 14자의 논리 원점 -4.26pt, 기준선·상대 간격 유지가 현재 PASS다.
- #8857 / `19f0847829`: 같은 원본 HFT 프로그램의 Print 프레임 복원. `style_resolver`의 소유 선택 → `TextStyle` → `paper_viewport`의 paint_transform이 소비 경로다. sample10 509·757쪽 등의 기존 개선을 보존한다.
- #8430 → #8605 / `373ec6fa58`: 보통 프로그램의 합성 굵기 fill+stroke 0.02em. 원본 CFF 선택과 화면·Print·실제 Bold 반례를 보존한다.

## 원인 분리 계획

1. 기존 devel과 동일 SHA 바이너리의 provenance를 확인해 재사용한다. 원본 표지의 논리 원점, 출력 MediaBox, Print 행렬, 잉크 경계를 별도로 대조한다.
2. 원본의 첫 24개 최상위 문단만 유지한 표지 대조본과 charPr 23의 bold만 제거한 보통 대조본을 만들어 한글 2024에서 독립 Print한다. 합성 대조본의 출력으로 원본 전체 검증을 대신하지 않는다.
3. 같은 OTF 윤곽·보통/굵게 원본 출력·독립 PDF의 글리프와 마스크를 대조해 좌표·굵기·힌팅/래스터 기여를 분리한다. 특정 문서·문구·관찰 오프셋에 맞춘 수치 보정은 추가하지 않는다.
4. 확인된 일반 계약을 공통 로직에서 보정하고 그 계약을 소비하는 Native/SVG/WASM에 반영한다. 회귀 등록의 시각 선행 조건과 기존 보호 페이지를 함께 검증한다.

## 진행 기록

- T7 routing: linked 3, pending 0. 기존 주 작업공간의 미커밋 변경을 보존했다.
- `cover-bold.hwpx`: 원본 section의 최상위 p0~p23을 유지하고 뒤 문단만 제거했다.
- `cover-regular.hwpx`: 위 입력의 charPr 23에서 bold만 제거했다. 원본 ZIP을 변경하지 않았고 두 파생본의 CRC를 확인했다.
- 독립 출력과 구현 보정·최종 검증은 진행 중이다. 완료 또는 PR 제출 가능으로 판정하지 않는다.

## 2026-11-12 원인 분리 결과

2026-11-12 원격 `devel`을 다시 조회해 기준 SHA `4c876654d86d071d342fc78706c51cfff6ddbde4`가 최신임을 확인했다.
제품 코드·테스트·기대값은 아직 변경하지 않았다. 아래는 원인 분리용 진단이며 수정 완료 또는 제출 검증이 아니다.

### 독립 대조 입력과 Print 출처

| 입력 | SHA-256 | 한컴 2024 Print job |
| --- | --- | --- |
| 첫 24개 문단을 유지한 굵게 표지 | `625a311578ced7a91725998d004590159e0886c5000346279060d7e649b21ab8` | `40e8173c-bb55-4aef-8625-dd8749dcf0a0` |
| charPr 23의 bold만 제거한 보통 표지 | `79e868eefdec63873ab08cb918e85dd7e802f51c828d68fb45eebac3c5ace056` | `9a0f46f4-1546-4ba6-b65b-f53dd87f93ad` |

한컴 제품은 `13.0.0.3901`, 실행은 `pdf_print_method=0`,
`pdf_output_mode=frame_print_to_pdf_ex_one_up`, 전처리 `none`이다. 각각 1쪽이다.
이 경로는 사용자가 제공한 **Microsoft Print to PDF** 경로와 구분한다.
서비스의 등록 글꼴 2개가 정상이라는 상태만으로 사용자 OTF와 같은 프로그램이 선택됐다고 단정하지 않는다.
대신 대상 글자의 Type3 윤곽을 공급 OTF와 읽기 전용으로 대조했다.

### 위치·윤곽·출력 경로의 분리

- 굵게 표지 파생본의 Native SVG PDF는 원본 첫 쪽 출력과 SHA-256까지 같다
  (`0ce0cb4e85da8f71583aa64255a8aeb60307577ad6f3b35fcc1791e8fc938936`).
  따라서 표지 재현 과정이 기존 현상을 바꾸지 않았다.
- Native SVG PDF와 독립 한컴 자체 PDF의 14개 글자 원점 최대 차이는 굵게에서 x `0.098404pt`,
  y `0.020752pt`, 보통에서 x `0.099045pt`, y `0.020752pt`다.
  한컴 자체 PDF의 정수 600dpi 장치 원점과 현재 연속 좌표의 차이는 남지만,
  Microsoft 출력에서 관측한 약 1pt 차이와 같은 원인으로 확정하지 않는다.
- 독립 한컴 자체 PDF의 13개 고유 글자에서 보통/굵게 CharProc의 경로가 동일하고,
  마지막 `f`만 `20 w B`로 바뀐다. FontMatrix는 0.001이므로 `0.02em` fill+stroke다.
  현재 공통 합성 굵기 계약을 그대로 지지한다.
- 공급 OTF CFF와 독립 Type3 경로의 곡선을 각 구간 1,001점으로 표본화해 양방향 최근접 거리를 비교했다.
  최대 `0.281326/1000em` 미만이다. 이는 곡선 분할과 표본 간격을 포함하는 근사 비교이며
  바이너리 동일성 또는 실제 선택 파일의 증거로 승격하지 않는다.
- Microsoft PDF의 대상은 600dpi 이진 이미지 마스크다. 글꼴 프로그램·텍스트 원점·굵기 연산은
  남아 있지 않으므로 해당 PDF만으로 내부 알고리즘을 단정할 수 없다.
- **진단 가설**: 공급 OTF의 보통 윤곽에 가로 0.05em에 대응하는 7~8px 확장을 적용했다.
  글자별 잉크 상자를 정렬하고 ±1px 위상만 대조한 8px 가설의 IoU는 93.23~97.82%였다.
  보통 윤곽은 59.26~76.43%였다. 가로 합성 굵기 가설을 지지하지만 위치를 정렬한 국소 진단이므로
  Visual Sweep 점수·위치 해결·일반 구현 규칙의 근거로 사용할 수 없다. 이를 제품 상수로 추가하지 않았다.
- Microsoft의 [합성 굵기 설명](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/ne-dwrite-dwrite_font_simulations)은
  widening 알고리즘의 존재만 설명한다. 해당 HFT 출력이 DirectWrite를 사용했다거나
  0.05em·8px가 규정값이라는 근거는 아니므로 구현 근거로 채택하지 않았다.

### 기존 개선의 확인 범위

- 기존/현재 SVG 14자 모두 논리 x 이동 `-4.26pt`, 기준선·크기 유지 검사를 재실행해 통과했다.
- Print SVG와 Screen SVG의 논리 원점이 같고, Print PDF 차이는 기존 공통 행렬로 설명된다.
  행렬 적용 후 PDF 원점의 최대 오차는 `0.000051065pt`다.
- `--compat 2024`의 Native SVG PDF는 기본 2022 호환 출력과 바이트가 동일했다.
- 같은 원본 OTF를 선택한 실제 Native Skia Print PNG를 144dpi로 출력하고 대상 영역을 확대 판독했다.
  Microsoft의 가로로 두꺼운 형태보다 한컴 자체 PDF의 둥근 윤곽에 가까운 동일 경향이다.
- #8857·#8605의 원 해결/통합 근거와 현재 호출 경로는 확인했지만, 보호 페이지 전체 및 fresh WASM
  재실행은 아직 하지 않았다. 제품 수정이 없는 현 단계의 위 진단을 필수 최종 검증으로 대신하지 않는다.
- 새 회귀·fixture·golden을 등록하거나 기존 기대값을 바꾸지 않았다.

### 필요한 독립 대조와 다음 작업

같은 사용자 PC의 한글 2024·제공 OTF·동일 인쇄 설정으로 `cover-bold.hwpx`와
`cover-regular.hwpx`를 각각 **Microsoft Print to PDF**로 출력해야 한다.
보통 출력에서도 기준선 차이가 남는지와 굵게일 때만 획의 가로 확장·원점 이동이 생기는지를 분리한다.
현재 서비스의 `tools/list`를 확인했으나 프린터/드라이버 선택 또는 Microsoft Print 도구가 없다.

이 대조에서 일반 계약이 확인되면 현재 한컴 자체 Print 계약을 보존하는 적용 경계를 정한 뒤
공통 paint/윤곽 로직을 보정한다. 이후 원본 및 정상 대조군의 Native/fresh WASM, 전쪽 TSV,
확대 판독, 관련 기존 회귀와 최종 필수 게이트를 실행한다. 아직 #8957 해결로 판정하지 않는다.

진단 산출물은 이 작업공간의 Git 제외 `output/issue8957/`에 있다.
`controls-measurements.json`, `controls-comparison.png`, `outline-hypotheses.json`,
`outline-hypotheses.png`, `geometry-preservation.json`, `compat-check.json`,
`native-skia.json` 및 한컴 job 원장을 보존했다. 입력 OTF와 비공개 산출물을 커밋하지 않았다.

## 사용자 Microsoft Print 대조본 수신 — 2026-11-12

사용자가 `cover-bold.pdf`와 `cover-regular.pdf`를 제공했다. 두 파일 모두 1쪽,
MediaBox 595.32×841.92pt이며 Producer는 `Microsoft: Print To PDF`다.
각 SHA-256은 `4c21b559619b8737a9b73e93b661960e161460e340ba261fa26b1158786ec03c`,
`6da2f8b6776643cbbc30022f8e41d861c962c7e06a734a107a043d1acd563c86`이다.
사용자가 확인한 한글 2024·제공 OTF 환경과 연결한다. PDF 메타데이터만으로 실제 한글 빌드·OS·배율을
추론하지 않는다. 기준 PDF 인쇄 계약은 특정 드라이버를 한정하지 않으며,
해당 한컴의 파일→인쇄 출력과 입력·출력·환경·설정 provenance를 요구한다.
따라서 Microsoft Print는 배제 사유가 아니며 이 1쪽 대조 입력의 독립 기준으로 사용할 수 있다.
기존 한컴 자체 PDF 정본을 다른 드라이버 출력으로 무조건 교체하는 근거는 아니다.

초기 직접 확대 대조에서 굵게의 가로 확장 차이와 보통에서도 남는 세로 위치 차이를 분리했다.
공통 A4 4763×6735px, 잉크 임계값 120의 보통 문구 bbox는 Microsoft
`[219.47972, 697.03726, 486.08009, 709.28791]pt`, 한컴 자체 Print는
`[219.47972, 696.03720, 485.95510, 708.28786]pt`다.
보통에서도 약 1pt의 세로 차이가 있으므로 이를 굵게 처리만의 결과로 설명할 수 없다.
새 굵게 출력의 bbox는 앞서 제공한 Microsoft 기준과 같다.
이 측정은 드라이버별 용지 크기를 공통 A4에 맞춘 진단이며 정식 Visual Sweep을 대신하지 않는다.
원장과 확대 비교는 `output/issue8957/user-controls-measurements.json`,
`output/issue8957/user-controls-comparison.png`에 있다. 구현과 최종 검증은 계속 미완료다.

## 공통 전진폭 결함 재현 — 2026-11-12

Microsoft 보통/굵게 원시 마스크를 직접 대조했다. 14자 모두 보통 마스크의 오른쪽 8px 확장과
굵게 마스크가 완전히 같고 위·아래 경계 차이는 0이다. 마스크 배치 원점은 rhwp 논리 글자 상단의
600dpi 올림과 굵게/보통 모두 일치한다. 보통 잉크의 세로 차이는 마스크 내부에서 약 7~8px 남는다.
따라서 이 래스터 확대와 내부 세로 이동을 조판 상수로 추가하지 않는다.

별개로 실제 Native Skia와 fresh WASM에서 딸기 HFT의 굵기 전진이 빠지는 결함을 재현했다.
SVG PDF의 원점은 211.506pt이고, 준비된 실제 OTF로 측정하는 경로는 줄 폭을 282.88pt가 아닌
270pt로 만들어 첫 원점을 약 6.44pt 오른쪽으로 옮긴다. 이 값은 4.26pt 저장 줄 여백 보정과 별개다.

선행 `8579e128024bb36e60c340d1078f57d90e47b3a7`의 R52는 실제 주 CFF에 임의 HFT 꾸밈을
더하지 않는 보호다. 이 보호를 유지하면서 독립 Print로 확인한 딸기 HFT의 전진 계약을 등록한다.
기존 신명 중고딕 계약과 같은 원본 선언/실제 글리프 공급자 확인을 공통 측정 경로에서 소비한다.
이름 없는 임의 CFF·TTF 대체·실제 Bold 프로그램·공백·보통 스타일에는 확대 적용하지 않는다.

소비 추적: `legacy_font_advances::has_verified_hft_bold_tracking_source` →
`ResolvedCharStyle.font_families_verified_hft_bold_tracking` →
`TextStyle.source_verified_hft_bold_tracking` + `NativeOtfMetrics`의 실제 glyph 공급 프로그램 →
`char_width_decision_with_hft_tracking` → 문자 전진/줄 폭 → 문단 가운데 정렬 및 실제 원점.
수정은 이 계약의 공유만 보완하며 Print 프레임·윤곽 굵기·그림 여백의 기존 계산은 유지한다.

최초 원본 표지의 전쪽 TSV는 외부 연결 그림을 읽지 못한 placeholder 때문에 Native 8.42~11.14%,
WASM 74.12~74.76%다. PDF는 해당 그림을 표시하지 않는다. 이 결과를 통과로 기록하거나
마스크를 임의로 바꾸지 않는다. 글꼴 계약의 독립 대조와 새 회귀 선행 검증은 별도로 만든
12/18/24pt 보통·굵게 텍스트 표본으로 수행하고 원본의 대상 영역도 계속 직접 비교한다.
기존 sample10 HWP 509·757쪽 Native는 각각 99.90%, 99.55%로 재검증했다.

## 수정 후보와 실제 출력 재검증 — 2026-11-12

앞의 미수정·사용자 PDF 대기 기록은 각 단계의 과거 상태다. 두 PDF는 수신했고,
현재는 `legacy_font_advances`의 확인된 HFT 계약에 딸기/Strawberry를 등록했다.
원본 선언과 실제 glyph 공급 프로그램이 같은 family 판정 함수를 소비한다.
추가 수치, 드라이버 보정, Print 행렬 변경, 윤곽 stroke 변경은 없다.

새 독립 1쪽 대조에서 Native/fresh WASM 모두 raw 89.84% → 100.00%다.
실제 표지 대상 글자 영역도 양쪽 85.47%대 → 100.00%이며, 전체 페이지 게이트와 구분했다.
기존 4.26pt 이동·기준선·크기·Print 원점을 재검사했고,
보통 표지 및 sample10 509/757쪽 PNG는 각 경로에서 수정 전후 바이트까지 같다.
독립 기준 PDF·원본·전쪽 TSV·직접 판독 PNG·해시는
`mydocs/pr/assets/issue8957/tracking/README.md`에 연결했다.

새 회귀는 MIT 합성 CFF의 family/style만 메모리에서 바꾸며 실제 `DocumentCore`의 선언 →
선택 프로그램 → 측정 → 가운데 배치 경로를 검사한다. 확인된 굵기 전진과 동일 중심,
문자 순서·문단 소속·공백 전진 보존, 미확인 선언/프로그램·TTF 선언·실제 Bold 반례를 다룬다.
상용 OTF를 회귀 fixture로 커밋하거나 설치 여부에 따라 건너뛰지 않는다.
기존 R52 소유 경계·Print 프레임·Square 저장 줄·HFT 윤곽 회귀는 유지하며 집중 재검증한다.

제공 표지의 연결 그림/placeholder 차이 때문에 원본 표지 전쪽은 여전히 90% 미만이다.
원본 표지의 새 golden을 등록하지 않았고 #8957 종료·PR 제출 가능으로 판정하지 않는다.
전체 회귀·Native Skia 공식 3종·최종 편람 383쪽 gate 등 PR 제출 게이트는 아직 완료하지 않았다.
GitHub 댓글·push·PR은 수행하지 않았다.

최소 글꼴 환경의 집중 회귀 57개(신규 관계형 2개 포함)와 `cargo fmt --all -- --check`를 통과했다.
새 검사의 합성 이름 치환 길이·타입 오류와 sparse checkout의 입력 누락은 검사 실행 전에 보정했다.
기존 검사를 제거하거나 기대값을 낮추지 않았다. 테스트 모듈의 자동 suite 배정 결과로 실행했으며
개별 CI 호출을 추가하지 않았다.

최종 집중 Clippy(`--lib --test regression_suite_027 --features native-skia --profile release-test -- -D warnings`)도 통과했다.
