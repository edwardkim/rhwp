# #8982·#8983 로컬 검증 원장

## 범위와 현재 상태

원 기여 SHA 두 개를 보존한 누적 후보에서 #8982의 속성 이름/값 오인을 공통 파서에서 보정했다. 승인 범위는 로컬 보완과 필수 검증이다. push·새 PR·공개 댓글·merge는 아직 수행하지 않았다. 현재 필수 전체 회귀·lint·역할별 보호 검증을 진행 중이므로 최종 제출 완료 판정은 아니다.

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
- 위 진단은 정식 suite와 최소 글꼴 환경의 전체 게이트를 대체하지 않는다. 해당 실행 결과와 Studio E2E·보호 전수·handbook 결과는 완료한 뒤 보완한다.
