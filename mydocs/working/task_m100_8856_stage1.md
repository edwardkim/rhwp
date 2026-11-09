# #8856 잔여 렌더링 보정 — 1단계

Issue: [#8856](https://github.com/edwardkim/rhwp/issues/8856).
기준 `edbd51067730098e830eb5d03aed625eac47b10f`, 브랜치 `task/8856-residual-rendering`.

## 전체 범위와 완료 조건

interview, request, NewYear_s_Day, KTX, policy-flags0, legislation-summary,
report-628, idol-counterexample, clip5587의 9문서군을 모두 다룬다.
원 이슈에 고정된 입력·독립 PDF·판정 문턱을 보존한다. 각 원인별로 공통 구현을 보정하고
Native/fresh WASM 전쪽 TSV, 같은 쪽 review/overlay, 내용 소속·누락·중복을 확인한다.
clip5587의 Native 4쪽 0.14171%p 하락과 정본 보호 TSV의 공식 표시값도 별도로 확인한다.
현재 단계의 집중 검사만으로 전체 이슈 해결이나 PR 준비 완료를 선언하지 않는다.

## 선행 분석과 첫 보정 경계

- 원 기여 PR #8611 → 실제 통합 #8857은 Native 글꼴 공급 상한을 고쳤다.
  request 선택 항목의 오른쪽 이탈을 해결한 PR은 아니다.
- `tests/cases/issue_7190_off_canvas_object_descendant.rs`는 request의 원래 이탈을
  진단기 재현 입력으로 사용하며, 결함 배치를 정상 계약으로 고정하고 있다.
  배치를 복구한 뒤 진단기 검출 범위는 명시적인 이탈을 만든 합성 대조군으로 보존한다.
- 원본 HWP의 광고접촉수단·직업 셀 문단은 `DISTRIBUTE` 정렬이다. 글자처럼 취급하는
  RadioButton 사이에는 공백만 있고 마지막 가시 요소는 양식 개체다. 원시 저장 폭은
  신문 2928, 인터넷홈쇼핑 6228, 홈쇼핑 3753 HWPUNIT이다.
- 현재 배분 계산은 마지막 텍스트 뒤에 양식 개체가 있어도 텍스트의 마지막 간격 하나를
  제외한다. 두 공백에 한 간격 몫을 각각 적용하면서 마지막 선택지를 용지 밖으로 보낸다.
  이미 공통 `LeadingSpacePlan`이 가진 `terminal_inline_object`를 배분 정렬에서도 소비한다.
  개체 폭 자체나 셀·용지 크기, 문서별 좌표는 바꾸지 않는다.
- idol의 외곽선은 독립 PDF 벡터에도 원형 점들이 있다. 축소 PNG가 실선처럼 보인다는
  이유로 원본 `CIRCLE`을 `SOLID`로 바꾸지 않는다. 지름·중심 간격과 중복 도색을 추적한다.

## 실제 검증 결과

2026-11-10 KST. 새 관계형 검사는 보정 전 1 PASS / 1 FAIL에서 보정 후 2 PASS / 0 FAIL로
바뀌었다. 정상 request에는 용지 이탈이 없으며, 명시적으로 이탈시킨 진단기 음성 대조군 등
기존 진단 3건도 모두 통과했다. 선택 항목 13개·원래 순서·셀 소속을 검사한다.

| request 전체 1쪽 | Native raw | fresh WASM raw |
| --- | ---: | ---: |
| 보정 전 | 96.20377% | 이번 단계 미측정 |
| 보정 후 | 99.22562% | 99.20457% |

후보 Native와 WASM에는 같은 글꼴 환경과 실제 웹폰트 프로그램을 전달했다.
WASM은 요청한 6개 face를 모두 로드했다. 앞선 99.42% 측정은 명시적 글꼴 목록이 비었으므로
동일 조건 증거에서 제외했다. Mac no-opt locked wrapper 빌드이며 Docker 최적화 결과가 아니다.
루트 pkg·Studio public·실제 브라우저 초기 GET 응답의 JS/WASM SHA-256이 일치하고,
HTTP 관찰을 넣은 출력과 정본 캡처의 PNG 해시도 같다.

9문서군 전체 219쪽의 Native base/후보와 같은 글꼴 조건의 후보 fresh WASM을 산출했다.
Native 새 하락은 0쪽이고, 두 후보 backend의 90% 미만 쪽도 없다. 이것은 최소 글꼴의 1차
비교이며 나머지 8문서군의 잔여 차이가 해결됐다는 뜻은 아니다. 이전 b768 대비 clip5587의
엄격 하락과 보호 TSV, 새 base WASM, 전체 회귀·필수 gate·실제 소비 경로는 아직 미완료다.

Mac 격리 CoreText 목록은 OS 기본 528 faces, `/System/Library` 밖 0개다. 개인 글꼴은
삭제하거나 비활성화하지 않았다. Chrome의 중첩 sandbox 초기화 실패 때문에 검증용 wrapper가
브라우저 자체 sandbox를 끄되 바깥 `sandbox-exec` 글꼴 차단은 유지한다. 정본 도구·제품의
보호 장치는 수정하지 않았다.

[정식 검증 영수증](../pr/assets/issue8856/stage1-distributed-forms/validation.json)과
[전쪽 비교 TSV](../pr/assets/issue8856/stage1-distributed-forms/nine-groups-219-pages.tsv)에
producer·입력/PDF·글꼴·로그 hash와 제한을 보존한다. 같은 폴더의 before/after Native 및
paired WASM raw·review·overlay를 직접 판독했다. 임시 진단·로그는 `output/issue8856/`에만 둔다.
