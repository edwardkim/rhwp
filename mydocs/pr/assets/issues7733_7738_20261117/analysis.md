# #7733·#7735·#7736·#7737·#7738 선행 분석과 검증 계획

## 현재 상태

검토 브랜치는 `task/7733-chemical-spec-outline-decree`, 고정 base는
`22da83586aab1baf960f2159abda347062ecf7ed`다. 다섯 이슈는 모두 열린 상태로 확인했다.
원본·기준의 신원·과거 해결 경로 분석과 Native 기준 전 252쪽 산출을 마쳤다. 사용자 목표는 다섯 문서 모든 페이지의 Native/fresh WASM 각각 98% 이상, 가급적 100%다.
아래 과거 수치를 현재 결과나 해결 완료로 사용하지 않는다. 제품·회귀 검사 변경은 아직 없다.

## 원본과 기준 PDF

[입력 감사](input-audit.json)의 다섯 원본과 PDF는 각 이슈에 기록된 SHA-256과 모두 일치한다.
PDF 전체는 각각 103·71·3·74·1쪽이며 합계 252쪽이다. 기존 독립 기준을 그대로 사용한다.
특히 사용자가 지정한 `pdf/한글문서파일형식_5.0_revision1.3-hancom-official.pdf`는
공식 배포 PDF 그대로 유지하며 MCP로 재산출하거나 수정하지 않는다.
HWP5/HWPX는 확장자만으로 구분하지 않으며 시행령의 `.hwpx` 입력은 실제 HWP5다.

| 이슈 | 대상 | 원 이슈에 기록된 미완료 | 현재 확인할 범위 |
| --- | --- | --- | --- |
| [#7733](https://github.com/edwardkim/rhwp/issues/7733) | 화학제품 표시 기준 축소 HWP | 76·78쪽 캡션·표 점유와 후속 표 배치 | 같은 축소본 전 103쪽과 원문 내용 소속 |
| [#7735](https://github.com/edwardkim/rhwp/issues/7735) | 파일 형식 5.0 revision1.3 배포용 HWP | 69/71쪽 불일치, 표 이어받기·머리말 누락 | 공식 HWP/PDF 전 71쪽, 표·머리말·발행정보 |
| [#7736](https://github.com/edwardkim/rhwp/issues/7736) | PR4093 개요 패널 데모 HWPX | 3쪽 표 셀과 뒤 개요 겹침·후속 문단 전진 | 같은 합성 원문 전 3쪽, 개요 getter·실제 paint |
| [#7737](https://github.com/edwardkim/rhwp/issues/7737) | 가상융합산업 시행령 78494 | 75/74쪽 불일치, 그림·표·참고 상자·본문 넘침 | 같은 원문 전 74쪽, 19·37·59·63쪽 경계·전체 소속 |
| [#7738](https://github.com/edwardkim/rhwp/issues/7738) | PR4093 최소 표 셀 번호 HWPX | `2. 표 셀 번호 문단`과 뒤 `3. 요구사항` 겹침 | 전 1쪽과 번호·표 점유·뒤 문단 소속 |

## 원 해결과 통합 경로

- #7733의 원 결함 [#6782](https://github.com/edwardkim/rhwp/issues/6782)는
  [원 PR #6796](https://github.com/edwardkim/rhwp/pull/6796) →
  [통합 #6847](https://github.com/edwardkim/rhwp/pull/6847)에서 셀 그림 오프셋을 보정했다.
  구현 `4d504fecd4`와 축소 공개 fixture·계약 보완 `44830b2c1e`를 현재 입력과 대조한다.
  그 결함의 종료와 축소본 전체 피델리티는 구분한다. 별도 전체 원문으로 입력을 바꾸지 않는다.
- #7735의 아래아 원 결함 [#2099](https://github.com/edwardkim/rhwp/issues/2099)는
  [#2128](https://github.com/edwardkim/rhwp/pull/2128)에서 해결했고 후속
  [#3563](https://github.com/edwardkim/rhwp/pull/3563)에서 제품명 표시를 검증했다.
  현재 `print_alpha_source_grid.rs`에는 2026-10-28 전 71쪽 Native/fresh WASM 최저
  95.31577/95.19516%라는 후속 기록이 있다. 이 기록은 이후 개선의 단서이며 현재 전쪽 TSV를 대신하지 않는다.
  원 이관 `eb94eba73d`의 실제 제거 범위와 현재 검사를 구분한다.
- #7736·#7738의 [원 PR #4093](https://github.com/edwardkim/rhwp/pull/4093)는
  `c119e5db5829df2e27850258500e5e2dd74a0015`로 병합됐다. 원 review는 표 셀의 번호를
  renderer와 같은 순서로 소비하는 getter/탐색 계약을 다룬다. 뒤의 시각 이관
  `93f62ee4fd`·`d01ab7eb21`는 배치 해결을 의미하지 않는다. 원문 합성 출처를 보존하며
  getter PASS와 독립 Print의 실제 위치·겹침 여부를 각각 확인한다.
- #7737의 원 결함 [#6776](https://github.com/edwardkim/rhwp/issues/6776)는
  [#6853](https://github.com/edwardkim/rhwp/pull/6853) →
  [통합 #6889](https://github.com/edwardkim/rhwp/pull/6889)에서 처리했다.
  `b1127b3520`의 TAC 그림 cell-unit 회계와 `64f33cfc1c`의 HWP5 계보·표 뒤 간격 보정을
  현재 측정·배치·paint 경로와 대조한다. 이관 `7336af268e`·`9671197f1`와 후속 5검사 제외는
  전쪽 개선이 아니므로 현재 결과로 대체 여부를 판단한다.

## 실행 순서와 완료 조건

1. 원본 글꼴 일반 환경에서 같은 base Native 전체 252쪽 TSV와 전체 쪽수를 산출한다.
   과거보다 나아진 쪽과 남은 실제 결함을 같은 기준 PDF·2px raw로 구분하고 대표 review·overlay를 함께 판독한다.
2. 남은 결함만 공통 측정·배치·paint 계약으로 보정한다. 원본·기준·문턱·쪽수 기대를 바꾸어 통과시키지 않는다.
3. 최종 source의 Native/fresh WASM 전체 TSV와 내용 소속·누락·중복·계수·원래 지적 경계를 확인한다.
   사용자 추가 지시에 따라 다섯 원본의 **모든 페이지를 Native/fresh WASM 각각 98% 이상, 가급적 100%**로 맞추고 직접 판독한다. 완료 페이지 양쪽 98% 이상인 중요한 경계는
   독립 Print의 ±2px 좌표 및 소속 관계로 회귀를 보강한다. 그 외는 원본·관계 계약으로 보호한다.
4. 회귀는 `tests/cases` 기존 자동 수집 경로에 반영한다. 회귀·필수 lint/build·Native Skia는 최소 글꼴 환경,
   보호 Visual Sweep은 검증된 원본 글꼴 환경으로 실행한다. Cargo 순차·jobs8·test-threads8·capture 총8을 유지한다.
5. 보호 문서의 공식 표시값 하락 금지와 개선된 보호값 갱신, 명시적 편람·실제 소비 경로를 검증하고
   PR·CI·병합 이후 다섯 해결 이슈와 #7445의 해당 항목을 실제 완료 증거에 따라 갱신한다.
   #7445 전체 해결이나 다른 하위 이슈의 종료로 범위를 확장하지 않는다.

임시 로그·진단 driver는 `output/pr-review/issues-7733-7738-20261117/`에만 둔다.
원본·PDF와 최종 TSV·대표 raw/PDF/review/overlay·원인 및 결과 기록은 정식 증적에 보존한다.

## 고정 base의 실제 Native 전쪽 결과

[전체 원본 TSV·페이지별 정밀 수치·생산자](base-native-results.json)를 보존했다.
현재 모든 문서의 전체 쪽수는 PDF와 같고, Native 252쪽 중 **27쪽이 98% 미만**이다.

| 이슈·문서 | 쪽수 | Native 최저 raw | 98% 미만 쪽수 | 상태 |
| --- | ---: | ---: | ---: | --- |
| #7733 화학 축소본 | 103 | 89.83308% | 12 | 보정 필요 |
| #7735 공식 형식문서 | 71 | 95.30311% | 3 | 보정 필요 |
| #7736 개요 데모 | 3 | 100.00000% | 0 | Native 관측 충족, fresh WASM·내용 판독·회귀는 별도 확인 |
| #7737 시행령 | 74 | 64.73203% | 12 | 보정 필요 |
| #7738 최소 개요 | 1 | 100.00000% | 0 | Native 관측 충족, fresh WASM·내용 판독·회귀는 별도 확인 |

직접 review/overlay에서 화학 79·80쪽의 쪽번호 누락과 일부 표 선 높이 차이,
공식 형식문서 8쪽 제목의 가로 원점 차이, 시행령 30쪽의 혼합 굵기 본문 줄바꿈·뒤 문단 위치 차이를 확인했다.
아직 원인을 글꼴 또는 조판으로 확정하지 않았다. 원본 속성·독립 PDF 좌표·실제 선택 프로그램과
측정/배치/paint 경로를 분리해 추적하고, 결과를 보정 전 기록으로 유지한다.
