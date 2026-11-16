# #8982·#8983 로컬 검증 원장

## 범위와 현재 상태

원 기여 SHA 두 개를 보존한 후보에서 공통 속성 경계 오류를 보완했다. 로컬 검증과 승인된 원격 처리로 [수용 PR #9017](https://github.com/edwardkim/rhwp/pull/9017)의 exact-head CI·self-review·merge를 완료했다. 두 원 PR은 MERGED, #8942·#8943은 CLOSED이며 #7992는 OPEN을 유지한다. [최종 검토](../../archives/bongseop_kim_20261112_review.md)와 [CI·병합 영수증](remote/merge-and-ci.json)을 참고한다.

## 원인과 소비 경로

- #8982: 기존 숫자 속성 helper의 부분 문자열 검색이 `data-border="1"`을 실제 border로 오인했다. `html_quoted_attribute`가 첫 태그의 이름·따옴표 값 경계를 읽고 숫자 속성과 이미지 style 추출이 공통 결과를 소비한다. `parse_html_attr_f64`의 단위 변환은 유지했다. 표 border의 양수는 칸 border fill의 기본 선으로 전달되며 셀 CSS가 우선한다. 저장기 → 공통 border/paint → Native Skia와 WASM Canvas의 실제 PNG를 확인했다.
- #8983: 첫 CharShapeRef가 0보다 뒤에서 시작하면 첫 서식이 앞 평문에도 적용됐다. 원 기여의 0 시작 평문 run을 유지한다. 원 HTML → 공통 inline parser → CharShapeRef → 문단/셀 편집 → HWP/HWPX 저장/재파싱 → 공통 text paint를 확인한다. 측정·pagination 수치나 backend별 보정을 추가하지 않았다.
- 기존 #7516 → #7521 → #7601과 현재 devel에 실제 포함된 재배치 commit `cb2bd912d5afc28e01015d79caf839664c942e63`을 추적했다. 4,000자 경계·Unicode·그림 소유를 포함한 기존 8개 검사를 유지했다.

## 독립 Print와 생성 출처

`hancom-jobs.json`은 각 입력 SHA-256·원 생성 source·job·다운로드 PDF SHA-256을 연결한다. 입력은 RHWP의 실제 공개 편집 API가 생성한 합성 문서다. 파일에 포함된 Hancom 11.x 호환 metadata를 한컴이 저장한 원본이라는 뜻으로 사용하지 않는다. HWP/HWPX의 동일 저장본을 한컴오피스 2020 `11.0.0.9136`에서 인쇄했다. 모든 job의 `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`을 확인했다. Print PDF는 `pdf/issue8942`, `pdf/issue8943`, `pdf/issue8942-8943`에 직접 저장했다.

## 전쪽 실제 출력

2026-11-12, 96dpi, PDFium 독립 PDF raster와 색상 경계 보정 전 2px 내용 실루엣 raw를 사용했다. 최적화를 마친 web package와 브라우저 Canvas를 실행했다. 각 결과는 실제 PNG 내보내기와 `--png-pair --silhouette-only`로 계산했으며 `render-run-manifest.json`과 `silhouette_manifest.json`을 함께 보존한다. raw 100%는 RGB 바이트 완전 일치라는 뜻이 아니다.

| 저장본·실행 조건 | 전체 쪽수 | Native 전쪽 최저 | fresh WASM 전쪽 최저 |
| --- | ---: | ---: | ---: |
| border-hwp / border-hwpx | 각 1 | 각 100.00% | 각 100.00% |
| leading-hwp / leading-hwpx | 각 1 | 각 100.00% | 각 100.00% |
| combined-hwp / combined-hwpx | 각 1 | 각 100.00% | 각 100.00% |
| data-only-hwpx / data-before-zero-hwpx | 각 1 | 각 100.00% | 각 100.00% |
| value-fake-before-zero-hwpx / whitespace-case-hwpx | 각 1 | 각 100.00% | 각 100.00% |
| styled-caret-hwpx / styled-cell-hwpx | 각 1 | 각 100.00% | 각 100.00% |

Native/fresh WASM 각각 12문서·12쪽 전부 측정했고 누락·쪽수 불일치가 없다. `focus/<backend>/<key>/`에 전쪽 TSV를 보존했다. `direct-review/`는 같은 영역·배율의 독립 Print와 실제 출력이다. 표 선과 양쪽 칸, 본문 앞뒤 순서, 앞 평문의 보통 글자와 명시한 굵게, 기울임꼴 삽입점의 앞뒤 글자를 직접 판독한다. 글꼴 프로그램과 SVG의 글꼴 payload는 커밋하지 않는다. 실제 공급·fallback hash는 실행 manifest에 기록한다.

이 수치는 위 저장본의 실제 출력 근거다. 별도의 합성 API assertion은 입력 문자·속성과 출력 모델/저장 관계를 검사하며, 다른 입력의 측정값을 그 API 입력의 픽셀 점수로 쓰지 않는다. 절대 px·쪽수 pin을 추가하지 않았다.

## 선행 진단과 최종 게이트 구분

- 원 코드의 7개 검사와 기존 #7516 8개: 최신 base에서 원 버그 4 FAIL·나머지 11 PASS, 원 기여 누적 적용 후 15 PASS.
- 공통 파서 보정 `0aa25a6634`: 같은 15 PASS, HWP/HWPX 두 차례 저장의 의미 보존 12 PASS, 속성 오인 0건. 추가 속성 경계 8개·span·서식 있는 본문/셀 삽입점의 진단도 통과했다.
- 위 진단은 정식 suite와 최소 글꼴 환경의 전체 게이트를 대체하지 않는다. 최종 실행 결과와 Studio E2E·보호 전수·handbook 결과는 아래 최종 절과 실제 영수증에 보완했다.

## 실제 Studio 경로

CDP 19222의 Windows Chrome에 별도 페이지를 열고 새 최적화 WASM을 로드했다. `studio-e2e/result.json`의 6개 사례는 textarea의 실제 제품 paste 이벤트 처리 → 편집 명령 → WASM을 실행한다. 이벤트는 테스트 드라이버가 구성한 `ClipboardEvent`이며 운영체제 클립보드 단축키 시험이라고 주장하지 않는다. 처리기를 대체하거나 성공 결과를 mock하지 않았다.

- 첫 문단 뒤 평문/굵게, 명시적 표 선, 두 기능의 복합 입력, 기울임꼴 본문 삽입점, 기울임꼴 셀 삽입점, data-border와 실제 border=0의 분리를 확인했다.
- 각 사례에서 HWP/HWPX를 두 차례 저장·재열기하여 총 24개 비교를 통과했다. 내용·서식·칸 선의 관계를 비교했고 저장 ID를 golden으로 고정하지 않았다. 실행한 저장본과 화면은 `studio-e2e/`에 보존한다.
- `studio-e2e/consumption/`은 브라우저 초기화에서 수신한 실제 WASM body의 SHA-256을 빌드 산출물과 비교한 영수증이다. 생성 source와 최적화 명령/산출물 hash는 위 빌드 영수증과 연결한다.
- 초기 드라이버의 짧은 문서 준비 대기와 cell control index=0 가정을 바로잡았다. 실제 생성 API는 control index=2를 반환했으며 그 반환값을 사용해 통과했다. 제품 코드 보정이나 경고 억제를 추가하지 않았다.

## 기존 기능과 특별 보호

- 기존 #7521/#7601의 독립 Print 원 사례 1쪽도 Native/fresh WASM 각각 raw 100.00%다. 주요 12개와 합친 13문서·13쪽씩을 실제 기준 devel/후보 생산자로 다시 열었고, 26개 PNG가 바이트 단위로 같다. `focus-base-head-comparison.json`과 `focus/base-*/`에 근거를 보존했다. HTML 붙여넣기 자체의 수정 전 실패와 이 **같은 저장본**의 열기 대조를 혼동하지 않는다.
- `wasm-base-head-probe.json`은 서로 다른 source에서 최적화를 완료한 두 실제 WASM을 브라우저에서 초기화했다. base의 표 선 누락/앞 평문의 굵게와 head의 올바른 선/평문을 확인했다. 실제 WASM body hash도 양쪽 package와 일치한다.
- `before-api/`와 `direct-review/before-after/`는 동일 HTML을 수정 전 API에 넣어 얻은 잘못된 저장 결과의 진단이다. 정상 fixture나 기대값으로 등록하지 않았다. 이 그림을 다른 입력 파일의 정상 raw 점수로 승격하지 않는다.
- 정본 보호 18문서, Native/fresh WASM 각각 1,771쪽·합계 3,542행을 전수 측정했다. 공식 두 자리 표시값 하락 0건이며 예외를 적용하지 않았다. `protected-comparison.json`·`protected-comparison.tsv`, `protection/<backend>/`의 전쪽 TSV와 정밀 manifest를 보존했다.
- 별도 필수 `--handbook-gate`도 현재 회귀 head `39f0ac0477`에서 실행해 Native HWPX 383쪽을 통과했다. `handbook-gate/`와 `handbook-gate-receipt.json`을 참고한다.
- 생산자 build source는 `0aa25a6634`, 정식 회귀 head는 `39f0ac0477`이다. 둘 사이의 Rust 제품·의존성·빌드/출력 script 변경은 0개이며 `product-source-equivalence.json`에 기록했다. 각각의 빌드·실행 SHA를 바꾸어 쓰지 않는다. 새 code/test head의 필수 lint·전체 회귀와 승인된 수용 PR의 exact-head GitHub CI도 별도로 완료했다.
- 보호 검증의 독립 구형 수식 문자 표본을 현재 정본 도구로 새로 인쇄·준비했다. 과거 수용된 표본과 문자별 원 프로그램 hash·변환이 동일함을 확인했다. `font-supply/`에는 출처·인쇄·hash 영수증만 남기며 글꼴 프로그램·표본 PDF는 포함하지 않는다.

## 최종 로컬 게이트

회귀 head `39f0ac0477`에서 최소 글꼴 환경으로 실행했다. `final-gate-receipts.json`의 13개 명령은 모두 종료 코드 0이다. 전체 회귀 13,341개(기존 조건 skip 45개), 집중 23개, Native 공식 3종 합계 4,073개(공식 lib 4,035·placeholder 2·direct PDF 36), 모든 lint와 workspace build가 통과했다. Native library의 기존 ignored 12개도 그대로 기록했다. 실행 로그 텍스트(마지막 빈 줄은 LF 한 개로 정규화)는 `gates/*.txt`, 집계는 `validation.json`에 보존했다.

실행 세션 수집기에서는 최종 상태 143을 반환했다. 13개 child 명령의 실제 종료 코드·PASS 원 로그와 마지막 완료 marker를 모두 대조해 완료를 확인했고, 이 수집기 상태도 `validation.json`에 남겼다. 검사 실패를 지우거나 전체 회귀를 반복하지 않았다.
