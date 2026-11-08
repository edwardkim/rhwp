# #7998 stage32 — 본문 source 글줄의 저장·재조판 폭

2026-10-20 다음 작업 진행 승인을 받아 stage31에서 분리한 본문 무편집 저장의 글줄 보존을 진행한다. 기준은 `805c9271ca2e9b99e1363c388eed16b82cf9b0f6`다. 이전 기준선 범위의 직접 통과와 회귀는 유지한다.

## 입력·독립 근거와 원인 경계

`body-basic.pdf`와 `body-baseline.pdf`는 두 baseline의 text-show 명령을 문단으로 인식한 공개 독립 PDF다. 원본 PDF·stage31 저장본·동일 저장본의 독립 한컴 2020 Print는 `/mnt/e/tmp/issue7998-stage31-evidence/`에 보존했다. `LineSeg`의 원본 줄 개수만 복원하고 실제 문단 폭을 페이지 전체로 발행해 한컴의 일반 재조판이 한 줄로 합친다. source 두 글줄은 작성 당시 강제 개행이라는 증거가 아니며, 임의 LineBreak 삽입·그림 대체·개별 글상자로 문단 분할을 하지 않는다.

이번에 추적하는 값은 재조판의 실제 가용 오른쪽 끝이다. `recognition::ParagraphProposal`의 원본 글줄/소속 → `structured_lowering::paragraph`의 관측 glyph 메트릭·문단 여백·LineSeg 폭 → `body::frame`의 TextBox 폭 → `ParagraphBox::content_for_style` → 실제 공통 reflow/painter → 양 저장의 표준 문단 속성으로 연결한다. 원본 글줄의 전진폭이 fit하고 다음 줄 첫 word가 앞 줄에 들어가지 않는 폭의 근거 및 공통 reflow의 검증을 분리한다. 줄 끝의 실제 pitch와 뒤 문단 간격은 같은 공통 row 메트릭으로 대조한다. bbox의 ink 폭만을 실제 원래 문단 폭이라고 단정하지 않는다.

실험 전 Native 실행 파일·rlib를 `/mnt/e/tmp/issue7998-stage32-evidence/before/`에 고정했다. 임시 진단/로그는 ignored `output/pr-review/issue7998/p4-body-reflow/`로 연결한다. 시각 선행 조건 전에는 새 정식 렌더링 회귀·fixture를 등록하지 않는다. 전체 suite·P5·원격 반영은 이번 절편이 아니다. B24/R1·나머지 R2·`parser_allocation_bounds_proven=false`를 유지한다.

## 원인 분리·구현과 현재 확인

폭만 가장 긴 진단 메트릭으로 줄인 임시 HWPX는 한컴에서 3줄로 바뀌었다. 뒤 문단
간격만 보정해도 줄 구성은 그대로여서 basic 47.78%/script 43.06%였다. 이 산출물은
`diagnostic-visual/`에 남기고 제품 규칙이나 정식 fixture로 채택하지 않았다. 하나의 bbox
폭·진단용 반올림 폭을 원래 문단 폭이라고 대입하지 않는다.

제품은 `structured_lowering/wrap_width.rs`에서 각 source 글줄의 기록 전진폭을 최소
범위로 삼고, 같은 공통 `reflow_line_segs`/`ParagraphBox::content_for_style`가 source
UTF-16 row cut을 재현하는 정수 HWPUNIT 폭 구간을 찾는다. 두 번의 유한 이분 탐색이며
한 단위씩 전수 탐색하지 않는다. 구간 중간값은 임계 word가 양자화 경계에 놓이는 것을
피하는 표현 선택이다. 실제 작성 폭은 unknown으로 남기고, 구간·결과를 opt-in 진단의
`sourceTypography.lines[].paragraphReflow`에 보존한다. 좌우 여백만 표준 속성으로 발행하고
강제 개행/문단 분할/그림 대체를 하지 않는다. 가운데 정렬은 같은 중심의 대칭 상자로
검증한다. 모든 source 컷과 baseline·원점을 검증한 뒤 공통 재조판 결과 자체를 발행한다.

폭의 소비는 `paragraph` 결과 → body `prepare`의 wrap 검증 → `body::frame`의 소유
TextBox → 편집의 `ParagraphBox` → 일반 측정/paint 및 HWP/HWPX 문단 여백이다.
`paragraph_flow_end`가 반환한 마지막 줄의 높이+후행 간격을 FlowItem bottom으로
소비해 뒤 문단 spacing_after를 산출한다. basic의 원래 마지막 줄 gap=0/후속 간격26pt
대신 공통 마지막 gap=4pt/후속 간격22pt가 같은 최종 다음 원점을 만든다. 측정에서
paint의 제한을 모방하지 않으며 같은 row 결과와 점유 끝점을 사용한다.

들여쓰기/목록은 공통 fresh flow의 원점이 아직 source와 달라 승격하지 않는다. 원문과
source LineSeg를 유지하고 `source_reflow_unverified` 사유를 남긴다. 임의로 column_start를
덮거나 indent를 0으로 지우지 않는다. 나머지 R2의 후속 경계이며 이번의 호환 완료 주장에
넣지 않는다. 두 입력의 실제 serialized HWPX도 수정 전과 바이트 동일함을 별도로 확인한다.

초기 원인 분리 검사 `before_contract.rs`는 실제 frozen 기준에서 두 source 줄이 직접
삽입·삭제 후 한 줄로 합쳐지는 FAIL을 검출했다. 이어 `wrap_contract.rs` 전체를 **동일한
소스**로 frozen rlib와 현재 rlib에 각각 링크해 before FAIL/after PASS를 확인했다. 5종의
PDF/HWP/HWPX 15경로에서 실제 삽입 paint·직접 삭제·양 저장 재열기의 문단 text/row
소속과 첫 줄·다음 문단 기준선 관계를 확인한다. 별도로 suffix를 지우면 한 줄로 줄고
다음 문단이 실제 점유에 맞춰 이동한다. 과거 source 줄 수를 모든 편집 뒤 고정하지 않는다.
새 정식 회귀 등록은 아래 fresh WASM·독립 Print·직접 판독 이후에 진행했다.

Native 최종 빌드 2분44초와 Docker fresh WASM 7분37초를 완료했다. source 파일의 SHA·
Native/WASM SHA와 JS glue는 `source-runtime.json`에 고정했다. 기존 자동 수집 #7998
21개 모듈 157건·문단 여백 12건은 현재 제품에서 PASS다. 실제 Studio WASM 15경로의
삽입 표시/직접 삭제/undo/redo/양 저장도 PASS이며, PDF 경로 편집 저장본과 suffix 삭제
대조군의 bytes는 Native와 같다. source 16문서의 실제 TextBox 문단·표/셀/중첩/원문 소유
그룹과 쪽수는 수정 전후 같다(`semantic-owner-comparison.json`).

독립 한컴 Print는 같은 입력인 초기 2건을 재사용한 최종 20개 작업/10개 byte-identical
alias로 30개 저장본을 대조하고, 한 줄로 줄인 편집 대조군 2개도 새 Print를 확보했다.
source PDF 26개37쪽+저장본32개32쪽의 전체 TSV는 Native/fresh WASM 각각 58입력69쪽,
최저 **93.56061% / 93.50649%**다. 양쪽 모두 미달·누락·쪽수 불일치 0이다. 이전 4개
미달의 새 source 저장본은 이제 정상 두 줄을 생성한다. 과거 잘못 생성된 파일 자체를
자동 복구한 것으로 보고하지 않는다.

## 전쪽 대조·직접 판독과 수용

검증 명령과 각 입력·Print의 SHA는 `native/results.json`과 `wasm/results.json`에 있다.
실행은 `python3 output/pr-review/issue7998/p4-body-reflow/validate.py native` 및 같은
명령의 `wasm` 인수이며, 내부의 canonical `scripts/visual_sweep.py --silhouette-only`
명령 전체와 파일별 TSV를 보존했다. source와 같은 바이트의 저장본·동일 runtime에 한해
캐시를 재사용한다. 기준 전쪽 PNG는 frozen CLI/WASM·입력 SHA가 같은 stage31 출력을
연결했고, 기준을 다시 빌드·캡처하지 않았다.

원본 26개37쪽의 PNG는 각 backend에서 35쪽이 같다. 바뀐 두 쪽은 보고서 3쪽의
글자 전진폭 서브픽셀 변화(최대 raw 하락 Native **0.00011%p**, WASM **0.00029%p**)와
견적서 1쪽의 RGB 8/6픽셀 차이다. 후자는 Native raw가 같고 WASM은 0.00001%p 높다.
`source-page-deltas.json`, `native-pixel-deltas.json`, `wasm-pixel-deltas.json`과 같은 영역의
원본/기준/현재 두 backend 확대 PNG를 연결했다. 큰 원점 이동·줄바꿈·누락은 없으며,
색상 경계의 작은 차이를 전체 PNG 동일로 보고하지 않는다.

직접 판독에서 일반 본문·첨자·가운데 정렬·두 단의 두 줄과 다음 문단, 한 줄 축소를
독립 한컴 Print 및 현재 Native/fresh WASM에서 확인했다. 한컴의 Haansoft Batang
대체와 PDF Helvetica의 글꼴 모양 차이는 확대에 남겼다. 비교 화면은
`E:\tmp\issue7998-body-wrap-review\index.html`이며 전체 TSV 원문·대표 compare/overlay·
실제 저장본·독립 Print·소스/runtime 지문·실제 Studio 결과를 함께 제공한다.

Studio 7793의 `?review=issue7998-body-wrap-basic`, `...-script`, `...-center`, `...-short`
새 네 탭에 실제 저장본을 열고 위 fresh WASM SHA의 실제 로드를 확인했다
(`human-review-tabs.json`). 기존 탭과 7700 서버는 유지했다. 작업지시자는 2026-10-20
이번 본문 글줄 범위를 **“시각·편집 구조 통과”**로 판정했다. 원본의 작은 raw 하락과
한컴 대체 글꼴 차이도 같은 비교 화면에 제시한 범위다.

정식 검사 후보는 자동 수집 밖의 `registered_candidate.rs`에 정리했다. 독립 PDF
text-show의 literal row/문단 기대값, 두 단의 양쪽 소유, 세 입력 형식, 실제 삽입 표시와
직접 삭제, 최종 paint의 글자 순서/단일 보존, 양 저장 재열기와 자연스러운 한 줄 축소를
검사한다. 같은 함수 소스 전체를 frozen 기준 rlib와 현재 rlib에 각각 링크하여 의도한
두 줄→한 줄 원인으로 **FAIL→PASS**를 확인했다(`registered-candidate-before.log`,
`registered-candidate-after.log`, `registered-candidate-before-after.json`). 사용자 판정 뒤
동일 바이트의 함수를 `tests/cases/issue_7998_pdf_body_soft_wrap.rs`로 등록했다. 기존
169건은 이번 제품에서 이미 통과했으며 새 1건 때문에 다시 실행하지 않았다. **기존
157+여백12+신규1=170 PASS**이며 전체 170건을 묶어 재실행한 것으로 보고하지 않는다.

자동 수집 `node scripts/rust-test-suite-manifest.mjs --prepare`와 `--check`를 통과했다.
새 모듈은 `regression_suite_008`에 배정되며 1565 sources·6972 static attrs·28 suites+
20 exceptions/48 targets다. CI에 별도 호출을 추가하지 않았다. 입력 fixture 추가 없이
기존 공개 PDF 다섯 개를 재사용했다. 제품 source SHA와 등록 함수 SHA가 실제 검사·
빌드·전쪽 비교 시점과 같음을 확인했으므로 빌드·Print·캡처를 반복하지 않았다.

제품 코드는 `structured_lowering.rs`의 `wrap_width` 연결, `body.rs`의 `prepare` 소비와
새 `wrap_width.rs`다. 기존 편집/paint/저장 코드를 바꾸지 않고 공통 `ParagraphBox`·
`reflow_line_segs`·`paragraph_flow_end`의 결과를 발행한다. `rustfmt --check`와
`git diff --check`를 확인했다. Studio JS는 내용이 Git 기준과 같고 빌드 복사 과정의
실행 비트만 달랐으므로 0644로 복구했다. 실행 중인 새 WASM 산출물은 유지한다.

이번 로컬 절편의 수용과 본문 글줄 회귀 등록을 마감한다. 다음은 아직 source와 공통
재조판 원점이 일치하지 않는 들여쓰기/목록 및 나머지 B10–B13 메트릭 경계다. R1/B24·
P5·한컴 비사각 클립 호환 등 기존 미완료/예외를 해소한 것으로 보고하지 않는다.
전체 회귀·전체 lint·Worker/package·원격 push·PR은 실행하지 않았으며,
`parser_allocation_bounds_proven=false`를 유지한다.
