# Stage 91 — 한 줄 미주 수용과 셀 도형의 기준 좌표계

## 독립 근거와 묶음 수정 계획

Issue: #7642. 변경 전 source는 `b770b1d2e01ddc3570771c313f9e952a8df4e97a`다.
Stage 90에서 독립 Print 전체 191쪽과 직접 Skia를 대조한 9개 원본 중
all-21/39/41만 미달이다. 전체 55개 원본/853쪽과 관계형 회귀 전환 범위를 유지한다.

### 한 줄 미주의 실제 수용

all-21 HWPX의 미주 내부 pi864는 앞 항목을 포함한 실제 점유가 가용 단 높이
안이다. 자매 HWP와 독립 Print 모두 현재 왼쪽 단에 남긴다. 앞선 분기에서
사용하는 저장 좌표 추정과 실제 결과가 다르며, 실제 query가 한 줄을 제외해
오래된 추정이 단 넘김을 결정한다. 원본과 PDF 해시는 Stage 90 receipt에 고정했다.

`query_reflowed_endnote_fit`은 비어 있지 않은 한 줄도 실제 점유로 Whole/Overflow를
반환한다. 다줄의 Split, 명시적 나눔/keep 그룹/별도 객체 경로는 유지한다.
`initial_fit → measure_endnote_column_with_item → 선행 tail 분기 → 최종 방출`에서
같은 단·같은 앞 항목 목록인 결과만 재사용한다. 빈 단, 새 미주 머리,
실제로 넘치는 한 줄, float, keep-with-next는 각각 기존 별도 계약을 따른다.

### 셀 도형의 PAPER/PAGE 축

`samples/exam_math.hwp` 구역1 문단68의 셀 쪽 번호 도형은 비-TAC,
IN_FRONT_OF_TEXT, 가로/세로 PAPER 기준, 가로 RIGHT·세로 TOP 정렬이다.
선언 크기 4252HU, 가로 offset 5201HU, 세로 offset 5853HU다.
독립 Print 17쪽은 같은 번호를 용지 오른쪽 위에 출력한다. 현재는 셀 원점과
셀 폭을 사용해 본문 쪽으로 이동한다. 이 값은 원본 진단이며 고정 px 회귀값이 아니다.

`build_render_tree/바탕쪽/단 fast probe → 현재 용지·본문 frame →
layout_cell_shape_with_parent_path → layout_shape_object → paint`를 대조했다.
가로 PAPER는 실제 용지 폭, PAGE는 본문 frame을 사용한다. 세로 PAPER는
실제 용지 높이, PAGE는 본문 frame을 사용한다. PARA/COLUMN과 TAC의
기존 셀/문단 흐름, CellPath·객체 layer는 유지한다. 비대칭 여백에서도
용지 폭을 본문 폭·여백으로 역산하지 않는다. 바탕쪽이 없는 머리말도 현재
페이지 frame을 소비하도록 페이지 시작에 먼저 공급하고 기존 fast probe와 공유한다.

## 검증 계획과 미검증 범위

두 원인을 한 배치로 수정하고 fmt·Native/WASM Clippy·Skia check를 순차 실행한다.
동결 Native binary로 앞선 실패 원본 9개를 전쪽 재비교한다. 미주 컷과
쪽 번호/제목을 같은 배율로 직접 판독하고 exam_eng/kor/science/social 및
셀 PARA/TAC 경로를 대조군으로 확인한다. 방향 확인 뒤 Native SVG/fresh WASM
전쪽 TSV와 공식 회귀 게이트를 진행한다.

Native/fresh WASM 90% 이상 및 확대 판독 전에 정식 회귀를 추가하거나
기대값을 변경하지 않는다. source fixture/PDF를 삭제해 통과시키지 않는다.
이 계획 시점에는 구현·빌드·시각 결과 모두 미검증이다.

## 구현 기록

`reflow_fit.rs`는 한 줄을 제외하지 않고 기존 실제 점유 조회로 Whole/Overflow를
반환한다. `paragraph.rs`의 선행 분기도 단과 앞 항목 수 일치를 확인한다.
`layout.rs`는 물리 용지·본문 frame 공급을 공통 helper로 모으고 페이지 시작·
바탕쪽·단 probe에서 같은 값을 사용한다. `table_cell_content.rs`는 비-TAC
도형의 PAPER/PAGE 축을 선언대로 해석한다.

`cargo fmt --all`과 `git diff --check`는 통과했다. 코드 source를 동결한 뒤
선행 타입 검사와 Native 빌드·원본 직접 비교를 순차 실행한다. 다른 필수
회귀·fresh WASM·전쪽 TSV는 아직 미검증이며 정식 테스트 기대값은 바꾸지 않았다.

## 선행 검사와 다음 검증의 실행 상태

source `c6ccb1cb64452d444c154a69545bbc90804e395c`에서 fmt check,
Native Clippy `-D warnings`, WASM lib Clippy `-D warnings`, native-skia
lib check가 모두 exit 0으로 종료했다. 전체 workspace/all-target Clippy,
전체 회귀와 fresh WASM은 아직 통과 판정하지 않았다.
`stage91-paper-frame-type-checks.json`에 명령·소스 해시·종료 상태를 기록했다.
현재 고정 `target/pr-review`로 Native 빌드를 진행하며 성공 뒤 같은 동결 binary로
이전 미달 9개 원본 전체 쪽을 직접 비교한다.

이전 source960의 집중 검증은 끝난 23개 결과를 스냅샷으로 보존하고
소유 프로세스 2개를 확인해 의도적으로 종료했다. 관측 timeout을 실패로
해석해 재시작한 것이 아니다. `stage91-superseded-old-focus-processes.json`은
PID/start tick·SIGTERM·종료 후 프로세스 부재를 기록한다.
현재 source c6의 새 원본 비교로 대체하며 공유 Cargo cache는 보존했다.

추가 14개 회귀 파일의 24개 함수를 직접 읽고
`stage91-additional-regression-contract-classification.json`으로 분류했다.
#6632의 그림 폭/y 선택과 #6681의 SVG 좌표 대역/baseline은 관계형으로
바꿔야 한다. #6495의 OVERFLOW 건수 상한·마지막 쪽 존재 검사는 내용 소유·
포함·전체 쪽수 보존을 입증하지 못한다. #6186의 저장 속성 왕복, #6646의
내용/공백 메트릭 계약, #6323/6334의 바탕쪽 소유/중복 검사는 고정 px와
구분했다. 원본 실패를 return/continue로 통과시키는 경로도 검토 대상으로 기록했다.
이는 정식 검사 변경이나 91파일 전수 검토 완료 기록이 아니다.

## 빌드 종료와 실제 원본 비교

Native 빌드는 exit 0, 406.677초로 종료했다. binary SHA-256은
`72250c2d1e4283467ac3b1c656113c6e535ee23f3466e0457a34589227426031`다.
`paper-frame-native-provenance.json`은 source c6과 빌드 전후 해시 일치를 기록한다.

같은 binary로 실패 원본 191쪽과 대조군 51쪽, 총 242쪽의 직접 Skia 비교를
완료했다. 실패 원본 9개 중 7개와 대조군 5개는 각각 전쪽 90% 이상이다.
수학 시험지 두 개는 여전히 미달이다. 이는 Native SVG/fresh WASM 전체
55개 원본/853쪽의 최종 통과를 뜻하지 않는다.

### 이전 미달 원본 9개

| 입력 | 전쪽 비교 | 최저 일치율 | 90% 미만 쪽 |
|---|---:|---:|---|
| `samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래20.hwp` | 21쪽 | 96.42088% | 없음 |
| `samples/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwp` | 22쪽 | 95.61623% | 없음 |
| `samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래20.hwpx` | 21쪽 | 96.42080% | 없음 |
| `samples/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwpx` | 22쪽 | 95.61623% | 없음 |
| `samples/3-11월_실전_통합_2024-구분선위20미주사이7구분선아래2.hwp` | 21쪽 | 96.42088% | 없음 |
| `samples/3-11월_실전_통합_2024-구분선위20미주사이7구분선아래2.hwpx` | 21쪽 | 96.42080% | 없음 |
| `samples/exam_math.hwp` | 20쪽 | 78.69607% | 1, 2, 9, 13, 17, 18 |
| `samples/exam_math_no.hwp` | 20쪽 | 77.76098% | 1, 9, 13, 15, 16, 17, 18 |
| `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx` | 23쪽 | 90.53366% | 없음 |

### 셀 도형 대조군 5개

| 입력 | 전쪽 비교 | 최저 일치율 | 90% 미만 쪽 |
|---|---:|---:|---|
| `samples/21_언어_기출_편집가능본.hwp` | 15쪽 | 94.44037% | 없음 |
| `samples/exam_eng.hwp` | 8쪽 | 91.13983% | 없음 |
| `samples/exam_kor.hwp` | 20쪽 | 91.57678% | 없음 |
| `samples/exam_science.hwp` | 4쪽 | 93.59650% | 없음 |
| `samples/exam_social.hwp` | 4쪽 | 95.99778% | 없음 |

### 한 줄 미주의 소유와 이어받기 확인

all-21 HWPX의 19·20·21쪽은 이전 60.80087/30.54841/41.58859%에서
98.31866/97.86713/97.21069%로 개선됐다.
`stage91-next-diagnostic/all21-pages.json`에서 원본 소유
`section0/para340/control0/notePara15`가 19쪽 왼쪽 단의 fullParagraph로
한 번 남는 것을 확인했다. 자매 HWP와 독립 Print의 해당 줄 소속과 일치한다.
확인할 수 없는 소유를 픽셀 대역으로 추정하지 않았다.
새 `stage91-original-skia/all-21/review/review_020.png`을 직접 읽고 사용자에게 표시했다.
수식의 미세 차이까지 전부 해소했다는 판정은 아니다.

### 수학 시험지의 남은 글꼴 차이

새 `stage91-math-page17/review/review_017.png`을 직접 읽고 사용자에게 표시했다.
용지 기준 쪽 번호 위치는 개선됐지만 제목의 획·모양과 일부 기호 차이가 남는다.
단일 쪽 PNG의 CLI 파일명이 번호 없는 `exam_math.png`였으므로 이미 성공한
출력을 `--page 16`에 맞춰 `_017`로 정규화하고 비교만 재실행했다.
파일명·pair 선택 오류는 출력 회귀나 점수 통과로 기록하지 않았다.

별도 canonical Native SVG 17쪽 Sweep도 실행했다. 전체 쪽수는 양쪽 모두 20쪽,
관련 17쪽 일치율은 78.96865%, `re_review_required`다.
`stage91-math-font-native-svg/math-font/silhouette.tsv`와 summary에 보존했다.
따라서 Skia 한 경로만의 차이로 간주하거나 기대값을 고쳐 통과시키지 않는다.

원본 파싱 속성의 큰 제목은 `신명 신그래픽`, bold=true, 4100HU, ratio92다.
작은 머리글은 `신명 디나루`, bold=false, 2100HU, ratio85다. 원본·PDF를
바꾸지 않고 다음 글꼴 공급 분석의 근거로 유지한다.
실제 SVG 내장 font metadata는 `stage91-math-font-supply-inspection.json`으로
확인했다. 표시 family `굴림`의 실제 face는 Gulim Regular(weight400),
`돋움`의 실제 face는 Noto Sans KR ExtraLight(weight400)다.
CSS 이름·시스템 설치 유무만으로 한컴 HFT의 실제 glyph와 같은 출력을
사용한다고 판정하지 않는다. 소스 font substitution과 bold 적용·실제 glyph
공급을 다음에 함께 대조한다. 설치된 Linux Gulim/HYGulim을 확인했으며,
Windows font 목록의 SSH 조회는 exit255여서 검증 성공으로 기록하지 않는다.

정식 회귀 추가/기대값 변경, fresh WASM 전쪽 TSV, 전체 회귀·필수 게이트,
새 PR 생성·CI·통합은 아직 완료되지 않았다. 전체 원본/회귀 범위를 유지한다.
