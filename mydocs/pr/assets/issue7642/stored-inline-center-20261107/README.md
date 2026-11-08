# 과학 문서의 저장 가운데 정렬점 — 2026-11-07

`samples/exam_science.hwp`의 인라인 수식이 있는 가운데 정렬 문단에서, 저장된 줄의
`baseline_distance == text_height / 2`는 문자 기준선이 아니라 세로 정렬점이다.
이 값을 문자 기준선으로 사용해 4쪽 조건 상자의 문자가 독립 한컴 Print보다 약 5.4px 올라갔다.
작은 수식 줄에서는 최소 문자 기준선 clamp가 원래 정렬점을 덮어쓰고 있었다.

## 원인과 적용 경계

선행 해결 `f70a8c937d3`(#8316 메인터너 보정)은 저장 인라인 **표** 줄의 가운데 정렬점과
문자 기준선을 구분했다. 이번에는 같은 문제의 수식 소비 경로를 보완한다.
#8555의 `8a4295cdf9`가 새 줄 구성에 전달하는 위·가운데·아래 정렬점은 유지한다.
HWP5 문단 모양의 세로 정렬 속성과 저장 LineSeg를 해석하며, 원본 줄 높이·기준선·내용을
고쳐 화면에 맞추지 않는다.

소비 경로는 `ComposedLine의 저장 정렬점 → paragraph_layout의 문자 기준선 투영 →
RunEmitVars → 인라인 EquationNode의 논리 상자 원점 → 공통 render tree의 Native/WASM 출력`이다.
문자는 자기 ascent, 수식은 자기 논리 상자의 중심을 같은 저장 정렬점에 맞춘다.
재구성한 줄·수식 이외의 TAC 개체·공백만 있는 줄은 기존 경로를 유지한다.
기존 인라인 표 기준선 분기도 그대로 보존한다.

## 독립 기준과 전쪽 대조

입력·PDF 전체 SHA, producer와 asset SHA는 검증 원장 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7642/stored-inline-center-20261107/validation.json`)에 기록한다.
PDF는 기존 `pdf/issue7642-exam-print/exam-07-exam_science-hwp-print-2020.pdf`를 재사용했다.
96 DPI·2022 호환·PDFium·2px raw 조건이다. 글꼴을 공급한 전쪽 검증이며 최소 글꼴 검증의
완료로 확대하지 않는다. Mac fresh WASM은 locked wrapper의 `--target web --out-dir pkg --no-opt`
로컬 대체 빌드다. Root pkg와 Studio public의 JS/WASM 해시도 대조했다.

| 쪽 | Native 전 | Native 후 | fresh WASM 전 | fresh WASM 후 |
| --- | --- | --- | --- | --- |
| 1 | 99.88919% | 99.90105% | 99.89494% | 99.90657% |
| 2 | 98.87323% | 99.19496% | 98.89846% | 99.22469% |
| 3 | 99.20876% | 99.92920% | 99.17192% | 99.92914% |
| 4 | 97.11295% | 99.40548% | 96.91722% | 99.42983% |

전 4쪽을 각각 새로 산출했고 누락·쪽수 차이가 없다. 전체 원문은
[Native 전 TSV](before-native-silhouette.tsv)·[후 TSV](after-native-silhouette.tsv),
[WASM 전 TSV](before-wasm-silhouette.tsv)·[후 TSV](after-wasm-silhouette.tsv)다.
2·3·4쪽의 수식·조건·선택지와 앞뒤 문장을 직접 대조했다. 조건 상자의 문자·분수 정렬은
복구됐으며 화학식과 일부 글리프의 가장자리·농도 차이는 남는다. 100% 일치로 보고하지 않는다.

| 출력 | 보정 전 | 보정 후 |
| --- | --- | --- |
| Native 4쪽 | [review](before-native-p004-review.png) · [overlay](before-native-p004-overlay.png) | [review](after-native-p004-review.png) · [overlay](after-native-p004-overlay.png) |
| fresh WASM 4쪽 | [review](before-wasm-p004-review.png) · [overlay](before-wasm-p004-overlay.png) | [review](after-wasm-p004-review.png) · [overlay](after-wasm-p004-overlay.png) |

## 회귀 검사

`tests/cases/issue_7642_stored_inline_equation_center.rs`에 두 검사를 추가한다.

- **고정형** `centered_text_baselines_match_independent_print`: 독립 PDF에서 읽은 다섯 문자
  기준선을 96 DPI 쪽 좌표에서 ±2px로 검사한다. 후보 좌표를 기대값으로 복사하지 않는다.
- **관계형** `inline_equations_keep_source_centers_and_content`: 조건의 다섯 수식과 저장 줄의
  정렬점, 원본 셀·문단 소유와 수식 내용·순서·누락·중복을 검사한다. 같은 쪽의 수식만 있는
  선택지 다섯 개는 기존 기준선 정렬을 유지해야 한다. 저장 HWPUNIT 양자화 한 단위는 허용한다.

같은 정식 검사 원문을 frozen 변경 전·후 rlib에 연결해 **2 FAIL → 2 PASS**를 확인했다.
정식 suite 자동 수집 뒤 새 검사와 선행 정의·칸 편집·그림 이동·위임전결·잘린 그림 창·
개인정보 이어받기·글상자·기존 인라인 표 정렬 대조군까지 **62 PASS/0 FAIL**을 확인했다.
Mac 최소 글꼴 sandbox에서 `native-skia`, jobs 8·test threads 8로 실제 실행했다.
이는 글꼴 미설치 환경에서 해당 좌표·소유 계약이 유지된다는 검사이며 전체 문서의 최소 글꼴
시각 수용과는 구분한다. 로그는 ignored output의 `science-centered-inline-repair/focused-nextest.log`다.
이 단계의 통과를 브랜치 전체 회귀·전체 문서·Studio 실제 소비 경로 완료로 확대하지 않는다.
